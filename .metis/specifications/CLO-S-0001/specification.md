---
id: system-architecture
level: specification
title: "System Architecture"
short_code: "CLO-S-0001"
created_at: 2026-03-16T13:30:36.974666+00:00
updated_at: 2026-03-16T13:30:36.974666+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#specification"
  - "#phase/discovery"


exit_criteria_met: false
initiative_id: NULL
---

# System Architecture

## Overview

Clotho is a Rust workspace organized as 7 crates, each with a clear responsibility boundary. The system follows a layered architecture where domain logic (clotho-core) sits at the center, with storage (clotho-store), graph (clotho-graph), and extraction (clotho-extract) as middle-tier services, and CLI (clotho-cli), MCP (clotho-mcp), and sync (clotho-sync) as interface/infrastructure layers.

## System Context

### Actors
- **User (CLI)**: Interacts via the `clotho` command — initializes workspaces, ingests transcripts, reviews extractions, queries the graph, creates reflections
- **AI Agent (MCP)**: Interacts via MCP server over stdio — queries entities, reads context, triggers ingestion, creates notes/reflections
- **Git Remote**: Passive sync target — receives auto-pushed workspace state for multi-device replication

### External Systems
- **LLM Providers**: Used by clotho-extract for speech act classification, entity extraction, and summarization. Backend is trait-based and configurable — Claude API is the default, with support for multiple providers and per-task model selection (see CLO-A-0001).
- **graphqlite**: SQLite-backed graph database with Cypher query support (colliery-io/graphqlite)
- **Git**: Used as a dumb sync layer, not version control

### Boundaries
- **Inside scope**: Workspace management, entity lifecycle, relation graph, AI extraction, search index, git sync, CLI and MCP interfaces
- **Outside scope**: Transcription services (transcripts arrive as markdown), web UI, multi-user collaboration, custom model training

## Crate Structure

```
clotho/
├── Cargo.toml              # Workspace manifest
│
├── clotho-core/            # Domain logic (entities, traits, types)
│   └── src/
│       ├── lib.rs
│       ├── domain/
│       │   ├── entities/   # All 16 entity structs
│       │   ├── traits.rs   # 7 core traits
│       │   └── types.rs    # EntityId, Status, TaskState, etc.
│       └── error.rs
│
├── clotho-graph/           # graphqlite integration
│   └── src/
│       ├── lib.rs
│       ├── schema.rs       # Node/edge definitions
│       └── queries.rs      # Common Cypher queries
│
├── clotho-store/           # Data access layer
│   └── src/
│       ├── lib.rs
│       ├── content.rs      # Markdown file ops
│       ├── data.rs         # JSONL ops
│       ├── index.rs        # FTS5 search
│       └── sync.rs         # Cross-backend coordination
│
├── clotho-extract/         # AI extraction pipeline
│   └── src/
│       ├── lib.rs
│       ├── ontology.rs     # Speech act definitions
│       ├── pipeline.rs     # Extraction flow
│       └── resolution.rs   # Entity matching
│
├── clotho-cli/             # Command-line interface
│   └── src/
│       └── main.rs
│
├── clotho-mcp/             # MCP server
│   └── src/
│       ├── lib.rs
│       ├── server.rs
│       └── tools/
│
└── clotho-sync/            # Git sync layer
    └── src/
        ├── lib.rs
        ├── commit.rs
        └── push.rs
```

## Dependency Graph

```
clotho-cli ──┬──▶ clotho-core
              ├──▶ clotho-store ──▶ clotho-core
              ├──▶ clotho-graph ──▶ clotho-core
              ├──▶ clotho-extract ──▶ clotho-core
              └──▶ clotho-sync

clotho-mcp ──┬──▶ clotho-core
              ├──▶ clotho-store
              ├──▶ clotho-graph
              └──▶ clotho-extract
```

All crates depend on `clotho-core`. The CLI and MCP server are the top-level consumers. `clotho-sync` is independent of business logic — it only knows about filesystem paths.

## Requirements

### Functional Requirements

| ID | Requirement | Rationale |
|----|-------------|-----------|
| REQ-1.1 | Initialize a .workspace/ directory with correct structure | Entry point for all usage |
| REQ-1.2 | Ingest markdown transcripts and trigger AI extraction | Core capture workflow |
| REQ-1.3 | Present draft extractions for human review (promote/edit/discard) | Human-in-the-loop principle |
| REQ-1.4 | Query the relation graph via Cypher | Relations are first-class |
| REQ-1.5 | Full-text search across all entities | Discovery and navigation |
| REQ-1.6 | Create time-period-bound reflections | Sense-making workflow |
| REQ-1.7 | Expose all capabilities via MCP server | AI agent integration |
| REQ-1.8 | Auto-sync workspace via git | Multi-device replication |

### Non-Functional Requirements

| ID | Requirement | Rationale |
|----|-------------|-----------|
| NFR-1.1 | All data readable without Clotho installed (Markdown, JSONL, TOML) | Portable format principle |
| NFR-1.2 | Single-user performance (no concurrent access optimization needed) | Design constraint |
| NFR-1.3 | Search index fully rebuildable from content/ and data/ | index/ is gitignored |
| NFR-1.4 | Git sync operates silently with shallow history (~20 commits) | Sync, not VCS |

## Constraints

### Technical Constraints
- Rust implementation across all crates
- graphqlite (colliery-io/graphqlite) as the graph backend
- SQLite + FTS5 for full-text search
- Git as sync transport (not version control)
- Single-user, multi-device architecture

### Design Constraints
- All AI extractions must start as drafts (never auto-promoted)
- Markdown content must be human-browsable in any editor
- JSONL for machine data (git-friendly line-based diffs)
- No proprietary formats anywhere in the workspace