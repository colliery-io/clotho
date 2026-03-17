---
id: entity-management-graph-relations
level: initiative
title: "Entity Management & Graph Relations (CLI + MCP)"
short_code: "CLO-I-0008"
created_at: 2026-03-17T12:53:15.911473+00:00
updated_at: 2026-03-17T12:53:15.911473+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
initiative_id: entity-management-graph-relations
---

# Entity Management & Graph Relations (CLI + MCP)

## Context

The current CLI and MCP server can only create Capture layer entities (Notes, Meetings, Transcripts, Artifacts, Reflections). There's no way to create the Structural or Execution layer entities that give captured content its meaning — Programs, Responsibilities, Objectives, Workstreams, Tasks, or People. There's also no way to create graph relations between entities, meaning content exists in isolation with no BELONGS_TO, RELATES_TO, BLOCKED_BY, etc.

Without these capabilities, Clotho is a content store but not a work management system.

## Goals & Non-Goals

**Goals:**
- `clotho create <type> --title "..." [--parent <id>] [--status active]` — Create any of the 15 entity types
- `clotho get <id>` — Read a single entity's metadata + content
- `clotho update <id> [--title] [--status] [--state]` — Update entity fields
- `clotho delete <id>` — Delete an entity from all backends
- `clotho relate <source_id> <relation_type> <target_id>` — Create typed graph edges
- `clotho unrelate <source_id> <relation_type> <target_id>` — Remove graph edges
- `clotho relations <id>` — Show all relations for an entity
- Matching MCP tools for all of the above (clotho_create_entity, clotho_get_entity, clotho_update_entity, clotho_delete_entity, clotho_create_relation, clotho_delete_relation, clotho_get_relations)

**Non-Goals:**
- Extraction pipeline (deferred to CLO-I-0004)
- Bulk import/export
- Entity type-specific validation rules (e.g., Objective must have a program_id) — keep it generic for v1

## Detailed Design

### CLI Commands

```bash
# Structural layer
clotho create program --title "Technical Education"
clotho create responsibility --title "Team Mentorship"
clotho create objective --title "Reduce deploy time" --parent <program_id>

# Execution layer
clotho create workstream --title "API Redesign"
clotho create task --title "Write RFC" --state todo
clotho create person --title "Alice" --email "alice@example.com"

# Read / Update / Delete
clotho get <id>
clotho update <id> --title "New Title" --status inactive
clotho delete <id>

# Relations
clotho relate <task_id> belongs_to <program_id>
clotho relate <artifact_id> delivers <objective_id>
clotho relate <transcript_id> mentions <person_id>
clotho unrelate <task_id> belongs_to <program_id>
clotho relations <program_id>
```

### MCP Tools

- `clotho_create_entity` — Create any entity type (type, title, optional parent/status/state/email)
- `clotho_get_entity` — Read entity metadata + content (replaces clotho_read_entity or supplements it)
- `clotho_update_entity` — Update entity fields
- `clotho_delete_entity` — Delete from all backends
- `clotho_create_relation` — Create typed graph edge
- `clotho_delete_relation` — Remove graph edge
- `clotho_get_relations` — List all relations for an entity

### Entity Type Mapping

All 15 entity types creatable:

| Type | Layer | Default Status/State | Has Content |
|------|-------|---------------------|-------------|
| Program | Structural | active | Yes |
| Responsibility | Structural | active | Yes |
| Objective | Structural | active | Yes |
| Workstream | Execution | active | Yes |
| Task | Execution | todo | Yes |
| Meeting | Capture | — | Yes |
| Transcript | Capture | — | Yes |
| Note | Capture | — | Yes |
| Reflection | Capture | — | Yes |
| Artifact | Capture | — | Yes |
| Decision | Derived | draft | No |
| Risk | Derived | draft | No |
| Blocker | Derived | draft | No |
| Question | Derived | draft | No |
| Insight | Derived | draft | No |
| Person | Cross-cutting | — | Yes |

### Relation Types Available

All 12 relation types from CLO-S-0005: belongs_to, relates_to, delivers, spawned_from, extracted_from, has_decision, has_risk, blocked_by, mentions, has_cadence, has_deadline, has_schedule

## Alternatives Considered

- **Type-specific CLI commands** (e.g., `clotho create-program`, `clotho create-task`) — Too many commands. A generic `create <type>` with type-specific optional flags is cleaner.
- **Implicit relation creation** (e.g., `--parent` auto-creates BELONGS_TO) — Good ergonomic addition but keep explicit `relate` as the primary mechanism for v1.

## Implementation Plan

1. CLI: `clotho create` command (generic entity creation for all 15 types)
2. CLI: `clotho get` / `clotho update` / `clotho delete` commands
3. CLI: `clotho relate` / `clotho unrelate` / `clotho relations` commands
4. MCP: clotho_create_entity, clotho_update_entity, clotho_delete_entity tools
5. MCP: clotho_create_relation, clotho_delete_relation, clotho_get_relations tools
6. Integration tests for both CLI and MCP