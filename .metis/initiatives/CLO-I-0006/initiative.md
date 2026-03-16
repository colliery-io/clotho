---
id: mcp-server-clotho-mcp
level: initiative
title: "MCP Server (clotho-mcp)"
short_code: "CLO-I-0006"
created_at: 2026-03-16T13:23:16.462948+00:00
updated_at: 2026-03-16T13:23:16.462948+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


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

- `clotho-core`, `clotho-store`, `clotho-graph`, `clotho-extract`
- MCP SDK for Rust (or manual JSON-RPC implementation)

## Alternatives Considered

- **REST API** — MCP is more natural for AI agent integration; REST could be added later
- **gRPC** — Too heavy for single-user local tool; MCP's JSON-RPC is simpler

## Implementation Plan

1. Set up MCP server skeleton with stdio transport
2. Implement read-only tools (search, query, read, list)
3. Implement write tools (ingest, create note/reflection)
4. Implement draft management tools
5. Test with Claude Code integration