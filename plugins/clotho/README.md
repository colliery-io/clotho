# Clotho Claude Code Plugin

Personal work and time management through reflection, transcripts, and sense-making.

## Installation

1. Build the MCP server:
   ```bash
   cargo build --release -p clotho-mcp
   ```

2. Ensure `clotho-mcp` is in your PATH, or update `.mcp.json` with the full path.

3. Add the plugin to Claude Code:
   ```bash
   claude plugin add /path/to/clotho/plugins/clotho
   ```

## What It Does

This plugin provides 15 MCP tools for managing a Clotho workspace:

### Read-only Tools
- `clotho_search` — Full-text keyword search
- `clotho_query` — Cypher graph queries
- `clotho_read_entity` — Read entity by ID
- `clotho_list_entities` — List with filters
- `clotho_get_relations` — Show entity relations

### Write Tools
- `clotho_init` — Initialize workspace
- `clotho_ingest` — Ingest a file
- `clotho_create_entity` — Create any entity type
- `clotho_update_entity` — Update entity fields
- `clotho_delete_entity` — Delete entity
- `clotho_create_note` — Create a note
- `clotho_create_reflection` — Create a reflection
- `clotho_create_relation` — Create graph edge
- `clotho_delete_relation` — Remove graph edge
- `clotho_sync` — Git sync workspace

### Skills
- **workspace-management** — Entity CRUD operations
- **graph-queries** — Relations and Cypher queries
- **extraction** — In-session speech act extraction from transcripts
- **reflection** — Guided reflection creation workflow

## Entity Types

| Layer | Entities |
|-------|---------|
| Structural | Program, Responsibility, Objective |
| Execution | Workstream, Task |
| Capture | Meeting, Transcript, Note, Reflection, Artifact |
| Derived | Decision, Risk, Blocker, Question, Insight |
| Cross-cutting | Person |
