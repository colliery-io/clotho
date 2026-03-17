---
id: storage-data-access-layer-clotho
level: initiative
title: "Storage & Data Access Layer (clotho-store)"
short_code: "CLO-I-0003"
created_at: 2026-03-16T13:23:16.426176+00:00
updated_at: 2026-03-17T02:04:31.242090+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
initiative_id: storage-data-access-layer-clotho
---

# Storage & Data Access Layer (clotho-store)

## Context

The `clotho-store` crate is the data access layer that coordinates across the four storage backends: Markdown content files, JSONL data files, the graphqlite relation graph, and the SQLite+FTS5 search index. It owns the `.workspace/` directory structure and provides a unified API for reading and writing entities.

## Goals & Non-Goals

**Goals:**
- Separate `clotho-store` crate depending on `clotho-core`
- Manage the `.workspace/` directory structure (content/, data/, graph/, index/, config/)
- Implement `content.rs` — Markdown file operations for ContentBearing entities
- Implement `data.rs` — SQLite relational storage for entities and extractions (CLO-A-0002), JSONL for tags and events
- Implement `index.rs` — SQLite+FTS5 keyword search (derived, rebuildable)
- Implement `federation.rs` — Thin ATTACH-based cross-database query support (CLO-A-0002)
- Implement `sync.rs` — Coordination layer ensuring consistency across backends, including temporal edge materialization
- Support workspace initialization (`clotho init`) with schema migrations for entities.db/extractions.db

**Non-Goals:**
- Git sync operations (that's clotho-sync)
- AI extraction (that's clotho-extract)
- Graph queries beyond basic relay to clotho-core/graph
- VSS / vector similarity search (deferred — sqlite-vss adds build complexity, FTS5 is sufficient for v1)
- Embedding generation (that's clotho-extract via Embedder trait)

## Detailed Design

### Directory Structure Managed

```
.workspace/
├── content/              # Markdown (human-readable)
│   ├── meetings/
│   ├── reflections/
│   ├── artifacts/
│   ├── notes/
│   └── people/
├── data/                 # Structured storage
│   ├── entities.db       # SQLite — relational entity storage (CLO-A-0002)
│   ├── extractions.db    # SQLite — draft extractions pending review (CLO-A-0002)
│   ├── tags.jsonl        # Freeform tags (append-only JSONL)
│   └── events.jsonl      # Activity log (append-only JSONL)
├── graph/                # graphqlite (managed by clotho-core)
│   └── relations.db
├── index/                # Derived cache (gitignored, rebuilt)
│   └── search.db         # SQLite + FTS5
└── config/               # Settings
    ├── config.toml
    └── ontology.toml
```

### Format Rationale

| Layer | Format | Why |
|-------|--------|-----|
| content/ | Markdown | Human-readable, browsable in any editor |
| data/ (entities, extractions) | SQLite | Relational queries, schema enforcement, indexed filters (CLO-A-0002) |
| data/ (tags, events) | JSONL | Append-only simplicity, git-friendly diffs |
| graph/ | graphqlite | Native Cypher queries, relation-first design |
| index/ | SQLite+FTS5 | Fast keyword search, derived/rebuildable |
| config/ | TOML | Human-editable, simple |

### Key Modules

- `workspace.rs` — Workspace initialization, directory creation, config scaffolding.
- `content.rs` — Read/write markdown for meetings, notes, reflections, artifacts, people. Maps ContentBearing entities to file paths.
- `data.rs` — SQLite relational storage for entities (entities.db) and extractions (extractions.db) per CLO-A-0002. JSONL for tags and events. Includes schema definitions.
- `index.rs` — FTS5 keyword search (search.db). Derived/rebuildable from content/ and data/.
- `federation.rs` — Thin ATTACH-based cross-database query support (CLO-A-0002). Proves the pattern for cross-DB joins.
- `sync.rs` — Coordination layer for writes across backends. Handles temporal edge materialization (HasCadence/HasDeadline/HasSchedule → graph edges).

## Alternatives Considered

- **Single SQLite database** — Simpler but loses human-readable browsability of markdown files and git-friendly JSONL diffs
- **Pure filesystem (no JSONL)** — Entity metadata needs structured storage; markdown frontmatter alone gets unwieldy
- **Sled/RocksDB** — Overkill for single-user; SQLite is more portable and debuggable

## Implementation Plan

1. clotho-store crate scaffold + workspace init
2. Content module (markdown read/write, path mapping)
3. Data module — entities.db schema, CRUD operations (CLO-A-0002)
4. Data module — extractions.db schema, draft lifecycle operations (CLO-A-0002)
5. Data module — JSONL for tags and events (append/read)
6. Index module — FTS5 schema, index build, keyword search queries
7. Federation module — thin ATTACH-based cross-database queries (CLO-A-0002)
8. Sync coordination layer (writes across all backends + temporal edge materialization)
9. Integration tests with temp workspace directories