---
id: graph-layer-clotho-graph
level: initiative
title: "Graph Layer (clotho-graph)"
short_code: "CLO-I-0002"
created_at: 2026-03-16T13:23:16.411856+00:00
updated_at: 2026-03-17T01:30:31.656319+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: graph-layer-clotho-graph
---

# Graph Layer (merged into clotho-core)

## Context

The graph layer lives inside clotho-core (not a separate crate) because the graph is a core operational concept. It wraps graphqlite (colliery-io/graphqlite, on crates.io) to provide the typed relation graph. Relations between entities are first-class citizens in Clotho — the graph of connections is as important as the content itself.

This replaces the placeholder Graph/Relation/RelationType types from CLO-I-0001 with real graphqlite-backed implementations.

## Goals & Non-Goals

**Goals:**
- Add graphqlite dependency to clotho-core
- Replace placeholder Graph struct with real graphqlite-backed GraphStore
- Define graph schema mapping all 15 entity types to graph nodes
- Implement all 12 typed relations as edges (9 semantic + 3 temporal)
- Provide CRUD API for nodes and edges
- Provide query helpers for common Cypher patterns
- Wire up the Relatable trait to use real graph queries

**Non-Goals:**
- Full-text search (that's clotho-store's FTS5 index)
- Entity persistence (that's clotho-store)
- Exposing raw Cypher to end users (that's clotho-cli/clotho-mcp)
- Temporal edge materialization (that's clotho-store's sync layer, decided during CLO-I-0001)

## Detailed Design

### Module Structure

```
clotho-core/src/
    graph/
        mod.rs         # GraphStore struct, init, open
        nodes.rs       # Node CRUD (register, remove, get)
        edges.rs       # Edge CRUD (add, remove, query by source/target/type)
        queries.rs     # Common query helpers (traversals, pattern matching)
```

### GraphStore

```rust
pub struct GraphStore {
    // graphqlite connection handle
}

impl GraphStore {
    pub fn open(path: &Path) -> Result<Self, GraphError>;
    pub fn in_memory() -> Result<Self, GraphError>;  // for tests
}
```

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
| `HAS_CADENCE` | Program, Responsibility, Workstream, Task | (edge props) | Temporal: recurring schedule |
| `HAS_DEADLINE` | Objective, Task, Artifact, Risk, Blocker, Question | (edge props) | Temporal: hard due date |
| `HAS_SCHEDULE` | Task, Meeting | (edge props) | Temporal: specific date/time |

### Common Query Patterns

- What decisions came from a given meeting/program?
- What's blocking a specific program's tasks?
- What artifacts deliver against an objective?
- What entities does a person appear in?
- What tags co-occur frequently?

### Dependencies

- `graphqlite` (crates.io) — SQLite-backed graph with Cypher support
- clotho-core types, traits (same crate)

## Alternatives Considered

- **Separate clotho-graph crate** — Rejected: graph is a core concept, the separation was only code organization. Merging avoids circular dependency issues with Relatable trait.
- **Standalone Neo4j** — Too heavy for a single-user local tool
- **In-memory graph** — Doesn't persist across sessions; graphqlite gives SQLite durability with Cypher ergonomics
- **Adjacency list in JSONL** — Loses query expressiveness; Cypher is a major ergonomic win

## Implementation Plan

1. Add graphqlite dependency to clotho-core
2. Create graph/ module with GraphStore (open, in_memory)
3. Implement node registration (register entity as node, remove, get)
4. Implement edge CRUD (add_edge, remove_edge, query by source/target/type)
5. Replace placeholder Graph type in traits.rs — Relatable now takes &GraphStore
6. Build common query helpers (by entity, by relation type, traversals)
7. Integration tests with in_memory GraphStore