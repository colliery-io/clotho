---
id: storage-data-access-layer-clotho
level: initiative
title: "Storage & Data Access Layer (clotho-store)"
short_code: "CLO-I-0003"
created_at: 2026-03-16T13:23:16.426176+00:00
updated_at: 2026-03-16T13:23:16.426176+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
initiative_id: storage-data-access-layer-clotho
---

# Storage & Data Access Layer (clotho-store)

## Context

The `clotho-store` crate is the data access layer that coordinates across the four storage backends: Markdown content files, JSONL data files, the graphqlite relation graph, and the SQLite+FTS5 search index. It owns the `.workspace/` directory structure and provides a unified API for reading and writing entities.

## Goals & Non-Goals

**Goals:**
- Manage the `.workspace/` directory structure (content/, data/, graph/, index/, config/)
- Implement `content.rs` — Markdown file operations for ContentBearing entities
- Implement `data.rs` — SQLite relational storage for entities and extractions (CLO-A-0002), JSONL for tags and events
- Implement `index.rs` — SQLite+FTS5 keyword search + SQLite+VSS semantic similarity search (CLO-A-0003)
- Implement `sync.rs` — Coordination layer ensuring consistency across backends, including ATTACH-based federation
- Support workspace initialization (`clotho init`) with schema migrations for entities.db/extractions.db

**Non-Goals:**
- Git sync operations (that's clotho-sync)
- AI extraction (that's clotho-extract)
- Graph queries beyond basic relay to clotho-graph
- Embedding generation (that's clotho-extract via Embedder trait; clotho-store only stores/queries vectors)

## Detailed Design

### Directory Structure Managed

```
.workspace/
├── content/              # Markdown (human-readable)
│   ├── meetings/
│   ├── reflections/
│   ├── artifacts/
│   └── notes/
├── data/                 # JSONL (machine-managed)
│   ├── entities.jsonl
│   ├── extractions.jsonl
│   ├── tags.jsonl
│   └── events.jsonl
├── graph/                # graphqlite
│   └── relations.db
├── index/                # SQLite+FTS5 (gitignored, rebuilt)
│   └── search.db
└── config/               # TOML
    ├── config.toml
    └── ontology.toml
```

### Format Rationale

| Layer | Format | Why |
|-------|--------|-----|
| content/ | Markdown | Human-readable, browsable in any editor |
| data/ | JSONL | Append-friendly, line-based diffs, stream-processable |
| graph/ | graphqlite | Native Cypher queries, relation-first design |
| index/ | SQLite+FTS5 | Fast full-text search, derived/rebuildable |
| config/ | TOML | Human-editable, simple |

### Key Modules

- `content.rs` — Read/write markdown for meetings, notes, reflections, artifacts. Maps ContentBearing entities to file paths.
- `data.rs` — SQLite relational storage for entities (entities.db) and extractions (extractions.db) per CLO-A-0002. JSONL for tags and events. Includes schema definitions and migration support.
- `index.rs` — FTS5 keyword search (search.db) + VSS semantic similarity search (vectors.db, CLO-A-0003). Both derived/rebuildable from content/ and data/.
- `federation.rs` — ATTACH-based cross-database query support (CLO-A-0002). Enables joins across entities.db, relations.db, search.db, and vectors.db.
- `sync.rs` — Ensures writes are atomic across backends. Coordinates content + data + graph + index updates.

## Alternatives Considered

- **Single SQLite database** — Simpler but loses human-readable browsability of markdown files and git-friendly JSONL diffs
- **Pure filesystem (no JSONL)** — Entity metadata needs structured storage; markdown frontmatter alone gets unwieldy
- **Sled/RocksDB** — Overkill for single-user; SQLite is more portable and debuggable

## Implementation Plan

1. Workspace initialization (directory creation, config scaffolding, SQLite schema creation)
2. Content module (markdown read/write, path mapping)
3. Data module — entities.db schema, CRUD operations, migrations (CLO-A-0002)
4. Data module — extractions.db schema, draft lifecycle operations (CLO-A-0002)
5. Data module — JSONL for tags and events (append/read)
6. Index module — FTS5 schema, index build, keyword search queries
7. Index module — VSS schema, embedding storage, similarity search queries (CLO-A-0003)
8. Federation module — ATTACH-based cross-database queries (CLO-A-0002)
9. Sync coordination layer (atomic writes across all backends)
10. Integration tests with temp workspace directories