---
id: visible-content-layout-inbox
level: initiative
title: "Visible Content Layout + Inbox"
short_code: "CLO-I-0009"
created_at: 2026-03-17T15:16:32.124129+00:00
updated_at: 2026-03-17T15:16:32.124129+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
initiative_id: visible-content-layout-inbox
---

# Visible Content Layout + Inbox

## Context

Per CLO-A-0004: all content currently lives inside `.clotho/content/` (hidden). This violates the "browsable in any editor" principle and makes structural entities (programs, responsibilities) invisible. The cadence-driven ceremony model (CLO-I-0010) requires an inbox for external integrations to deposit materials.

## Goals & Non-Goals

**Goals:**
- Move all content directories to project root (visible): programs/, responsibilities/, objectives/, workstreams/, tasks/, meetings/, reflections/, artifacts/, notes/, people/
- Refactor `ContentStore` to resolve paths relative to project root instead of `.clotho/content/`
- Refactor `Workspace::init` to create visible dirs at root + `.clotho/` for machine data
- Add `.clotho/inbox/` as a landing zone for incoming data
- Update `SyncEngine` gitignore to cover new layout (sync visible dirs + .clotho/data/ + .clotho/graph/ + .clotho/config/, ignore .clotho/index/)
- Update all CLI commands, MCP tools, and tests
- Update plugin session-start hook to reference new layout

**Non-Goals:**
- Inbox processing logic (that's CLO-I-0010 ceremonies)
- Integration connectors (Otter.ai, calendar sync, etc.)
- Content frontmatter/metadata in markdown files (future enhancement)

## Detailed Design

### Workspace::init creates

At project root:
- programs/, responsibilities/, objectives/, workstreams/, tasks/
- meetings/, reflections/, artifacts/, notes/, people/

At .clotho/:
- data/, graph/, index/, inbox/, config/
- config/config.toml, config/ontology.toml

### ContentStore path resolution

Current: `workspace_path.join("content").join(subdir).join(id.md)`
New: `project_root.join(subdir).join(id.md)`

Where `project_root` = `workspace_path.parent()` (parent of `.clotho/`)

### Git sync paths

Synced: programs/, responsibilities/, objectives/, workstreams/, tasks/, meetings/, reflections/, artifacts/, notes/, people/, .clotho/data/, .clotho/graph/, .clotho/config/
Gitignored: .clotho/index/, .clotho/inbox/

### Inbox

- `.clotho/inbox/` created on init
- Files deposited by external integrations
- Processed by ceremonies → content moved to visible dirs + entities created
- Gitignored (transient staging area, not source of truth)

## Alternatives Considered

- **Keep everything in .clotho/** — violates browsability principle
- **Symlinks** — fragile, platform-dependent

## Implementation Plan

1. Refactor Workspace::init — create visible dirs at root, .clotho/ for machine data
2. Refactor ContentStore — path resolution relative to project root
3. Add inbox directory to init
4. Update SyncEngine .gitignore
5. Update all CLI commands (path references)
6. Update all MCP tools (path references)
7. Update plugin session-start hook
8. Update all tests
9. Update E2E test suite