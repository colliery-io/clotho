---
id: git-sync-layer-clotho-sync
level: initiative
title: "Git Sync Layer (clotho-sync)"
short_code: "CLO-I-0007"
created_at: 2026-03-16T13:23:16.478907+00:00
updated_at: 2026-03-16T13:23:16.478907+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: S
initiative_id: git-sync-layer-clotho-sync
---

# Git Sync Layer (clotho-sync)

## Context

The `clotho-sync` crate implements git as a dumb sync layer for replicating the workspace across devices. This is explicitly not version control — it's silent, automatic replication with shallow history. Single-user assumption means conflicts are rare (same user, different device).

## Goals & Non-Goals

**Goals:**
- Auto-commit on save with debounce (every 30s of inactivity)
- Auto-push after each commit (silent)
- Pull-before-push to handle multi-device scenarios
- Shallow history pruning (~20 commits)
- Main branch only, no branching

**Non-Goals:**
- Version control semantics (branching, tagging, meaningful commit messages)
- Multi-user collaboration or conflict resolution
- Merge conflict UI (single-user assumption)

## Detailed Design

### Sync Model

- **Synced:** `content/`, `data/`, `graph/`, `config/`
- **Gitignored:** `index/` (rebuilt on clone from synced data)

### Sync Flow

1. Detect changes (filesystem watcher or post-write hook)
2. Debounce — wait 30s of inactivity
3. `git pull --rebase` (handle rare conflicts with last-write-wins)
4. `git add -A` synced directories
5. `git commit` with timestamp message
6. `git push`
7. Prune to ~20 commits periodically

### Key Modules

- `commit.rs` — Debounced auto-commit logic
- `push.rs` — Pull-before-push, silent push, history pruning

### Open Questions

- Graph sync strategy: commit SQLite (graphqlite) directly or export/import a deterministic format?

## Alternatives Considered

- **Syncthing/Dropbox** — Doesn't handle binary SQLite files well; git at least gives line-based diffs for JSONL/markdown
- **CRDTs** — Overkill for single-user; git's simplicity wins
- **Full git history** — Unnecessary for sync; shallow keeps the repo small

## Implementation Plan

1. Implement auto-commit with debounce timer
2. Implement pull-before-push logic
3. Implement history pruning
4. Set up .gitignore for index/
5. Integration tests with temp git repos