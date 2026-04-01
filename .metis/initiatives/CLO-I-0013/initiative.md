---
id: hierarchical-navigation-ontology
level: initiative
title: "Hierarchical navigation — ontology-driven nesting in TUI and content layout"
short_code: "CLO-I-0013"
created_at: 2026-03-28T12:30:45.927898+00:00
updated_at: 2026-04-01T00:17:56.575876+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: hierarchical-navigation-ontology
---

# Hierarchical navigation — ontology-driven nesting in TUI and content layout Initiative

## Context

The TUI navigator currently groups entities by type (Tasks, Notes, People, etc.) in a flat list. This ignores the rich relation graph that already exists — tasks belong to programs, objectives nest under programs, decisions are extracted from transcripts, people are mentioned in meetings. The flat view forces the user to know what type something is before they can find it, rather than navigating by context.

The `content/` directory structure mirrors this flat model — `content/tasks/`, `content/notes/`, etc. with no hierarchy. As the workspace grows, both the TUI and the filesystem become harder to navigate.

## Problem Statement

1. **Flat navigator loses context**: 200 tasks in a flat list with no grouping by program, workstream, or responsibility. The user can't see "what tasks belong to the Monolith Breakup program" without running a graph query.
2. **No relationship visibility**: The `belongs_to`, `relates_to`, `extracted_from` relations exist in the graph but aren't surfaced in navigation. Opening a Program entity doesn't show its child objectives, tasks, or related workstreams.
3. **Content directory doesn't reflect structure**: Files are organized by type, not by semantic grouping. A program's tasks, notes, and artifacts are scattered across different directories.

## Goals

- **Hierarchical tree in navigator**: Show entities nested by their relations — Programs > Objectives > Tasks, Responsibilities > related entities, Meetings > extracted decisions/risks
- **Relationship-aware entity view**: When viewing an entity, show its relations inline — children, parents, related items
- **Optional nested content layout**: Optionally organize `content/` to mirror the hierarchy (e.g., `content/programs/monolith-breakup/tasks/`)
- **Multiple navigation modes**: Toggle between flat-by-type (current) and hierarchical views

## Non-Goals

- Replacing the graph query system — hierarchy is a view, not a replacement for Cypher
- Enforcing strict hierarchy — entities can exist without parents (orphans are valid)
- Automatic reorganization of existing content files (migration should be opt-in)

## Detailed Design

### Navigator Hierarchy

The navigator builds a tree from the relation graph instead of flat type groups:

```
▾ Programs
  ▾ Monolith Breakup
    ▾ Objectives
      Extract user service
      Decouple payment flow
    ▾ Tasks (12)
      Write migration RFC
      Review API contracts
    ▾ Risks (3)
      Data consistency during migration
    ▸ Decisions (5)
  ▾ Technical Education
    ...
▾ Responsibilities
  ▾ Team Mentorship
    ▸ Related tasks (4)
  ▾ Budget Management
    ...
▾ People
  Alice
  Bob
▾ Unlinked (orphan entities)
  Random note
  Ad-hoc decision
```

Key relations driving hierarchy:
- `belongs_to` → parent/child nesting
- `extracted_from` → group derived entities under their source
- `mentions` → link people to their contexts
- `relates_to` → cross-reference in expanded views

### Entity Detail View

When viewing an entity in a tab, show a relations section:

```
# Monolith Breakup (Program)

Status: active
Created: 2026-03-15

## Relations
  Objectives: Extract user service, Decouple payment flow
  Tasks: 12 (3 doing, 2 blocked, 7 todo)
  Risks: 3 active
  Decisions: 5
```

### View Toggle

`v` key in navigator cycles between:
1. **Flat** (current) — grouped by entity type
2. **Hierarchy** — nested by relations
3. **Recent** — sorted by updated_at, no grouping

## Implementation Plan

### Milestone 1: Graph-aware navigator
- Query relations on refresh to build hierarchy tree
- Render nested tree with expand/collapse
- `v` key to toggle between flat and hierarchy views
- Handle orphan entities (no belongs_to) in an "Unlinked" section

### Milestone 2: Relation-aware entity view
- When opening an entity tab, query its relations
- Show relations section below entity content
- Clickable relation links (open related entity in new tab)

### Milestone 3: Nested content layout (optional)
- `clotho reorganize` CLI command to restructure content/ by hierarchy
- Update content_path in entities.db to reflect new locations
- Keep flat layout as default, nested as opt-in