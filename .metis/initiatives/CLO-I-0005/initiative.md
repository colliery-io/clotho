---
id: cli-interface-clotho-cli
level: initiative
title: "CLI Interface (clotho-cli)"
short_code: "CLO-I-0005"
created_at: 2026-03-16T13:23:16.455200+00:00
updated_at: 2026-03-16T13:23:16.455200+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
initiative_id: cli-interface-clotho-cli
---

# CLI Interface (clotho-cli)

## Context

The `clotho-cli` crate is the primary user interface. It provides the `clotho` command with subcommands for workspace initialization, transcript ingestion, extraction review, graph querying, and reflection creation. It ties together all other crates into a cohesive user experience.

## Goals & Non-Goals

**Goals:**
- `clotho init` — Initialize a new .workspace/ directory
- `clotho ingest transcript <file> --meeting <name>` — Ingest a transcript and trigger extraction
- `clotho review` — Interactive review of draft extractions (promote/edit/discard)
- `clotho query <cypher>` — Run Cypher queries against the relation graph
- `clotho reflect --period <type>` — Create a new reflection entry
- Human-friendly output with optional structured (JSON) output

**Non-Goals:**
- GUI or web interface
- Real-time watching/daemon mode (that's closer to clotho-sync territory)
- Direct database manipulation

## Detailed Design

### Core Commands

```bash
clotho init                                    # Initialize workspace
clotho ingest transcript <file> --meeting "..."  # Ingest + extract
clotho review                                  # Review draft extractions
clotho query "<cypher>"                        # Query relation graph
clotho reflect --period weekly                 # Create reflection
```

### Dependencies

- `clotho-core` — Entity types
- `clotho-store` — Workspace operations
- `clotho-graph` — Graph queries
- `clotho-extract` — Extraction pipeline
- `clotho-sync` — Git sync triggers

### UX Principles

- Commands should feel natural and fast
- Review flow should present drafts one at a time with clear promote/edit/discard options
- Query results should be human-readable by default, JSON with `--json` flag

## Alternatives Considered

- **TUI (terminal UI)** — Could be added later but CLI-first keeps the initial scope manageable
- **REPL mode** — Possible future enhancement; batch commands are sufficient for v1

## Implementation Plan

1. Set up clap command structure with subcommands
2. Implement `init` (delegates to clotho-store)
3. Implement `ingest` (delegates to clotho-store + clotho-extract)
4. Implement `review` (interactive draft review loop)
5. Implement `query` (delegates to clotho-graph)
6. Implement `reflect` (delegates to clotho-store)
7. Add `--json` output flag across commands