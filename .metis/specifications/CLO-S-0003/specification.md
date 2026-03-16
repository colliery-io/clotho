---
id: storage-architecture
level: specification
title: "Storage Architecture"
short_code: "CLO-S-0003"
created_at: 2026-03-16T13:30:38.814594+00:00
updated_at: 2026-03-16T13:30:38.814594+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#specification"
  - "#phase/discovery"


exit_criteria_met: false
initiative_id: NULL
---

# Storage Architecture

## Overview

This specification defines the `.workspace/` directory structure, the four storage backends (Markdown, JSONL, graphqlite, SQLite+FTS5), their format rationale, the git sync model, and the data flow between backends. This is the canonical reference for `clotho-store` and `clotho-sync`.

## Directory Structure

```
.workspace/
├── content/              # Human-readable markdown
│   ├── meetings/
│   │   ├── 2025-01-15-standup.md
│   │   └── 2025-01-15-standup.transcript.md
│   ├── reflections/
│   │   ├── 2025-w03-weekly.md
│   │   └── 2025-q1-quarterly.md
│   ├── artifacts/
│   │   └── pmo-roadmap-v2.md
│   └── notes/
│       └── architecture-thoughts.md
│
├── data/                 # Machine-managed structured data
│   ├── entities.db       # SQLite — relational entity storage (CLO-A-0002)
│   ├── extractions.db    # SQLite — draft extractions pending review (CLO-A-0002)
│   ├── tags.jsonl        # Freeform tags (append-only JSONL)
│   └── events.jsonl      # Activity log (append-only JSONL)
│
├── graph/                # Relation graph (graphqlite)
│   └── relations.db      # SQLite with Cypher support
│
├── index/                # Derived cache (gitignored)
│   ├── search.db         # SQLite + FTS5 (keyword search)
│   └── vectors.db        # SQLite + VSS (semantic similarity, CLO-A-0003)
│
└── config/               # Settings
    ├── config.toml       # User preferences
    └── ontology.toml     # Known entities, extraction config
```

## Storage Backends

### content/ — Markdown

**Purpose**: Human-readable content that users can browse in any editor.

**Contains**: Meeting notes, transcripts, reflections, artifacts, standalone notes.

**Conventions**:
- Meetings: `content/meetings/YYYY-MM-DD-slug.md`
- Transcripts: `content/meetings/YYYY-MM-DD-slug.transcript.md`
- Reflections: `content/reflections/YYYY-{period}-{type}.md` (e.g., `2025-w03-weekly.md`, `2025-q1-quarterly.md`)
- Artifacts: `content/artifacts/slug.md`
- Notes: `content/notes/slug.md`

**Why Markdown**: Human-readable, browsable without Clotho, universal editor support, clean git diffs.

### data/ — SQLite + JSONL (hybrid)

**Purpose**: Machine-managed structured data. Per CLO-A-0002, entity and extraction data moved to SQLite for proper relational queries; tags and events remain JSONL.

**SQLite files** (per CLO-A-0002):
- `entities.db` — Relational entity storage with schema, indexes, and SQL queries. Replaces entities.jsonl. Supports filtering by type, status, date ranges, and joins.
- `extractions.db` — Draft extractions pending human review. Supports status tracking, source span lookups, confidence sorting.

**JSONL files** (retained):
- `tags.jsonl` — Freeform tag definitions and entity-tag associations. Simple append-only, low query complexity.
- `events.jsonl` — Activity log (entity creation, state transitions, extraction events). Pure append-only, git-diff-friendly.

**Why hybrid**: Entity data needs relational queries (filters, joins, aggregations) that JSONL can't provide without full-file scans. Tags and events are append-only with low query complexity — JSONL's git-friendly diffs are more valuable there. Both SQLite databases support federated queries with graph/relations.db via `ATTACH DATABASE`.

### graph/ — graphqlite

**Purpose**: Typed relation graph, the source of truth for all entity relationships.

**Contains**: `relations.db` — A SQLite database managed by graphqlite, supporting Cypher queries.

**Why graphqlite**: Native Cypher queries are a major ergonomic win for a relation-first system. SQLite backend gives durability and portability. See CLO-S-0005 for the full relation schema.

### index/ — SQLite + FTS5 + VSS

**Purpose**: Derived search indexes. Entirely rebuildable from content/ and data/ at any time.

**Files**:
- `search.db` — SQLite + FTS5 for keyword full-text search across all entity content and metadata.
- `vectors.db` — SQLite + VSS for semantic similarity search (CLO-A-0003). Stores vector embeddings of transcript segments, extracted statements, notes, reflections, and artifact summaries. Supports nearest-neighbor queries joinable with entity and graph data via ATTACH.

**Why gitignored**: Both are derived caches. Rebuilding on clone (re-indexing for FTS5, re-embedding for VSS) is safer than syncing binary databases.

### config/ — TOML

**Purpose**: Human-editable configuration.

**Files**:
- `config.toml` — User preferences (sync settings, default period types, etc.)
- `ontology.toml` — Known entities for extraction resolution, custom speech act patterns, confidence thresholds.

**Why TOML**: Human-editable, simple syntax, good Rust ecosystem support (serde).

## Format Rationale Summary

| Layer | Format | Why |
|-------|--------|-----|
| content/ | Markdown | Human-readable, browsable in any editor |
| data/ (entities, extractions) | SQLite | Relational queries, schema enforcement, indexed filters (CLO-A-0002) |
| data/ (tags, events) | JSONL | Append-only simplicity, git-friendly diffs |
| graph/ | graphqlite | Native Cypher queries, relation-first design |
| index/ (search) | SQLite+FTS5 | Fast keyword search, derived/rebuildable |
| index/ (vectors) | SQLite+VSS | Semantic similarity search, derived/rebuildable (CLO-A-0003) |
| config/ | TOML | Human-editable, simple |

### Cross-Layer Federation (CLO-A-0002)

All SQLite databases can be joined via `ATTACH DATABASE`:

```
entities.db ←ATTACH→ relations.db ←ATTACH→ search.db ←ATTACH→ vectors.db
```

This enables queries like "find semantically similar content to this blocker, filtered to active tasks in program X, traversing the BLOCKED_BY relation" — spanning VSS, relational, and graph layers in a single query.

## Git Sync Model

Git is used as a **dumb sync layer**, not for version control.

### What Gets Synced

- **Synced**: `content/`, `data/`, `graph/`, `config/`
- **Gitignored**: `index/` (rebuilt on clone from synced data)

### Sync Behavior

- **Auto-commit**: Debounced commits on save (every 30s of inactivity)
- **Auto-push**: Silent push after each commit
- **Shallow history**: Prune to last ~20 commits
- **No branches**: Main only
- **Conflict handling**: Pull before push; single-user assumption means conflicts are rare (same user, different device)

### Data Flow on Write

1. Entity change occurs (via CLI or MCP)
2. Content written to `content/` (if ContentBearing)
3. Entity record written to `data/entities.db` (INSERT or UPDATE)
4. Relations updated in `graph/relations.db`
5. FTS5 search index updated in `index/search.db`
6. VSS embedding generated and stored in `index/vectors.db` (async, via Embedder trait)
7. Event logged to `data/events.jsonl`
8. After 30s debounce: git commit + push

### Data Flow on Clone/Pull

1. `git clone` or `git pull` brings content/, data/, graph/, config/
2. `index/search.db` is rebuilt from content/ and data/ (FTS5 re-index)
3. `index/vectors.db` is rebuilt by re-embedding all content (via Embedder trait)
4. Workspace is ready to use

## Open Questions

1. **Graph sync strategy** — Commit SQLite (graphqlite) directly or export/import a deterministic text format? Binary SQLite in git is functional but diffs are opaque. This now applies to entities.db and extractions.db as well.
2. **Schema migrations** — How to handle entities.db schema changes across versions? SQLite migration strategy needed.
3. **VSS rebuild cost** — Re-embedding all content on clone requires API calls. Should we cache embeddings in a portable format alongside the source data, or accept the rebuild cost?