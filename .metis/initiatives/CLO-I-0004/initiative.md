---
id: ai-extraction-pipeline-clotho
level: initiative
title: "AI Extraction Pipeline (clotho-extract)"
short_code: "CLO-I-0004"
created_at: 2026-03-16T13:23:16.443679+00:00
updated_at: 2026-03-16T13:23:16.443679+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: XL
initiative_id: ai-extraction-pipeline-clotho
---

# AI Extraction Pipeline (clotho-extract)

## Context

The `clotho-extract` crate implements the AI-powered extraction pipeline that processes transcripts and notes to produce draft derived entities (Decisions, Risks, Blockers, Questions, Insights) and draft Tasks. This is the most complex initiative — it bridges unstructured human content with structured domain entities while maintaining the human-in-the-loop principle.

## Goals & Non-Goals

**Goals:**
- Implement the speech act ontology (Commit, Decide, Risk, Block, Question, Insight, Delegate, Request, Update)
- Build the extraction pipeline: Transcript → AI Extraction → Raw Mentions → Entity Matching → Human Review
- Implement entity resolution with fuzzy matching against known entities
- Produce draft entities with confidence scores and source spans
- Support the full extraction lifecycle: draft → promoted | discarded

**Non-Goals:**
- Transcription itself (transcripts arrive as markdown)
- The review UI (that's clotho-cli)
- Training custom models (uses existing LLM APIs)

## Architecture

### Extraction Pipeline

```
Transcript → AI Extraction → Raw Mentions → Entity Matching → Draft Entities → Human Review
```

### Speech Act Ontology

| Speech Act | Signal Patterns | Output |
|------------|-----------------|--------|
| Commit | "I'll do X", "Let me handle" | Draft Task (speaker owns) |
| Decide | "We're going with X", "Decision is Y" | Draft Decision |
| Risk | "The concern is...", "I'm worried about" | Draft Risk |
| Block | "We're stuck on...", "Can't proceed until" | Draft Blocker |
| Question | "We need to figure out...", "How do we..." | Draft Question |
| Insight | "What we learned...", "Key takeaway..." | Draft Insight |
| Delegate | "Can you take this?", "Assigning to X" | Draft Task (target owns) |
| Request | "I need X from you", "Please send" | Draft Task (inbound) |
| Update | "Here's where we are...", "Status on X" | Annotation (no entity) |

### Entity Resolution Flow

1. AI extracts raw mentions (people, programs, workstreams, artifacts, temporal markers)
2. Fuzzy match against known entities in the ontology
3. Matched → link to existing entity
4. Unresolved → flag for human review (link/create/discard)

### Key Modules

- `ontology.rs` — Speech act definitions, signal patterns, output mappings
- `pipeline.rs` — Orchestrates the full extraction flow, uses `dyn Extractor` / `dyn Summarizer` / `dyn Resolver` traits
- `resolution.rs` — Fuzzy entity matching, confidence scoring
- `backends/mod.rs` — Backend registry and factory (selects impl from config)
- `backends/claude.rs` — Anthropic Claude implementation of Extractor, Summarizer, Resolver
- `backends/ollama.rs` — Ollama/local model implementation (future)

## Detailed Design

All extracted entities carry:
- `extraction_status`: Draft (pending review)
- `source_span`: Reference back to the transcript location
- `confidence`: Float score from the extraction model

Extractions are stored in `data/extractions.jsonl` until promoted (moved to `data/entities.jsonl`) or discarded.

## Open Questions

- What confidence threshold triggers auto-draft vs. requiring higher review?
- ~~Which LLM API to target initially~~ — Resolved by CLO-A-0001: trait-based backend abstraction. Claude is the default, with per-task model selection via config.

## Alternatives Considered

- **Rule-based extraction only** — Too brittle for natural language variation; LLM-based approach handles nuance
- **Auto-promote high-confidence extractions** — Rejected to maintain human-in-the-loop principle; all extractions start as drafts
- **Batch-only processing** — Rejected in favor of per-transcript streaming to keep the feedback loop tight

## Implementation Plan

1. Define speech act ontology data structures
2. Build extraction pipeline skeleton (transcript in, raw mentions out)
3. Implement LLM integration for speech act classification
4. Implement entity mention extraction
5. Build fuzzy entity resolution against known entities
6. Wire up draft entity creation with confidence scores and source spans
7. Integration tests with sample transcripts