---
id: domain-model-entity-design
level: specification
title: "Domain Model & Entity Design"
short_code: "CLO-S-0002"
created_at: 2026-03-16T13:30:37.889970+00:00
updated_at: 2026-03-16T13:30:37.889970+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#specification"
  - "#phase/discovery"


exit_criteria_met: false
initiative_id: NULL
---

# Domain Model & Entity Design

## Overview

This specification defines the four-layer domain model, all 16 entity types, the 7 core traits, shared types, and lifecycle state machines. This is the canonical reference for `clotho-core` and the foundation that all other crates build upon.

## Layer Architecture

Entities are organized into four conceptual layers, each representing a different level of abstraction in the user's work:

```
┌─────────────────────────────────────────────────────────────┐
│                    Structural Layer                         │
│  "What you do"                                              │
│  Responsibility, Program, Objective                         │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    Execution Layer                          │
│  "Work in motion"                                           │
│  Workstream, Task, Cadence                                  │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                     Capture Layer                           │
│  "Raw material"                                             │
│  Meeting, Transcript, Note, Reflection, Artifact            │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                     Derived Layer                           │
│  "Sense-making"                                             │
│  Decision, Risk, Blocker, Question, Insight                 │
└─────────────────────────────────────────────────────────────┘
```

## Entity Definitions

### Structural Layer — "What you do"

**Responsibility**
- Ongoing role obligations that never "complete"
- Examples: team mentorship, HR reporting, budget management, 1:1s
- Lifecycle: `active | inactive`

**Program**
- Strategic initiatives with explicit objectives
- Examples: technical education, PMO establishment, monolith breakup
- Lifecycle: `active | inactive`

**Objective**
- Outcomes within a program
- Belongs to exactly one Program
- Lifecycle: `active | inactive`

### Execution Layer — "Work in motion"

**Workstream**
- Long-running work threads
- May relate to Programs or Responsibilities
- Lifecycle: `active | inactive`

**Task**
- Discrete work items
- Lifecycle: `todo → doing → blocked → done`
- Can be spawned from meetings, objectives, or created directly

### Capture Layer — "Raw material"

**Meeting**
- Container entity for a meeting occurrence
- Has associated Transcript and/or Notes
- Carries metadata: date, attendees, related entities

**Transcript**
- Raw meeting content (typically from transcription service)
- Source for AI extraction
- Belongs to exactly one Meeting

**Note**
- Authored content, freeform
- Can belong to a Meeting or stand alone
- May relate to any entity via tags/relations

**Reflection**
- Time-period bound thinking
- Period types: `daily | weekly | monthly | quarterly | adhoc`
- Carries: `period_start`, `period_end`, optional `period_name`
- May relate to Programs for scoped reflection

**Artifact**
- Deliverable with external source link
- Examples: design docs, PRs, presentations, shipped features
- Ingested as markdown with link to original
- Delivers against Tasks or Objectives

### Derived Layer — "Sense-making"

All derived entities follow the extraction lifecycle: `draft → promoted | discarded`

- **Decision** — Recorded decision point
- **Risk** — Identified risk
- **Blocker** — Impediment to progress
- **Question** — Open question requiring resolution
- **Insight** — Learning or observation worth preserving

### Person (cross-cutting)

- Lightweight rolodex entry for people mentioned in transcripts and notes
- Used for entity resolution during extraction
- Carries: name (required), email (optional, for fuzzy matching), notes (optional freeform text)
- Implements ContentBearing (notes stored as markdown)

## Temporal Traits (replaces Cadence entity)

Cadence is not a standalone entity — it's a temporal scheduling concern that attaches to entities via traits. Three distinct concepts:

```rust
pub struct Cadence {
    pub frequency: Frequency,
    pub cron: Option<String>,
    pub label: Option<String>,
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

Temporal data is stored on entity structs AND materialized as graph edges (HAS_CADENCE, HAS_DEADLINE, HAS_SCHEDULE) to enable cross-entity temporal queries.

## Core Traits

```rust
/// Core identity for all entities
pub trait Entity {
    fn id(&self) -> &EntityId;
    fn entity_type(&self) -> EntityType;
    fn title(&self) -> &str;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
}

/// Entities with active/inactive lifecycle
pub trait Activatable: Entity {
    fn status(&self) -> Status; // Active | Inactive
    fn set_status(&mut self, status: Status);
}

/// Entities with task-like workflow
pub trait Taskable: Entity {
    fn state(&self) -> TaskState; // Todo | Doing | Blocked | Done
    fn transition(&mut self, to: TaskState) -> Result<(), TransitionError>;
    fn valid_transitions(&self) -> Vec<TaskState>;
}

/// Entities from AI extraction (draft lifecycle)
pub trait Extractable: Entity {
    fn extraction_status(&self) -> ExtractionStatus; // Draft | Promoted | Discarded
    fn source_span(&self) -> Option<&SourceSpan>;
    fn confidence(&self) -> f32;
    fn promote(&mut self) -> Result<(), PromotionError>;
    fn discard(&mut self);
}

/// Entities that participate in the relation graph
pub trait Relatable: Entity {
    fn relations(&self, graph: &Graph) -> Vec<Relation>;
    fn graph_label(&self) -> &'static str;
}

/// Entities with freeform tags
pub trait Taggable: Entity {
    fn tags(&self) -> &[Tag];
    fn add_tag(&mut self, tag: Tag);
    fn remove_tag(&mut self, tag: &str);
}

/// Entities with markdown content
pub trait ContentBearing: Entity {
    fn content(&self) -> &str;
    fn set_content(&mut self, content: String);
    fn content_path(&self) -> PathBuf;
}
```

## Trait Composition Matrix

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

## Shared Types

- `EntityId` — Unique identifier for all entities
- `EntityType` — Enum of all 15 entity types
- `Status` — `Active | Inactive` (for Activatable entities)
- `TaskState` — `Todo | Doing | Blocked | Done` (for Taskable entities)
- `ExtractionStatus` — `Draft | Promoted | Discarded` (for Extractable entities)
- `SourceSpan` — Reference to a location in a transcript (for extraction provenance)
- `Tag` — Freeform string tag
- `PeriodType` — `Daily | Weekly | Monthly | Quarterly | Adhoc` (for Reflections)

## Lifecycle State Machines

### Activatable Lifecycle
```
active ⟷ inactive
```

### Task Lifecycle
```
todo → doing → done
       doing → blocked → doing
```
Valid transitions from each state:
- `todo`: → doing
- `doing`: → blocked, → done
- `blocked`: → doing
- `done`: (terminal)

### Extraction Lifecycle
```
draft → promoted
draft → discarded
```
Both promoted and discarded are terminal states.

## Open Questions

1. ~~**Cadence implementation**~~ — Resolved: temporal traits (HasCadence, HasDeadline, HasSchedule) with hybrid storage (entity structs + materialized graph edges). See CLO-I-0001.
2. ~~**Person entity richness**~~ — Resolved: lightweight rolodex (name, email, notes). Person gets ContentBearing for notes.
3. **Artifact content** — Store full markdown or just metadata + link?