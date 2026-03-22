use std::fs;
use std::path::Path;

use git2::Repository;
use thiserror::Error;

/// Errors from sync operations.
#[derive(Debug, Error)]
pub enum SyncError {
    #[error("git error: {0}")]
    GitError(#[from] git2::Error),

    #[error("no git repository found at {0}")]
    NoRepository(String),

    #[error("sync failed: {0}")]
    SyncFailed(String),

    #[error("prune failed: {0}")]
    PruneFailed(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result of a sync operation.
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Whether a commit was created.
    pub committed: bool,
    /// Whether changes were pushed to remote.
    pub pushed: bool,
    /// Number of files changed in the commit.
    pub files_changed: usize,
}

/// .gitignore content for Clotho workspaces.
const GITIGNORE_CONTENT: &str = "\
# Clotho derived indexes (rebuilt on clone)
.clotho/index/
# Clotho inbox (transient staging area)
.clotho/inbox/
";

/// Git-based sync engine for Clotho workspaces.
///
/// Uses git as a dumb sync layer — not version control.
/// Auto-commits with timestamp messages, pushes silently.
pub struct SyncEngine {
    repo: Repository,
}

impl SyncEngine {
    /// Initialize a git repository for a Clotho workspace.
    ///
    /// The git repo is created at the workspace's parent directory
    /// (i.e., the directory containing `.clotho/`).
    /// Writes a .gitignore to exclude `index/`.
    pub fn init(workspace_path: &Path) -> Result<Self, SyncError> {
        // workspace_path is the .clotho/ dir, repo lives in parent
        let repo_path = workspace_path
            .parent()
            .ok_or_else(|| SyncError::SyncFailed("workspace has no parent directory".into()))?;

        let repo = if repo_path.join(".git").exists() {
            Repository::open(repo_path)?
        } else {
            Repository::init(repo_path)?
        };

        // Write .gitignore
        let gitignore_path = repo_path.join(".gitignore");
        if !gitignore_path.exists() {
            fs::write(&gitignore_path, GITIGNORE_CONTENT)?;
        } else {
            // Ensure index/ is in .gitignore
            let existing = fs::read_to_string(&gitignore_path)?;
            if !existing.contains(".clotho/index/") || !existing.contains(".clotho/inbox/") {
                let mut content = existing;
                if !content.ends_with('\n') {
                    content.push('\n');
                }
                content.push_str(GITIGNORE_CONTENT);
                fs::write(&gitignore_path, content)?;
            }
        }

        Ok(Self { repo })
    }

    /// Open an existing git repository for a Clotho workspace.
    pub fn open(workspace_path: &Path) -> Result<Self, SyncError> {
        let repo_path = workspace_path
            .parent()
            .ok_or_else(|| SyncError::NoRepository("workspace has no parent".into()))?;

        if !repo_path.join(".git").exists() {
            return Err(SyncError::NoRepository(repo_path.display().to_string()));
        }

        let repo = Repository::open(repo_path)?;
        Ok(Self { repo })
    }

    /// Check whether the repository has a remote named "origin".
    pub fn has_remote(&self) -> bool {
        self.repo.find_remote("origin").is_ok()
    }

    /// Access the underlying git2 Repository.
    pub fn repository(&self) -> &Repository {
        &self.repo
    }

    /// Sync the workspace: stage all changes, commit, and push (if remote).
    ///
    /// Flow: add all → check for changes → commit → pull-rebase → push
    pub fn sync(&self) -> Result<SyncResult, SyncError> {
        let mut index = self.repo.index()?;

        // Stage all files (respects .gitignore)
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        // Check if there are staged changes
        let tree_oid = index.write_tree()?;
        let tree = self.repo.find_tree(tree_oid)?;

        let has_head = self.repo.head().is_ok();
        let parent_commit = if has_head {
            let head = self.repo.head()?;
            let commit = head.peel_to_commit()?;
            // Check if tree differs from HEAD
            let head_tree = commit.tree()?;
            if tree.id() == head_tree.id() {
                return Ok(SyncResult {
                    committed: false,
                    pushed: false,
                    files_changed: 0,
                });
            }
            Some(commit)
        } else {
            None
        };

        // Count changed files
        let diff = if let Some(ref parent) = parent_commit {
            self.repo
                .diff_tree_to_tree(Some(&parent.tree()?), Some(&tree), None)?
        } else {
            self.repo.diff_tree_to_tree(None, Some(&tree), None)?
        };
        let files_changed = diff.stats()?.files_changed();

        // Create commit with timestamp message
        let now = chrono::Utc::now();
        let message = format!("clotho sync: {}", now.format("%Y-%m-%d %H:%M:%S"));

        let sig = self
            .repo
            .signature()
            .unwrap_or_else(|_| git2::Signature::now("clotho", "clotho@localhost").unwrap());

        let parents: Vec<&git2::Commit> = match parent_commit {
            Some(ref c) => vec![c],
            None => vec![],
        };

        self.repo
            .commit(Some("HEAD"), &sig, &sig, &message, &tree, &parents)?;

        // Push if remote is configured
        let pushed = if self.has_remote() {
            self.push().unwrap_or(false)
        } else {
            false
        };

        Ok(SyncResult {
            committed: true,
            pushed,
            files_changed,
        })
    }

    /// Push to origin/main. Returns true if push succeeded.
    fn push(&self) -> Result<bool, SyncError> {
        let mut remote = self.repo.find_remote("origin")?;

        // Set up callbacks for authentication
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });

        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // Determine the current branch name
        let head = self.repo.head()?;
        let refname = head.name().unwrap_or("refs/heads/main");
        let refspec = format!("{}:{}", refname, refname);

        remote.push(&[&refspec], Some(&mut push_options))?;
        Ok(true)
    }

    /// Prune history to keep only the most recent `keep` commits.
    ///
    /// Squashes all older commits into a single orphan root commit,
    /// preserving the current HEAD tree state.
    pub fn prune_history(&self, keep: usize) -> Result<usize, SyncError> {
        let head = self
            .repo
            .head()
            .map_err(|_| SyncError::PruneFailed("no HEAD".into()))?;
        let head_commit = head
            .peel_to_commit()
            .map_err(|e| SyncError::PruneFailed(e.to_string()))?;

        // Walk history to count commits
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push(head_commit.id())?;
        revwalk.set_sorting(git2::Sort::TOPOLOGICAL)?;

        let commit_ids: Vec<git2::Oid> = revwalk.filter_map(|r| r.ok()).collect();

        let total = commit_ids.len();
        if total <= keep {
            return Ok(0); // Nothing to prune
        }

        // Get the commit at position `keep` (the new root)
        // We'll create a new orphan commit with the tree from that commit,
        // then rebase the recent `keep` commits on top of it.

        // Simpler approach: create a new orphan commit with HEAD's tree,
        // containing the message "clotho: pruned history", then reset HEAD to it.
        // This loses all history but keeps the working tree intact.
        // For a sync layer this is fine — history is not meaningful.

        // More nuanced: keep the last `keep` commits by re-parenting
        // the oldest-kept commit as an orphan.

        let keep_oldest_id = commit_ids[keep - 1];
        let keep_oldest = self.repo.find_commit(keep_oldest_id)?;

        let sig = self
            .repo
            .signature()
            .unwrap_or_else(|_| git2::Signature::now("clotho", "clotho@localhost").unwrap());

        // Create orphan commit with the same tree as the oldest-kept commit
        let orphan_oid = self.repo.commit(
            None, // don't update any ref
            &sig,
            &sig,
            "clotho: pruned history",
            &keep_oldest.tree()?,
            &[], // no parents = orphan
        )?;

        let orphan_commit = self.repo.find_commit(orphan_oid)?;

        // Now replay the remaining keep-1 commits on top of the orphan
        // by rewriting them with new parents
        let mut new_parent = orphan_commit;

        // Walk from oldest-kept-1 to HEAD (reverse order)
        for &cid in commit_ids[..keep - 1].iter().rev() {
            let old_commit = self.repo.find_commit(cid)?;
            let new_oid = self.repo.commit(
                None,
                &old_commit.author(),
                &old_commit.committer(),
                old_commit.message().unwrap_or(""),
                &old_commit.tree()?,
                &[&new_parent],
            )?;
            new_parent = self.repo.find_commit(new_oid)?;
        }

        // Reset HEAD to the new tip
        self.repo
            .reset(new_parent.as_object(), git2::ResetType::Soft, None)?;

        let pruned = total - keep;
        Ok(pruned)
    }

    /// Count the number of commits in the repository.
    pub fn commit_count(&self) -> Result<usize, SyncError> {
        let head = match self.repo.head() {
            Ok(h) => h,
            Err(_) => return Ok(0),
        };
        let commit = head.peel_to_commit()?;
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push(commit.id())?;
        Ok(revwalk.count())
    }
}
