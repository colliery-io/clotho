---
id: entity-lifecycle-management-search
level: initiative
title: "Entity lifecycle management — search, consolidation, and retirement"
short_code: "CLO-I-0012"
created_at: 2026-03-28T12:30:44.876556+00:00
updated_at: 2026-04-01T00:22:25.887150+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: entity-lifecycle-management-search
---

# Entity lifecycle management — search, consolidation, and retirement Initiative

## Context

After ~3 weeks of use, the workspace has accumulated 1000s of entities — notes, tasks, derived items from transcript extraction, etc. The TUI navigator shows a flat list per entity type which becomes unusable at this scale. There's no way to search/filter within the TUI, no mechanism to consolidate duplicate or related notes, and no lifecycle management to retire stale entities.

## Problem Statement

1. **Discovery**: Finding a specific entity in the TUI requires scrolling through hundreds of items. No search, no filter, no fuzzy matching.
2. **Accumulation**: Daily debriefs generate many fine-grained entities (decisions, risks, questions, insights) that pile up without consolidation. Similar items from different days remain separate.
3. **Staleness**: Old tasks in `todo` for weeks, risks that were resolved, questions that were answered — nothing gets cleaned up unless manually done.

## Goals

- **TUI search/filter**: Type-to-filter in the navigator, fuzzy matching, search across title and content
- **Consolidation agent**: An agent (or skill) that periodically reviews entities, merges duplicates, summarizes clusters of related notes into higher-level summaries
- **Retirement workflow**: Mark entities as archived/retired. Hide from default views but keep in the database for history. Agent can suggest candidates for retirement based on age and staleness.
- **Status-aware filtering**: Show active/blocked/stale items prominently, push resolved items down

## Non-Goals

- Automatic deletion of entities (always soft-archive, never delete)
- Complex query UI (Cypher queries via Claude are sufficient for ad-hoc exploration)

## Detailed Design

### TUI Search/Filter
- `/` key in navigator enters search mode — type to filter entities across all groups
- Fuzzy matching on title
- `Esc` exits search, restores full list
- Could also add a `clotho_search` result view as a tab

### Consolidation Agent
- New agent `entity-consolidator` that:
  - Groups entities by semantic similarity (title + content)
  - Suggests merges for near-duplicates
  - Summarizes clusters of related notes into a single summary note
  - Links the summary to the originals via `spawned_from` relations
- Could run as part of weekly review or on-demand

### Retirement
- Add `archived` status to entities
- `clotho_archive_entity` MCP tool
- Navigator hides archived entities by default
- Agent suggests retirement candidates: tasks in `done` for >2 weeks, risks/blockers that are resolved, questions that are answered

## Implementation Plan

### Milestone 0: Active-only default view
- Navigator filters to active entities by default (exclude archived/retired/deprecated)
- Quick win that immediately reduces noise without any new UI
- Requires the entity status field to be consistently set
- `/` search mode in navigator
- Fuzzy match on entity titles
- Filter results in real-time as user types

### Milestone 2: Retirement
- `archived` status on entities
- Archive MCP tool
- Navigator filters out archived by default
- Toggle to show/hide archived

### Milestone 3: Consolidation Agent
- Similarity detection across entities
- Merge/summarize workflow
- Integration with weekly review ceremony