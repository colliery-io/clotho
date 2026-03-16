---
id: core-domain-model-trait-system
level: initiative
title: "Core Domain Model & Trait System (clotho-core)"
short_code: "CLO-I-0001"
created_at: 2026-03-16T13:23:16.394491+00:00
updated_at: 2026-03-16T20:44:34.684033+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
initiative_id: core-domain-model-trait-system
---

# Core Domain Model & Trait System (clotho-core)

## Context

The `clotho-core` crate is the foundation of the entire system. It defines the four-layer domain model (Structural, Execution, Capture, Derived), all entity types, the core trait system, and shared types. Every other crate depends on this one. clotho-core may also include clotho-graph (graph is a core operational concept, not just an integration).

## Goals & Non-Goals

**Goals:**
- Define all 15 entity types across the four domain layers (Cadence removed as entity — see temporal traits below)
- Implement the entity trait system: Entity, Activatable, Taskable, Extractable, Relatable, Taggable, ContentBearing
- Implement temporal traits: HasCadence, HasDeadline, HasSchedule (hybrid: stored on entities + materialized as graph edges)
- Implement LLM backend traits: Extractor, Summarizer, Resolver, Embedder
- Define shared types: EntityId, Status, TaskState, ExtractionStatus, SourceSpan, Tag, EntityType, Cadence, Frequency
- Establish lifecycle state machines (active/inactive, todo/doing/blocked/done, draft/promoted/discarded)
- Ensure trait composition matches the design matrix
- Include serde Serialize/Deserialize derives on all types for integration points

**Non-Goals:**
- Persistence layer implementation (that's clotho-store)
- AI extraction logic (that's clotho-extract)
- LLM backend implementations (that's clotho-extract/backends)

## Architecture

### Entity Layers

**Structural Layer** — "What you do"
- `Responsibility` — Ongoing role obligations (active/inactive lifecycle)
- `Program` — Strategic initiatives with objectives (active/inactive)
- `Objective` — Outcomes within a Program (active/inactive, belongs to one Program)

**Execution Layer** — "Work in motion"
- `Workstream` — Long-running work threads (active/inactive)
- `Task` — Discrete work items (todo → doing → blocked → done)

**Capture Layer** — "Raw material"
- `Meeting` — Container for transcript + notes, carries date/attendees/relations
- `Transcript` — Raw meeting content, belongs to one Meeting
- `Note` — Freeform authored content, standalone or meeting-attached
- `Reflection` — Time-period bound (daily/weekly/monthly/quarterly/adhoc)
- `Artifact` — Deliverable with external source link

**Derived Layer** — "Sense-making" (all follow draft → promoted | discarded)
- `Decision`, `Risk`, `Blocker`, `Question`, `Insight`

**Cross-cutting**
- `Person` — Lightweight rolodex entry (name, email, notes)

### Temporal Traits (replaces Cadence entity)

Cadence is not a standalone entity — it's a temporal scheduling concern that attaches to entities via traits. Three distinct temporal concepts, implemented as separate traits + materialized graph edges for queryability:

```rust
/// Recurring schedule (e.g., "weekly sync every Monday")
pub struct Cadence {
    pub frequency: Frequency,
    pub cron: Option<String>,       // For custom schedules
    pub label: Option<String>,      // "weekly sync", "quarterly review"
    pub next_occurrence: Option<DateTime<Utc>>,
}

pub enum Frequency {
    Daily, Weekly, Biweekly, Monthly, Quarterly, Yearly, Custom,
}

pub trait HasCadence: Entity {
    fn cadence(&self) -> Option<&Cadence>;
    fn set_cadence(&mut self, cadence: Option<Cadence>);
}

pub trait HasDeadline: Entity {
    fn deadline(&self) -> Option<DateTime<Utc>>;
    fn set_deadline(&mut self, deadline: Option<DateTime<Utc>>);
}

pub trait HasSchedule: Entity {
    fn scheduled_at(&self) -> Option<DateTime<Utc>>;
    fn set_scheduled_at(&mut self, at: Option<DateTime<Utc>>);
}
```

Graph materialization: temporal data is stored on entity structs AND materialized as graph edges (HAS_CADENCE, HAS_DEADLINE, HAS_SCHEDULE) to enable cross-entity temporal queries like "what's due this week?" or "what recurring items fire on Mondays?"

### Trait Composition Matrix

| Entity | Entity | Activatable | Taskable | Extractable | Relatable | Taggable | ContentBearing | HasCadence | HasDeadline | HasSchedule |
|--------|--------|-------------|----------|-------------|-----------|----------|----------------|------------|-------------|-------------|
| Program | Y | Y | | | Y | Y | Y | Y | | |
| Responsibility | Y | Y | | | Y | Y | Y | Y | | |
| Objective | Y | Y | | | Y | Y | Y | | Y | |
| Workstream | Y | Y | | | Y | Y | Y | Y | | |
| Task | Y | | Y | | Y | Y | Y | Y | Y | Y |
| Meeting | Y | | | | Y | Y | Y | | | Y |
| Transcript | Y | | | | Y | Y | Y | | | |
| Note | Y | | | | Y | Y | Y | | | |
| Reflection | Y | | | | Y | Y | Y | | | |
| Artifact | Y | | | | Y | Y | Y | | Y | |
| Decision | Y | | | Y | Y | Y | | | | |
| Risk | Y | | | Y | Y | Y | | | Y | |
| Blocker | Y | | | Y | Y | Y | | | Y | |
| Question | Y | | | Y | Y | Y | | | Y | |
| Insight | Y | | | Y | Y | Y | | | | |
| Person | Y | | | | Y | Y | Y | | | |

## Detailed Design

### Entity Traits

- `Entity` — id, entity_type, title, created_at, updated_at
- `Activatable: Entity` — status (Active/Inactive), set_status
- `Taskable: Entity` — state (Todo/Doing/Blocked/Done), transition, valid_transitions
- `Extractable: Entity` — extraction_status (Draft/Promoted/Discarded), source_span, confidence, promote, discard
- `Relatable: Entity` — relations(graph), graph_label (clotho-core depends on graph — graph is a core concept)
- `Taggable: Entity` — tags, add_tag, remove_tag
- `ContentBearing: Entity` — content, set_content, content_path

### Temporal Traits

- `HasCadence: Entity` — cadence (recurring schedule with Frequency + optional cron)
- `HasDeadline: Entity` — deadline (hard due date)
- `HasSchedule: Entity` — scheduled_at (specific date/time)

### LLM Backend Traits (per CLO-A-0001, CLO-A-0003)

- `Extractor` — async extract(ExtractionRequest) -> ExtractionResult
- `Summarizer` — async summarize(SummaryRequest) -> SummaryResult
- `Resolver` — async resolve(ResolutionRequest) -> ResolutionResult
- `Embedder` — async embed(texts) -> Vec<Vec<f32>>, dimension() -> usize

These traits are defined in clotho-core so that clotho-extract can depend on abstractions, not concrete LLM implementations. Request/response types and errors live in `domain/llm_types.rs`.

### Person Entity

Lightweight rolodex: name (required), email (optional, used for fuzzy matching during extraction), notes (optional freeform text). Person implements Entity + Relatable + Taggable + ContentBearing (notes stored as markdown content).

### Serialization

All types derive serde Serialize/Deserialize for integration points (clotho-store persistence, MCP transport, CLI JSON output). This is not persistence logic — it's data representation that naturally belongs with the type definitions.

## Alternatives Considered

- **Single enum vs trait composition** — A single large Entity enum was considered but rejected in favor of traits for extensibility and type safety at compile time.
- **ECS pattern** — An entity-component-system approach was considered but adds unnecessary indirection for a single-user system with a fixed set of entity types.
- **Cadence as a first-class entity** — Rejected: cadence is a temporal concern that attaches to entities, not a standalone thing. Modeled as traits + graph edges instead.
- **Unified Temporal trait** — Rejected in favor of three separate traits (HasCadence, HasDeadline, HasSchedule) because not all entities need all three. Keeps composition clean.
- **Keeping clotho-graph separate from clotho-core** — Considered but graph is a core operational concept. The separation is code organization, not conceptual. Core depends on graph (Relatable takes &Graph).

## Implementation Plan

1. Define shared types module (EntityId, Status, TaskState, ExtractionStatus, Cadence, Frequency, etc.)
2. Define entity trait definitions (Entity, Activatable, Taskable, Extractable, Relatable, Taggable, ContentBearing)
3. Define temporal trait definitions (HasCadence, HasDeadline, HasSchedule)
4. Define LLM backend trait definitions (Extractor, Summarizer, Resolver, Embedder) + request/response types
5. Implement Structural layer entities (Program, Responsibility, Objective)
6. Implement Execution layer entities (Workstream, Task)
7. Implement Capture layer entities (Meeting, Transcript, Note, Reflection, Artifact)
8. Implement Derived layer entities (Decision, Risk, Blocker, Question, Insight)
9. Implement Person entity
10. Unit tests for all state transitions, trait implementations, and temporal scheduling