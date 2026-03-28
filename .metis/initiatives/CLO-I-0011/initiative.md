---
id: clotho-tui-persistent-agent-surface
level: initiative
title: "Clotho TUI - Persistent Agent Surface"
short_code: "CLO-I-0011"
created_at: 2026-03-26T17:07:08.477765+00:00
updated_at: 2026-03-26T19:10:48.592051+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: L
initiative_id: clotho-tui-persistent-agent-surface
---

# Clotho TUI - Persistent Agent Surface Initiative

## Context

Clotho has been used via Claude MCP for ~2 weeks and works well for authoring — creating entities, capturing notes, building graphs. But the chat interface is terrible for **reviewing persistent state**: todo lists, notes, artifacts. Chat is sequential and ephemeral; these things are spatial and persistent.

We need a **persistent surface** that both the user and the agent can interact with — the agent pushes content to it, the user views and edits it, the agent reads it back later.

## Goals & Non-Goals

**Goals:**
- Provide a TUI that renders persistent clotho entities for scanning/review
- Allow the agent to **push content** to the TUI (daily briefings, meeting notes, etc.)
- Support inline editing of pushed content (user marks up, agent reads back later)
- Persist TUI state (open tabs, pinned items) across restarts
- Share the same `clotho-store` as the MCP server — no sync protocol needed

**Non-Goals:**
- Replacing the Claude chat interface for authoring — chat stays primary for complex operations
- Web UI or desktop app (TUI only for now)
- Multi-user / remote access
- Real-time collaboration (near-real-time via store polling is sufficient)

## Architecture

### Overview

The TUI is a new crate (`clotho-tui`) that reads/writes the same SQLite store as `clotho-mcp` and `clotho-cli`. It uses `ratatui` + `crossterm` for rendering.

```
┌─────────────┐       ┌─────────────┐
│  Claude Chat │       │  Clotho TUI │
│  (via MCP)   │       │  (ratatui)  │
└──────┬───────┘       └──────┬──────┘
       │ write                │ read/write
       ▼                      ▼
    ┌─────────────────────────┐
    │     clotho-store        │
    │     (SQLite + JSONL)    │
    └─────────────────────────┘
```

### Two Types of TUI Content

1. **Surfaces** — Agent pushes a text blob (daily briefing, status update, quick summary). Stored in a dedicated `surfaces` table in clotho-store. User can edit inline, agent can read back on demand. Closing a tab marks the surface as closed but doesn't delete it — surfaces remain searchable and retrievable for history, memory, and retrieval ("what did I mark up in last week's briefing?"). Ephemeral in the UI sense (dismissable, not a domain entity), durable in the data sense.

2. **Entity views** — Agent or user opens an actual store entity (meeting note, task, person). TUI is a viewport into the store. Edits write back to the entity. Already persistent by nature — TUI just stores the entity ID reference.

### Data Layer

**Surfaces table** in clotho-store:
- `id`: UUID
- `title`: String (becomes tab name)
- `content`: Text (markdown, user-editable)
- `surface_type`: Optional hint (briefing, meeting-notes, checklist, freeform)
- `status`: active / closed
- `created_at`, `updated_at`: Timestamps

Agent reads/writes surfaces through the store via MCP tools (`push_surface`, `read_surface`, `list_surfaces`). TUI reads/writes surface content through the same store. One data layer, one access pattern, no file-watching race conditions.

### TUI State File

A thin local state file tracks display-only concerns:
- Which tabs are open (surface IDs + entity IDs) and their order
- Active tab index
- Scroll/cursor positions
- Navigator collapse/expand state

No content in the state file — that all lives in the store.

### Layout

Three-panel layout: navigator, content area, embedded chat terminal.

```
┌──────────────────────────────────────────────┐
│ Clotho TUI                                    │
│┌────────┐┌──────────────────────────────────┐│
││Navigator││ [Daily Briefing] [1:1 w/ Alex]   ││
││         ││                                   ││
││ ▸ Tasks ││ ## Daily Briefing - Mar 26        ││
││ ▸ Notes ││ ☐ Review PR #42                   ││
││ ▸ People││ ☑ Ship clotho 0.0.1               ││
││ ▸ Surfac││ ☐ Prep for Thursday demo          ││
││         │├──────────────────────────────────┤│
││         ││ claude> push me a daily briefing   ││
││         ││ Done — pushed "Daily Briefing"     ││
││         ││ to your TUI.                       ││
││         ││                                   ││
││         ││ claude> _                          ││
│└────────┘└──────────────────────────────────┘│
│ clotho v0.0.1 | 12 entities | connected       │
└──────────────────────────────────────────────┘
```

- **Left panel**: Navigator — browse entities by type (Tasks, Notes, People, Surfaces). Open items in new tabs.
- **Top-right panel**: Tabbed content area. Each tab is either a store entity or an agent-pushed surface. Full inline editing.
- **Bottom-right panel**: Embedded terminal running `claude`, `claude -c`, or `claude -r` in the workspace directory. This is a real PTY — full Claude Code behavior, not a reimplemented chat client. Claude has the MCP server configured so surface pushes and entity operations appear live in the TUI.
- **Status bar**: Version, entity count, connection status.

### Embedded Terminal

The chat panel is a PTY (pseudo-terminal) embedded in the TUI via a terminal emulator widget. On launch, `clotho tui` spawns `claude` (or `claude -c` / `claude -r` based on user preference or flags) in the workspace directory. The TUI frames it alongside persistent content — no window switching needed.

The full interaction loop stays in one window:
1. Chat with Claude in the bottom panel
2. Claude pushes a surface → tab appears in top panel
3. Switch focus to tab, mark things up
4. Switch back to chat, tell Claude "I updated the briefing"
5. Claude reads it back via `read_surface` MCP tool

### Editing Model

Full inline editing. The TUI renders content as editable text. Changes write back to the store on save (explicit save, not auto-save on every keystroke).

## Detailed Design

### New Crate: `clotho-tui`

Dependencies:
- `ratatui` + `crossterm` for terminal rendering
- `clotho-store` for data access (shared with CLI/MCP)
- `serde` + `serde_json` for TUI state persistence

### New MCP Tools

- **`push_surface`** — Create or replace a surface. Params: `title`, `content`, `surface_type`, `replace` (if true, replaces existing active surface with same title).
- **`read_surface`** — Read a surface by ID or title. Returns content including user edits.
- **`list_surfaces`** — List surfaces, filterable by status (active/closed) and type. Supports search across title and content.

## Alternatives Considered

1. **Web UI (local axum server + SPA)**: Rich rendering but introduces a JS/TS frontend stack. More maintenance burden. Could be a future evolution.
2. **Tauri desktop app**: Best UX ceiling but biggest scope — packaging, distribution, two build systems. Overkill for current needs.
3. **Read-only TUI + chat for writes**: Simpler but defeats the purpose — inline editing of pushed surfaces is core to the value prop (marking up briefings, checking off todos).
4. **Standalone TUI with its own store**: Would require a sync protocol between TUI and MCP. Sharing the store directly is simpler and more reliable.

## Implementation Plan

### Milestone 1: App Shell + Embedded Terminal
- New `clotho-tui` crate with ratatui + crossterm
- Three-panel layout: navigator, content area, chat terminal
- Embedded PTY running `claude` / `claude -c` / `claude -r` in workspace directory
- Focus switching between panels (keybinding to toggle chat ↔ content)
- Status bar
- This milestone delivers a usable tool immediately: Claude chat framed with a navigator

### Milestone 2: Navigator + Entity Views
- Poll store on interval, render entity list in navigator grouped by type
- Tabbed content panel — open entities from navigator
- Render entity content (read-only)
- TUI state file — persist open tabs, active tab across restarts

### Milestone 3: Surfaces Data Layer + Agent Push
- Surfaces table + migration in clotho-store
- Surface CRUD operations in clotho-store
- `push_surface`, `read_surface`, `list_surfaces` MCP tools
- TUI polls for new active surfaces, opens tabs automatically
- Surfaces appear in navigator under their own section

### Milestone 4: Inline Editing
- Editable text in content panel (surfaces and entities)
- Explicit save to store
- Checkbox toggling for todo-style items
- Agent reads back user-edited content via MCP

### Milestone 5: Polish
- Keyboard shortcuts / help overlay
- Search within navigator
- Surface templates (briefing, checklist, freeform)
- Notifications for new agent pushes (subtle tab highlight)
- Resizable panels