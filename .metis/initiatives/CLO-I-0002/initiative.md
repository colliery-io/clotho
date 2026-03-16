---
id: graph-layer-clotho-graph
level: initiative
title: "Graph Layer (clotho-graph)"
short_code: "CLO-I-0002"
created_at: 2026-03-16T13:23:16.411856+00:00
updated_at: 2026-03-16T13:23:16.411856+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
initiative_id: graph-layer-clotho-graph
---

# Graph Layer (clotho-graph)

## Context

The `clotho-graph` crate wraps graphqlite (colliery-io/graphqlite) to provide the typed relation graph. Relations between entities are first-class citizens in Clotho — the graph of connections is as important as the content itself. This crate defines the schema, provides common queries, and bridges between clotho-core entity types and graph nodes/edges.

## Goals & Non-Goals

**Goals:**
- Define graph schema mapping all clotho-core entities to graph nodes
- Implement all typed relations: BELONGS_TO, RELATES_TO, DELIVERS, SPAWNED_FROM, EXTRACTED_FROM, HAS_DECISION, HAS_RISK, BLOCKED_BY, MENTIONS
- Provide a query API for common Cypher patterns
- Support freeform tag co-occurrence queries
- Store relations in `graph/relations.db` (graphqlite SQLite backend)

**Non-Goals:**
- Full-text search (that's clotho-store's FTS5 index)
- Entity persistence (that's clotho-store)
- Exposing raw Cypher to end users (that's clotho-cli/clotho-mcp)

## Detailed Design

### Typed Relations

| Relation | From | To | Semantics |
|----------|------|-----|-----------|
| `BELONGS_TO` | Task, Objective, Note, etc. | Program, Responsibility | Ownership |
| `RELATES_TO` | Any | Workstream | Topical connection |
| `DELIVERS` | Artifact | Task, Objective | Evidence of completion |
| `SPAWNED_FROM` | Note, Task, Extraction | Meeting | Origin tracking |
| `EXTRACTED_FROM` | Derived entity | Transcript | Source span |
| `HAS_DECISION` | Meeting, Transcript | Decision | Contains |
| `HAS_RISK` | Any | Risk | Flags risk |
| `BLOCKED_BY` | Task | Blocker | Impediment |
| `MENTIONS` | Transcript, Note | Person, Program, etc. | Reference |

### Common Query Patterns

- What decisions came from a given meeting/program?
- What's blocking a specific program's tasks?
- What artifacts deliver against an objective?
- What entities does a person appear in?
- What tags co-occur frequently?

### Dependencies

- `graphqlite` (colliery-io/graphqlite) — SQLite-backed graph with Cypher support
- `clotho-core` — Entity types, EntityId, Relatable trait

## Alternatives Considered

- **Standalone Neo4j** — Too heavy for a single-user local tool
- **In-memory graph** — Doesn't persist across sessions; graphqlite gives SQLite durability with Cypher ergonomics
- **Adjacency list in JSONL** — Loses query expressiveness; Cypher is a major ergonomic win

## Implementation Plan

1. Set up graphqlite dependency and database initialization
2. Define node schema mapping from clotho-core entity types
3. Define edge types for all typed relations
4. Implement CRUD for relations (add, remove, query)
5. Build common query helpers (by entity, by relation type, traversals)
6. Integration tests with in-memory graphqlite instances