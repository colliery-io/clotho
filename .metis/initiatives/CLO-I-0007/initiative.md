---
id: git-sync-layer-clotho-sync
level: initiative
title: "Git Sync Layer (clotho-sync)"
short_code: "CLO-I-0007"
created_at: 2026-03-16T13:23:16.478907+00:00
updated_at: 2026-03-17T13:41:31.630979+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: S
initiative_id: git-sync-layer-clotho-sync
---

# Git Sync Layer (clotho-sync)

## Context

The `clotho-sync` crate implements git as a dumb sync layer for replicating the workspace across devices. This is explicitly not version control — it's silent, automatic replication with shallow history. Single-user assumption means conflicts are rare (same user, different device).

## Goals & Non-Goals

**Goals:**
- `SyncEngine` using git2 (libgit2) — no shell-out to git
- Sync-on-write: called after every workspace mutation (via StoreSync or CLI/MCP write paths)
- Pull-before-push to handle multi-device scenarios
- Auto-commit with timestamp message
- Silent push after each commit
- Shallow history pruning (~20 commits)
- Main branch only, .gitignore for index/
- `clotho sync` CLI command for manual sync
- `clotho_sync` MCP tool

**Non-Goals:**
- Filesystem watcher / debounce daemon (explicit sync-on-write is sufficient for v1)
- Version control semantics (branching, tagging, meaningful commit messages)
- Multi-user collaboration or merge conflict UI
- SSH key management (uses system git credentials)

## Detailed Design

### Sync Model

- **Synced:** `content/`, `data/`, `graph/`, `config/`
- **Gitignored:** `index/` (rebuilt on clone from synced data)

### SyncEngine API

```rust
pub struct SyncEngine {
    repo: git2::Repository,
}

impl SyncEngine {
    pub fn init(workspace_path: &Path) -> Result<Self>;  // git init + .gitignore
    pub fn open(workspace_path: &Path) -> Result<Self>;   // open existing repo
    pub fn sync(&self) -> Result<SyncResult>;              // pull → add → commit → push
    pub fn has_remote(&self) -> bool;
    pub fn prune_history(&self, keep: usize) -> Result<()>;
}
```

### Sync Flow

1. `git add -A` synced paths (content/, data/, graph/, config/)
2. Check for changes (staged diff)
3. If changes: `git commit` with timestamp message
4. If remote configured: `git pull --rebase` then `git push`
5. Periodically: prune to ~20 commits

### Key Modules

- `engine.rs` — SyncEngine struct, init/open/sync/prune
- `lib.rs` — Re-exports

### Dependencies

- `git2` crate (libgit2 bindings)
- `clotho-store` (for workspace path resolution)

## Alternatives Considered

- **Shell-out to git** — Rejected: weird interactions with user environments, PATH issues, inconsistent behavior
- **Syncthing/Dropbox** — Doesn't handle binary SQLite files well
- **CRDTs** — Overkill for single-user
- **Full git history** — Unnecessary for sync; shallow keeps the repo small

## Implementation Plan

1. clotho-sync crate scaffold + git2 dependency
2. SyncEngine: init (git init + .gitignore), open
3. SyncEngine: sync (add → commit → push with pull-before-push)
4. SyncEngine: prune history
5. CLI: `clotho sync` command
6. MCP: `clotho_sync` tool
7. Integration tests with temp git repos