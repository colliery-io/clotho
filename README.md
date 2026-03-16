# Clotho

<div align="center">
  <strong>Personal work and time management through reflection, transcripts, and sense-making.</strong>
  <br><br>
  Spin the raw threads of meetings, notes, and artifacts into a coherent narrative of your work.
</div>

## Overview

Clotho is a personal knowledge management system designed for individuals who want to:

- **Capture** the flow of work — meetings, transcripts, notes, reflections, and artifacts
- **Extract** meaning from that flow — decisions, risks, blockers, insights, and action items
- **Connect** everything into a queryable graph of relationships
- **Reflect** on patterns across time periods and programs

Named after the Greek Fate who spins the thread of life, Clotho takes raw material and weaves it into something you can follow.

## Core Concepts

### Structural Layer — "What You Do"

| Entity | Purpose |
|--------|---------|
| **Responsibility** | Ongoing role obligations — mentorship, reporting, budgets. Never "complete." |
| **Program** | Strategic initiatives with objectives — technical education, PMO establishment. |
| **Objective** | Outcomes you're driving toward within a program. |

### Execution Layer — "Work in Motion"

| Entity | Purpose |
|--------|---------|
| **Workstream** | Long-running work threads. Active or inactive. |
| **Task** | Discrete work items. States: `todo → doing → blocked → done` |
| **Cadence** | Recurring schedule metadata (quarterly reviews, weekly syncs). |

### Capture Layer — "Raw Material"

| Entity | Purpose |
|--------|---------|
| **Meeting** | Container for transcripts and notes. |
| **Transcript** | Raw meeting content, source for extraction. |
| **Note** | Authored content, freeform. |
| **Reflection** | Time-period bound thinking (daily, weekly, quarterly, adhoc). |
| **Artifact** | Deliverables with links to source — docs, PRs, presentations. |

### Derived Layer — "Sense-Making"

| Entity | Purpose |
|--------|---------|
| **Decision** | Extracted decision point. |
| **Risk** | Identified risk. |
| **Blocker** | Impediment to progress. |
| **Question** | Open question requiring resolution. |
| **Insight** | Learning or observation worth preserving. |

All derived entities start as **drafts** and require human review to promote, edit, or discard.

## AI Extraction

Clotho uses AI to extract structured information from transcripts:

**Speech Acts:**
- `Commit` — "I'll do X" → Draft Task (owned by speaker)
- `Decide` — "We're going with X" → Draft Decision
- `Risk` — "The concern is..." → Draft Risk
- `Block` — "We're stuck on..." → Draft Blocker
- `Question` — "We need to figure out..." → Draft Question
- `Insight` — "What we learned..." → Draft Insight
- `Delegate` — "Can you take this?" → Draft Task (owned by target)
- `Request` — "I need X from you" → Draft Task (inbound)
- `Update` — "Here's where we are..." → Annotation (no new entity)

**Entity Resolution:**
- Extracted mentions are fuzzy-matched against known entities
- Unresolved mentions are flagged for human review
- Review can link to existing, create new, or discard

## Relations

Clotho maintains a graph of typed relationships:

```cypher
// Structural
(task)-[:BELONGS_TO]->(program)
(artifact)-[:DELIVERS]->(objective)

// Provenance  
(note)-[:SPAWNED_FROM]->(meeting)
(extraction)-[:EXTRACTED_FROM]->(transcript)

// Semantic
(meeting)-[:HAS_DECISION]->(decision)
(task)-[:BLOCKED_BY]->(blocker)

// Mentions
(transcript)-[:MENTIONS]->(person)
(note)-[:MENTIONS]->(program)
```

Query with Cypher via graphqlite:

```cypher
// What decisions came from PMO meetings?
MATCH (p:Program {title: 'PMO Establishment'})<-[:RELATES_TO]-(m:Meeting)-[:HAS_DECISION]->(d:Decision)
RETURN m.title, d.title

// What's blocking the monolith breakup?
MATCH (p:Program {title: 'Monolith Breakup'})<-[:BELONGS_TO]-(t:Task)-[:BLOCKED_BY]->(b:Blocker)
RETURN t.title, b.title
```

## Storage

```
.workspace/
├── content/           # Markdown (human-readable, git-synced)
│   ├── meetings/
│   ├── reflections/
│   ├── artifacts/
│   └── notes/
├── data/              # JSONL (append-friendly, git-synced)
│   ├── entities.jsonl
│   ├── extractions.jsonl
│   ├── tags.jsonl
│   └── events.jsonl
├── graph/             # graphqlite (git-synced)
│   └── relations.db
├── index/             # SQLite + FTS5 (gitignored, rebuilt)
│   └── search.db
└── config/            # TOML (git-synced)
    ├── config.toml
    └── ontology.toml
```

**Design principles:**
- `content/` is what you browse in an editor
- `data/` is machine-managed, append-friendly
- `graph/` is the source of truth for relations
- `index/` is derived and rebuilt on clone
- Git as sync layer, not VCS — shallow history (~20 commits), silent auto-push

## Installation

```bash
# Coming soon
curl -fsSL https://raw.githubusercontent.com/colliery-io/clotho/main/scripts/install.sh | bash
```

## Usage

```bash
# Initialize a workspace
clotho init

# Ingest a transcript
clotho ingest transcript meeting-notes.md --meeting "2025-01-15 Standup"

# Review draft extractions
clotho review

# Query the graph
clotho query "MATCH (t:Task)-[:BLOCKED_BY]->(b) RETURN t.title, b.title"

# Create a reflection
clotho reflect --period weekly
```

## Architecture

```
clotho/
├── clotho-core       # Domain logic, entities, traits
├── clotho-graph      # graphqlite integration
├── clotho-store      # DAL for content/, data/, index/
├── clotho-extract    # AI extraction pipeline
├── clotho-cli        # Command-line interface
├── clotho-mcp        # MCP server for AI agents
└── clotho-sync       # Git sync layer
```

## License

Apache 2.0