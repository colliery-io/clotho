---
id: mcp-server-clotho-mcp
level: initiative
title: "MCP Server (clotho-mcp)"
short_code: "CLO-I-0006"
created_at: 2026-03-16T13:23:16.462948+00:00
updated_at: 2026-03-17T12:54:16.889092+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: mcp-server-clotho-mcp
---

# MCP Server (clotho-mcp)

## Context

The `clotho-mcp` crate exposes Clotho's capabilities as a Model Context Protocol (MCP) server, enabling AI agents (like Claude Code) to interact with the workspace. This allows AI assistants to query the relation graph, read entities, trigger extractions, and surface relevant context during conversations.

## Goals & Non-Goals

**Goals:**
- Implement an MCP server exposing Clotho tools
- Provide tools for: entity CRUD, graph queries, search, extraction triggering, reflection creation
- Support stdio transport for local AI agent integration
- Expose workspace state as MCP resources

**Non-Goals:**
- Replacing the CLI (MCP is a complementary interface)
- Remote/network MCP serving (local stdio only for v1)
- Autonomous extraction without human review triggers

## Detailed Design

### MCP Tools (planned)

- `clotho_search` — Full-text search across entities
- `clotho_query` — Run Cypher queries against the graph
- `clotho_read_entity` — Read a specific entity by ID
- `clotho_list_entities` — List entities by type/filter
- `clotho_ingest` — Trigger transcript ingestion
- `clotho_list_drafts` — List pending extraction drafts
- `clotho_create_note` — Create a new note
- `clotho_create_reflection` — Create a new reflection

### Key Modules

- `server.rs` — MCP server setup and transport
- `tools/` — Individual tool implementations mapping to clotho-core/store/graph operations

### Dependencies

- `clotho-core`, `clotho-store`
- `rust-mcp-sdk` 0.8.0 (same as Metis) with features: server, macros, stdio
- `tokio`, `schemars`, `async-trait`

### Architecture (follows Metis pattern)

- `#[mcp_tool]` macro on tool structs with `JsonSchema` derive
- Each tool has `call_tool(&self) -> Result<CallToolResult, CallToolError>`
- `ServerHandler` impl dispatches by tool name
- Stdio transport via `rust-mcp-sdk::StdioTransport`

### MCP Tools (v1)

- `clotho_init` — Initialize workspace
- `clotho_ingest` — Ingest a file as content
- `clotho_search` — FTS5 keyword search
- `clotho_query` — Raw Cypher query
- `clotho_list_entities` — List entities with filters
- `clotho_read_entity` — Read entity by ID (metadata + content)
- `clotho_create_note` — Create a note
- `clotho_create_reflection` — Create a reflection

## Alternatives Considered

- **REST API** — MCP is more natural for AI agent integration; REST could be added later
- **gRPC** — Too heavy for single-user local tool; MCP's JSON-RPC is simpler
- **Hand-rolled JSON-RPC** — Unnecessary; rust-mcp-sdk is proven in Metis

## Implementation Plan

1. clotho-mcp crate scaffold + server skeleton (rust-mcp-sdk, stdio transport)
2. Implement read-only tools (search, query, read_entity, list_entities)
3. Implement write tools (init, ingest, create_note, create_reflection)
4. Integration tests