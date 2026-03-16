---
id: relation-model-graph-schema
level: specification
title: "Relation Model & Graph Schema"
short_code: "CLO-S-0005"
created_at: 2026-03-16T13:30:40.183173+00:00
updated_at: 2026-03-16T13:30:40.183173+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#specification"
  - "#phase/discovery"


exit_criteria_met: false
initiative_id: NULL
---

# Relation Model & Graph Schema

## Overview

This specification defines the typed relation system, the graphqlite schema, freeform tags, and example Cypher query patterns. Relations are first-class citizens in Clotho — the graph of connections between entities is as important as the entity content itself. This is the canonical reference for `clotho-graph`.

## Typed Relations

All relations are directed edges stored in graphqlite and queryable via Cypher.

| Relation | From | To | Semantics |
|----------|------|-----|-----------|
| `BELONGS_TO` | Task, Objective, Note, etc. | Program, Responsibility | Ownership/containment |
| `RELATES_TO` | Any | Workstream | Topical connection |
| `DELIVERS` | Artifact | Task, Objective | Evidence of completion |
| `SPAWNED_FROM` | Note, Task, Extraction | Meeting | Origin tracking |
| `EXTRACTED_FROM` | Derived entity | Transcript | Source provenance |
| `HAS_DECISION` | Meeting, Transcript | Decision | Contains decision |
| `HAS_RISK` | Any | Risk | Flags risk |
| `BLOCKED_BY` | Task | Blocker | Impediment |
| `MENTIONS` | Transcript, Note | Person, Program, etc. | Reference/mention |
| `HAS_CADENCE` | Program, Responsibility, Workstream, Task | (edge properties) | Materialized temporal: recurring schedule |
| `HAS_DEADLINE` | Objective, Task, Artifact, Risk, Blocker, Question | (edge properties) | Materialized temporal: hard due date |
| `HAS_SCHEDULE` | Task, Meeting | (edge properties) | Materialized temporal: specific date/time |

### Relation Semantics

**Ownership relations** (`BELONGS_TO`):
- An Objective belongs to exactly one Program
- Tasks, Notes can belong to Programs or Responsibilities
- Establishes the structural hierarchy

**Provenance relations** (`SPAWNED_FROM`, `EXTRACTED_FROM`):
- Track where entities originated
- Notes and Tasks can be spawned from Meetings
- All Derived entities are extracted from Transcripts with source spans

**Semantic relations** (`HAS_DECISION`, `HAS_RISK`, `BLOCKED_BY`):
- Connect entities to their derived sense-making outputs
- A Meeting HAS_DECISION when a decision was made in it
- A Task is BLOCKED_BY a Blocker

**Reference relations** (`MENTIONS`, `RELATES_TO`, `DELIVERS`):
- Looser connections between entities
- Transcripts MENTION People, Programs, etc.
- Artifacts DELIVER against Tasks or Objectives

## Graph Node Schema

Every entity in clotho-core maps to a graph node. The node label matches the entity type.

**Node labels**: `Program`, `Responsibility`, `Objective`, `Workstream`, `Task`, `Meeting`, `Transcript`, `Note`, `Reflection`, `Artifact`, `Decision`, `Risk`, `Blocker`, `Question`, `Insight`, `Person`

**Common node properties**:
- `id` — EntityId (unique identifier)
- `title` — Entity title
- `entity_type` — String label

Nodes are lightweight — they primarily serve as join points. Full entity data lives in `data/entities.jsonl` and `content/`.

## Freeform Tags

In addition to typed relations, entities support freeform tags for emergent patterns.

- Tags are strings stored in `data/tags.jsonl`
- Examples: `#urgent`, `#revisit`, `#stakeholder-concern`, `#tech-debt`
- Tags can be queried for co-occurrence patterns
- Frequently co-occurring tags may suggest promotion to typed relations

### Tag Co-occurrence Analysis

The graph can surface patterns like:
- "Entities tagged `#urgent` are frequently BLOCKED_BY the same Blocker"
- "Notes tagged `#stakeholder-concern` cluster around Program X"
- "Tasks tagged `#tech-debt` mostly BELONG_TO Workstream Y"

## Example Cypher Queries

### What decisions came from PMO meetings?
```cypher
MATCH (p:Program {title: 'PMO Establishment'})<-[:RELATES_TO]-(m:Meeting)-[:HAS_DECISION]->(d:Decision)
RETURN m.title, d.title
```

### What's blocking the monolith breakup?
```cypher
MATCH (p:Program {title: 'Monolith Breakup'})<-[:BELONGS_TO]-(t:Task)-[:BLOCKED_BY]->(b:Blocker)
RETURN t.title, b.title
```

### What artifacts deliver against an objective?
```cypher
MATCH (o:Objective {title: 'Reduce deploy time'})<-[:DELIVERS]-(a:Artifact)
RETURN a.title
```

### What entities does a person appear in?
```cypher
MATCH (p:Person {name: 'Alice'})<-[:MENTIONS]-(e)
RETURN e.entity_type, e.title
```

### What came out of a specific meeting?
```cypher
MATCH (m:Meeting {title: '2025-01-15 Standup'})<-[:SPAWNED_FROM]-(e)
RETURN e.entity_type, e.title
```

### Trace extraction provenance
```cypher
MATCH (d:Decision)-[:EXTRACTED_FROM]->(t:Transcript)-[:SPAWNED_FROM]->(m:Meeting)
RETURN d.title, t.title, m.title
```

## Requirements

### Functional Requirements

| ID | Requirement | Rationale |
|----|-------------|-----------|
| REQ-5.1 | Store all typed relations in graphqlite | Relation-first design |
| REQ-5.2 | Support Cypher queries across the full graph | Query expressiveness |
| REQ-5.3 | CRUD operations for all relation types | Graph must be mutable |
| REQ-5.4 | Store freeform tags in data/tags.jsonl | Emergent pattern support |
| REQ-5.5 | Support tag co-occurrence queries | Pattern discovery |

### Non-Functional Requirements

| ID | Requirement | Rationale |
|----|-------------|-----------|
| NFR-5.1 | Graph must be consistent with entity data in JSONL | Single source of truth per concern |
| NFR-5.2 | Relation operations must be atomic with entity writes | Data integrity |

## Constraints

### Technical Constraints
- graphqlite (colliery-io/graphqlite) as the graph backend
- SQLite-backed — stored in `graph/relations.db`
- Must support git sync (SQLite file committed directly)