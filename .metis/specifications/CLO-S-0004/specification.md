---
id: ai-extraction-pipeline
level: specification
title: "AI Extraction Pipeline"
short_code: "CLO-S-0004"
created_at: 2026-03-16T13:30:39.838177+00:00
updated_at: 2026-03-16T13:30:39.838177+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#specification"
  - "#phase/discovery"


exit_criteria_met: false
initiative_id: NULL
---

# AI Extraction Pipeline

## Overview

This specification defines the AI-powered extraction system that processes transcripts and notes to produce draft derived entities. It covers the speech act ontology, the extraction pipeline stages, entity resolution, the confidence model, and the human review lifecycle. This is the canonical reference for `clotho-extract`.

## System Context

### Actors
- **User**: Reviews draft extractions via CLI (`clotho review`) — promotes, edits, or discards
- **LLM Backend**: Performs speech act classification and entity extraction from transcript text. Backend is pluggable via traits (see CLO-A-0001).

### External Systems
- **LLM Providers (configurable)**: Extraction uses a trait-based backend abstraction (CLO-A-0001). Claude API is the default provider, but backends are swappable via `config/config.toml`. Different tasks (extraction, summarization, resolution) can use different backends/models.

### Boundaries
- **Inside scope**: Speech act classification, entity mention extraction, fuzzy entity resolution, draft entity creation, confidence scoring, backend abstraction traits
- **Outside scope**: Transcription (transcripts arrive as markdown), review UI (that's clotho-cli), model training

## Speech Act Ontology

The extraction pipeline classifies utterances in transcripts according to a speech act ontology. Each speech act maps to a specific output entity type.

| Speech Act | Signal Patterns | Output Entity | Ownership |
|------------|-----------------|---------------|-----------|
| `Commit` | "I'll do X", "I can take that", "Let me handle" | Draft Task | Speaker owns |
| `Decide` | "We're going with X", "Decision is Y", "We've decided" | Draft Decision | — |
| `Risk` | "The concern is...", "Risk here is...", "I'm worried about" | Draft Risk | — |
| `Block` | "We're stuck on...", "Blocked by...", "Can't proceed until" | Draft Blocker | — |
| `Question` | "We need to figure out...", "Open question:", "How do we..." | Draft Question | — |
| `Insight` | "What we learned...", "Key takeaway...", "Interesting finding" | Draft Insight | — |
| `Delegate` | "Can you take this?", "Assigning to X", "@person please" | Draft Task | Target owns |
| `Request` | "I need X from you", "Can you get me...", "Please send" | Draft Task | Inbound |
| `Update` | "Here's where we are...", "Status on X...", "Quick update" | Annotation only | — |

**Note**: `Update` produces an annotation on the meeting/transcript but does not create a new entity.

## Extraction Pipeline

### Pipeline Stages

```
┌────────────┐     ┌─────────────┐     ┌─────────────┐
│ Transcript │────▶│AI Extraction│────▶│Raw Mentions │
└────────────┘     └─────────────┘     └─────────────┘
                                              │
                                              ▼
                   ┌─────────────────────────────────────┐
                   │         Entity Matching             │
                   │   Fuzzy match against known entities│
                   └─────────────────────────────────────┘
                          │                    │
                          ▼                    ▼
                   ┌──────────┐         ┌────────────┐
                   │ Matched  │         │ Unresolved │
                   │Link exists│         │Flag review │
                   └──────────┘         └────────────┘
                                              │
                                              ▼
                                       ┌─────────────┐
                                       │Human Review │
                                       │Link/Create/ │
                                       │  Discard    │
                                       └─────────────┘
```

### Stage 1: AI Extraction

Input: Transcript markdown content + known entity context from `ontology.toml`

Processing:
1. Send transcript to LLM with speech act ontology definitions
2. LLM returns structured results: speech acts with text spans, entity mentions, temporal markers

Output: Raw extraction results with:
- Speech act classifications (with source text spans)
- Entity mentions (people, programs, workstreams, artifacts)
- Temporal markers (deadlines, past references, future milestones)

### Stage 2: Entity Resolution

Input: Raw entity mentions from Stage 1

Processing:
1. Fuzzy match each mention against known entities in `ontology.toml`
2. Score matches by string similarity + context relevance
3. Classify as Matched (high confidence) or Unresolved (needs review)

Output:
- Matched mentions → linked to existing entity IDs
- Unresolved mentions → flagged for human review

### Stage 3: Draft Entity Creation

Input: Classified speech acts + resolved entity mentions

Processing:
1. Create draft entities (Decision, Risk, Blocker, Question, Insight, Task) based on speech acts
2. Attach source spans for provenance
3. Attach confidence scores
4. Link to resolved entities via relations
5. Write to `data/extractions.jsonl`

Output: Draft entities ready for human review

## Entity Extraction Details

### Entity Mentions Extracted

- **People** — Attendees + mentioned individuals (for Person entity resolution)
- **Programs, Workstreams, Responsibilities** — Referenced structural/execution entities
- **Artifacts** — Referenced documents, PRs, presentations
- **Temporal markers** — Deadlines, past event references, future milestones

### Confidence Model

Each draft entity carries a `confidence: f32` score (0.0 to 1.0):
- Score reflects the LLM's confidence in the speech act classification
- Source span quality factors in (explicit statement vs. implied)
- Entity resolution confidence is separate from speech act confidence

**All drafts require human review regardless of confidence score** (human-in-the-loop principle).

### Draft Lifecycle

```
draft → promoted    (human approves: moved from extractions.jsonl to entities.jsonl)
draft → discarded   (human rejects: removed from extractions.jsonl)
```

During review, users can also:
- Edit the draft before promoting (change title, add context)
- Link unresolved mentions to existing entities
- Create new entities from unresolved mentions

## Requirements

### Functional Requirements

| ID | Requirement | Rationale |
|----|-------------|-----------|
| REQ-4.1 | Classify speech acts from transcript text | Core extraction capability |
| REQ-4.2 | Extract entity mentions (people, programs, artifacts) | Entity resolution needs |
| REQ-4.3 | Fuzzy match mentions against known entities | Reduce manual linking |
| REQ-4.4 | Produce draft entities with confidence scores and source spans | Provenance and review support |
| REQ-4.5 | Store drafts in extractions.jsonl until review | Separation from promoted entities |
| REQ-4.6 | Support promote and discard operations on drafts | Human-in-the-loop lifecycle |

### Non-Functional Requirements

| ID | Requirement | Rationale |
|----|-------------|-----------|
| NFR-4.1 | All extractions start as drafts, never auto-promoted | Human-in-the-loop principle |
| NFR-4.2 | Extraction must be idempotent (re-running on same transcript produces same drafts) | Reliability |
| NFR-4.3 | Source spans must reference exact transcript locations | Verifiability |

## Open Questions

1. **Confidence thresholds** — What score triggers auto-draft vs. requires higher scrutiny during review? (Note: all are drafts regardless, but UI could sort/highlight by confidence.)
2. ~~**LLM API selection**~~ — Resolved by CLO-A-0001: trait-based backend abstraction with per-task model selection via config. Claude Sonnet as default for extraction, Haiku for summarization.
3. **Incremental extraction** — If a transcript is updated, should we re-extract from scratch or diff?