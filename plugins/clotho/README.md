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

This plugin provides 21 MCP tools, 10 skills, and 4 agents for managing a Clotho workspace.

### Session
- `clotho_set_workspace` — Set workspace path (auto-detected on startup)

### Read Tools
- `clotho_search` — Full-text keyword search
- `clotho_query` — Cypher graph queries
- `clotho_read_entity` — Read entity by ID
- `clotho_list_entities` — List with filters
- `clotho_get_relations` — Show entity relations
- `clotho_get_ontology` — Get extraction ontology
- `clotho_search_ontology` — Search across ontologies
- `clotho_check_processed` — Check processing history

### Write Tools
- `clotho_init` — Initialize workspace
- `clotho_capture` — Capture a file
- `clotho_create_entity` — Create any entity type
- `clotho_update_entity` — Update entity fields
- `clotho_delete_entity` — Delete entity
- `clotho_create_note` — Create a note
- `clotho_create_reflection` — Create a reflection
- `clotho_create_relation` — Create graph edge
- `clotho_delete_relation` — Remove graph edge
- `clotho_update_ontology` — Update extraction ontology
- `clotho_mark_processed` — Record processing done
- `clotho_sync` — Git sync workspace

### Ceremonies
- `/daily-debrief` — End of day: capture, update, horizon check, extract
- `/daily-brief` — Start of day: prioritized view
- `/weekly-review` — End of week: reflection + pattern analysis
- `/report` — Status reports for boss/stakeholders/team
- `/period-review` — Quarterly+ retrospective

### Skills
- **workspace-management** — Entity CRUD operations
- **graph-queries** — Relations and Cypher queries
- **extraction** — In-session speech act extraction
- **reflection** — Guided reflection creation
- **transcript-processor** — Single transcript processing

### Agents
- **debrief-processor** — Extract signals from transcripts using program ontologies
- **review-compiler** — Weekly pattern analysis
- **report-builder** — Audience-appropriate report generation
- **period-compiler** — Deep retrospective analysis

## Entity Types

| Layer | Entities |
|-------|---------|
| Structural | Program, Responsibility, Objective |
| Execution | Workstream, Task |
| Capture | Meeting, Transcript, Note, Reflection, Artifact |
| Derived | Decision, Risk, Blocker, Question, Insight |
| Cross-cutting | Person |
