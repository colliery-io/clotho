---
id: clotho
level: vision
title: "clotho"
short_code: "CLO-V-0001"
created_at: 2026-03-16T13:20:47.787388+00:00
updated_at: 2026-03-16T13:22:36.461252+00:00
archived: false

tags:
  - "#vision"
  - "#phase/published"


exit_criteria_met: false
initiative_id: NULL
---

# Clotho Vision

## Purpose

Clotho is a personal work and time management system centered on capturing, extracting, and connecting the flow of work through notes, transcripts, reflections, and artifacts. It transforms the raw threads of daily work into a coherent, queryable narrative — helping individuals who juggle multiple programs, responsibilities, and workstreams maintain clarity and make better decisions.

## Product Overview

**Target audience:** Individual knowledge workers managing complex portfolios of work — engineering managers, technical leads, program managers, and similar roles where work spans multiple streams, meetings, and stakeholders.

**Key benefits:**
- Frictionless capture of meetings, transcripts, notes, reflections, and artifacts
- AI-assisted extraction of decisions, risks, blockers, questions, and insights from unstructured content
- A queryable graph of relationships between all work entities
- Time-period-bound reflections that surface patterns across programs and responsibilities

**Architectural inspiration:** Draws from Metis (Flight Levels work management) but adapted for personal/single-user work management with a four-layer domain model (Structural → Execution → Capture → Derived) and a filesystem + graph source of truth.

## Current State

Greenfield project. Design document and target README exist. No code written yet. The domain model, storage architecture, trait system, crate structure, and AI extraction pipeline have been designed but not implemented.

## Future State

A fully functional Rust-based personal work management system with:
- A CLI (`clotho`) for all interactions — init, ingest, review, query, reflect
- An MCP server (`clotho-mcp`) for AI agent integration
- A workspace format (`.workspace/`) using Markdown, JSONL, graphqlite, and SQLite+FTS5
- An AI extraction pipeline that identifies speech acts in transcripts and produces draft entities for human review
- Git-based sync across devices (shallow history, auto-commit/push, single-user assumption)

## Major Features

- **Four-layer domain model:** Structural (Responsibility, Program, Objective), Execution (Workstream, Task, Cadence), Capture (Meeting, Transcript, Note, Reflection, Artifact), Derived (Decision, Risk, Blocker, Question, Insight)
- **Trait-based entity system:** Core traits (Entity, Activatable, Taskable, Extractable, Relatable, Taggable, ContentBearing) composed per entity type
- **AI extraction pipeline:** Speech act ontology (Commit, Decide, Risk, Block, Question, Insight, Delegate, Request, Update) with entity resolution and human-in-the-loop review
- **Typed relation graph:** graphqlite-backed relations (BELONGS_TO, RELATES_TO, DELIVERS, SPAWNED_FROM, EXTRACTED_FROM, etc.) queryable via Cypher
- **Portable storage:** Markdown content, JSONL data, graphqlite graph, SQLite+FTS5 index, TOML config
- **Git sync:** Dumb replication — auto-commit, auto-push, shallow history, no branches

## Success Criteria

- Can initialize a workspace, ingest transcripts, and review AI-extracted draft entities via CLI
- Relation graph is queryable via Cypher through the CLI and MCP server
- Reflections can reference and aggregate across programs and time periods
- Workspace syncs reliably across devices via git
- All AI extractions require human review before promotion (human-in-the-loop)

## Principles

1. **Capture is cheap** — Getting information in should be frictionless
2. **Extraction is AI-assisted** — Structured data emerges from unstructured content
3. **Human-in-the-loop** — All AI extractions are drafts requiring review
4. **Relations are first-class** — The graph of connections is as important as the content
5. **Portable format** — Markdown + JSONL + SQLite, no proprietary lock-in
6. **Git as sync** — Not version control, just dumb replication across devices

## Constraints

- **Single-user system** — Multi-device sync but not multi-user collaboration
- **Rust implementation** — All crates in Rust (clotho-core, clotho-graph, clotho-store, clotho-extract, clotho-cli, clotho-mcp, clotho-sync)
- **graphqlite dependency** — Graph layer built on graphqlite (colliery-io/graphqlite)
- **No proprietary formats** — All data must be readable without Clotho installed
- **Git sync model** — Shallow history (~20 commits), main-only, no branches, pull-before-push