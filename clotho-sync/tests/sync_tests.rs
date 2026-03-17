use std::fs;

use tempfile::tempdir;

use clotho_sync::SyncEngine;

/// Helper: create a .clotho/ directory structure + visible dirs in a temp dir.
fn setup_workspace(tmp: &tempfile::TempDir) -> std::path::PathBuf {
    let ws = tmp.path().join(".clotho");
    // Machine dirs in .clotho/
    fs::create_dir_all(ws.join("data")).unwrap();
    fs::create_dir_all(ws.join("graph")).unwrap();
    fs::create_dir_all(ws.join("index")).unwrap();
    fs::create_dir_all(ws.join("inbox")).unwrap();
    fs::create_dir_all(ws.join("config")).unwrap();
    fs::write(ws.join("config/config.toml"), "[sync]\nauto_commit = true\n").unwrap();
    // Visible content dirs at project root
    for dir in &["programs", "tasks", "meetings", "notes", "people"] {
        fs::create_dir_all(tmp.path().join(dir)).unwrap();
    }
    ws
}

// ===========================================================================
// Init
// ===========================================================================

#[test]
fn init_creates_git_repo() {
    let tmp = tempdir().unwrap();
    let ws = setup_workspace(&tmp);

    let engine = SyncEngine::init(&ws).unwrap();
    assert!(tmp.path().join(".git").exists());
    assert!(!engine.has_remote());
}

#[test]
fn init_creates_gitignore() {
    let tmp = tempdir().unwrap();
    let ws = setup_workspace(&tmp);

    SyncEngine::init(&ws).unwrap();
    let gitignore = fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
    assert!(gitignore.contains(".clotho/index/"));
    assert!(gitignore.contains(".clotho/inbox/"));
}

#[test]
fn init_is_idempotent() {
    let tmp = tempdir().unwrap();
    let ws = setup_workspace(&tmp);

    SyncEngine::init(&ws).unwrap();
    SyncEngine::init(&ws).unwrap(); // Should not fail
    assert!(tmp.path().join(".git").exists());
}

// ===========================================================================
// Open
// ===========================================================================

#[test]
fn open_existing_repo() {
    let tmp = tempdir().unwrap();
    let ws = setup_workspace(&tmp);

    SyncEngine::init(&ws).unwrap();
    let engine = SyncEngine::open(&ws).unwrap();
    assert!(!engine.has_remote());
}

#[test]
fn open_fails_without_git() {
    let tmp = tempdir().unwrap();
    let ws = setup_workspace(&tmp);

    let result = SyncEngine::open(&ws);
    assert!(result.is_err());
}

// ===========================================================================
// Sync
// ===========================================================================

#[test]
fn sync_no_changes() {
    let tmp = tempdir().unwrap();
    let ws = setup_workspace(&tmp);

    let engine = SyncEngine::init(&ws).unwrap();
    // First sync commits initial files
    let r1 = engine.sync().unwrap();
    assert!(r1.committed);

    // Second sync — no changes
    let r2 = engine.sync().unwrap();
    assert!(!r2.committed);
    assert_eq!(r2.files_changed, 0);
}

#[test]
fn sync_commits_new_file() {
    let tmp = tempdir().unwrap();
    let ws = setup_workspace(&tmp);

    let engine = SyncEngine::init(&ws).unwrap();
    engine.sync().unwrap(); // initial commit

    // Create a new file
    fs::write(tmp.path().join("notes/test-note.md"), "# Hello").unwrap();

    let result = engine.sync().unwrap();
    assert!(result.committed);
    assert!(result.files_changed > 0);
    assert!(!result.pushed); // no remote
}

#[test]
fn sync_commits_modified_file() {
    let tmp = tempdir().unwrap();
    let ws = setup_workspace(&tmp);

    let engine = SyncEngine::init(&ws).unwrap();
    fs::write(tmp.path().join("notes/note.md"), "version 1").unwrap();
    engine.sync().unwrap();

    // Modify the file
    fs::write(tmp.path().join("notes/note.md"), "version 2").unwrap();
    let result = engine.sync().unwrap();
    assert!(result.committed);
}

#[test]
fn sync_has_remote_false() {
    let tmp = tempdir().unwrap();
    let ws = setup_workspace(&tmp);

    let engine = SyncEngine::init(&ws).unwrap();
    assert!(!engine.has_remote());
}

// ===========================================================================
// Commit count
// ===========================================================================

#[test]
fn commit_count_tracks() {
    let tmp = tempdir().unwrap();
    let ws = setup_workspace(&tmp);

    let engine = SyncEngine::init(&ws).unwrap();
    assert_eq!(engine.commit_count().unwrap(), 0);

    engine.sync().unwrap();
    assert_eq!(engine.commit_count().unwrap(), 1);

    fs::write(tmp.path().join("notes/a.md"), "a").unwrap();
    engine.sync().unwrap();
    assert_eq!(engine.commit_count().unwrap(), 2);

    fs::write(tmp.path().join("notes/b.md"), "b").unwrap();
    engine.sync().unwrap();
    assert_eq!(engine.commit_count().unwrap(), 3);
}

// ===========================================================================
// Prune
// ===========================================================================

#[test]
fn prune_reduces_commit_count() {
    let tmp = tempdir().unwrap();
    let ws = setup_workspace(&tmp);

    let engine = SyncEngine::init(&ws).unwrap();

    // Create 5 commits
    for i in 0..5 {
        fs::write(tmp.path().join(format!("notes/note-{}.md", i)), format!("note {}", i)).unwrap();
        engine.sync().unwrap();
    }
    assert_eq!(engine.commit_count().unwrap(), 5);

    // Prune to keep 2
    let pruned = engine.prune_history(2).unwrap();
    assert_eq!(pruned, 3);
    assert_eq!(engine.commit_count().unwrap(), 2);
}

#[test]
fn prune_noop_when_under_limit() {
    let tmp = tempdir().unwrap();
    let ws = setup_workspace(&tmp);

    let engine = SyncEngine::init(&ws).unwrap();
    fs::write(tmp.path().join("notes/a.md"), "a").unwrap();
    engine.sync().unwrap();

    let pruned = engine.prune_history(20).unwrap();
    assert_eq!(pruned, 0);
}

// ===========================================================================
// Gitignore respects index/
// ===========================================================================

#[test]
fn index_directory_not_committed() {
    let tmp = tempdir().unwrap();
    let ws = setup_workspace(&tmp);

    let engine = SyncEngine::init(&ws).unwrap();

    // Create files in index/ (should be ignored)
    fs::write(ws.join("index/search.db"), "fake db").unwrap();
    // Create file in content/ (should be committed)
    fs::write(tmp.path().join("notes/note.md"), "real content").unwrap();

    let result = engine.sync().unwrap();
    assert!(result.committed);

    // Verify index file is not tracked
    let repo = engine.repository();
    let head = repo.head().unwrap();
    let tree = head.peel_to_tree().unwrap();

    // Walk the tree — should not contain index/
    let mut has_index = false;
    tree.walk(git2::TreeWalkMode::PreOrder, |dir, _entry| {
        if dir.contains("index") {
            has_index = true;
        }
        git2::TreeWalkResult::Ok
    }).unwrap();
    assert!(!has_index, "index/ should not be in the git tree");
}
