---
id: cli-interface-clotho-cli
level: initiative
title: "CLI Interface (clotho-cli)"
short_code: "CLO-I-0005"
created_at: 2026-03-16T13:23:16.455200+00:00
updated_at: 2026-03-17T02:32:55.582726+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: cli-interface-clotho-cli
---

# CLI Interface (clotho-cli)

## Context

The `clotho-cli` crate is the primary user interface. It provides the `clotho` command with subcommands for workspace initialization, transcript ingestion, extraction review, graph querying, and reflection creation. It ties together all other crates into a cohesive user experience.

## Goals & Non-Goals

**Goals (v1):**
- `clotho init` — Initialize a new .workspace/ directory
- `clotho ingest <file> [--type meeting|note|artifact] [--title "..."]` — Ingest content (store only, no extraction)
- `clotho query <cypher>` — Raw Cypher queries against the relation graph
- `clotho search <query>` — FTS5 keyword search across all entities
- `clotho list [--type X] [--status Y] [--state Z]` — List entities with filters
- `clotho reflect --period <type>` — Create a new reflection entry
- `--json` flag for structured output across all commands

**Deferred (needs clotho-extract):**
- `clotho review` — Interactive review of draft extractions
- Extraction triggering on ingest

**Deferred (needs clotho-sync):**
- Git sync triggers on workspace mutations

**Non-Goals:**
- GUI or web interface
- Real-time watching/daemon mode
- Direct database manipulation

## Detailed Design

### Core Commands (v1)

```bash
clotho init                                          # Initialize workspace
clotho ingest <file> --type note --title "My Note"   # Ingest content
clotho query "MATCH (n:Task) RETURN n.title"          # Cypher query
clotho search "deployment risk"                       # FTS5 keyword search
clotho list --type Task --state doing                 # List entities
clotho reflect --period weekly                        # Create reflection
```

### Dependencies

- `clotho-core` — Entity types, graph
- `clotho-store` — Workspace, content, data, search

### UX Principles

- Commands should feel natural and fast
- Review flow should present drafts one at a time with clear promote/edit/discard options
- Query results should be human-readable by default, JSON with `--json` flag

## Alternatives Considered

- **TUI (terminal UI)** — Could be added later but CLI-first keeps the initial scope manageable
- **REPL mode** — Possible future enhancement; batch commands are sufficient for v1

## Implementation Plan

1. clotho-cli crate scaffold + clap command structure
2. Implement `init` (delegates to Workspace::init)
3. Implement `ingest` (reads file, stores as content + entity row)
4. Implement `list` (queries EntityStore with filters)
5. Implement `search` (delegates to SearchIndex)
6. Implement `query` (delegates to GraphStore::raw_cypher)
7. Implement `reflect` (creates Reflection entity + content file)
8. Add `--json` output flag across commands