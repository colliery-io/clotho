# Clotho MCP Server

Clotho is a personal work and time management system. Use these tools to interact with a Clotho workspace.

## Available Tools

- `clotho_init` — Initialize a new workspace
- `clotho_ingest` — Ingest a file as content (note, meeting, transcript, artifact)
- `clotho_search` — Full-text keyword search across all entities
- `clotho_query` — Run a Cypher query against the relation graph
- `clotho_list_entities` — List entities with optional type/status/state filters
- `clotho_read_entity` — Read an entity's metadata and content by ID
- `clotho_create_note` — Create a new note entity
- `clotho_create_reflection` — Create a new reflection entry

## Workspace Path

All tools that operate on an existing workspace require a `workspace_path` parameter pointing to the directory containing `.workspace/`.
