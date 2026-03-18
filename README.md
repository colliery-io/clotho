# Clotho

<div align="center">
  <strong>Personal work and time management through reflection, transcripts, and sense-making.</strong>
  <br><br>
  Spin the raw threads of meetings, notes, and artifacts into a coherent narrative of your work.
</div>

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/colliery-io/clotho/main/scripts/install.sh | sh
```

This installs two binaries to `~/.local/bin/`:
- `clotho` — CLI for workspace management
- `clotho-mcp` — MCP server for AI agent integration (Claude Code plugin)

## Quick Start

### 1. Initialize a workspace

```bash
mkdir my-work && cd my-work
clotho init
```

This creates:
- Visible content directories at the project root: `programs/`, `responsibilities/`, `tasks/`, `meetings/`, `notes/`, etc.
- Hidden machine data in `.clotho/`: databases, graph, search index, config

### 2. Set up your work structure

Create the programs and responsibilities that define your work:

```bash
clotho create program --title "Monolith Breakup"
clotho create program --title "Technical Education"
clotho create responsibility --title "Team Mentorship"
clotho create responsibility --title "Budget Management"
```

Create objectives under programs:

```bash
clotho create objective --title "Extract user service" --parent <program-id>
```

Create people you work with:

```bash
clotho create person --title "Alice" --email "alice@example.com"
clotho create person --title "Bob"
```

### 3. Configure extraction ontologies

Each program has an ontology — keywords and signal types that guide AI extraction from transcripts:

```bash
clotho ontology-set <program-id> \
  --add-keywords "database coupling, service contracts, strangler fig" \
  --add-technical "architecture coupling, missing service layer" \
  --add-social "team autonomy, ownership gaps" \
  --add-people "Ali K, Harrison"
```

### 4. Capture your work

**Ingest a transcript:**
```bash
clotho capture meeting-transcript.md --type transcript --title "Architecture Review"
```

**Create a note:**
```bash
clotho create note --title "API design thoughts" --content "# API Design\n\nThinking about..."
```

**Create tasks:**
```bash
clotho create task --title "Write migration RFC"
clotho create task --title "Review API contracts"
```

### 5. Connect everything with relations

```bash
# Task belongs to a program
clotho relate <task-id> belongs_to <program-id>

# Transcript mentions a person
clotho relate <transcript-id> mentions <person-id>

# Artifact delivers against an objective
clotho relate <artifact-id> delivers <objective-id>

# Task is blocked by a blocker
clotho relate <task-id> blocked_by <blocker-id>
```

### 6. Query your work

```bash
# List all tasks
clotho list --type Task

# List blocked tasks
clotho list --state blocked

# Search across all content
clotho search "migration strategy"

# Cypher graph query
clotho query "MATCH (t:Task)-[:BLOCKED_BY]->(b:Blocker) RETURN t.title, b.title"

# View an entity
clotho get <entity-id>

# View relations
clotho relations <entity-id>
```

### 7. Reflect and sync

```bash
# Create a weekly reflection
clotho reflect --period weekly

# Sync to git
clotho sync
```

## Claude Code Plugin

Clotho includes a Claude Code plugin with ceremony-driven workflows:

### Install the plugin
```bash
claude plugin add /path/to/clotho/plugins/clotho
```

### Available ceremonies

| Command | When | What it does |
|---------|------|-------------|
| `/daily-debrief` | End of day | Scans inbox, ingests materials, updates tasks, checks horizon, extracts from transcripts |
| `/daily-brief` | Start of day | Prioritized view: blocked items, due dates, stale tasks, open risks |
| `/weekly-review` | End of week | Guided reflection, pattern identification, problem areas |
| `/report` | As needed | Audience-appropriate status reports (boss/stakeholders/team) |
| `/period-review` | Quarterly+ | Deep retrospective with decision outcome tracking |

### Available skills

| Skill | Purpose |
|-------|---------|
| `clotho-workspace` | Entity CRUD, workspace management |
| `clotho-graph` | Relations, Cypher queries |
| `clotho-extraction` | In-session speech act extraction |
| `clotho-reflection` | Guided reflection creation |
| `clotho-transcript-ingestor` | Single transcript processing |

### 18 MCP Tools

**Read:** clotho_search, clotho_query, clotho_read_entity, clotho_list_entities, clotho_get_relations, clotho_get_ontology, clotho_search_ontology

**Write:** clotho_init, clotho_capture, clotho_create_entity, clotho_update_entity, clotho_delete_entity, clotho_create_note, clotho_create_reflection, clotho_create_relation, clotho_delete_relation, clotho_update_ontology, clotho_sync

## Filesystem Layout

```
my-work/                          # Project root
├── programs/                     # Visible — your strategic initiatives
│   └── monolith-breakup.md
├── responsibilities/             # Visible — your ongoing obligations
│   └── team-mentorship.md
├── objectives/                   # Visible — outcomes within programs
├── workstreams/                  # Visible — long-running work threads
├── tasks/                        # Visible — discrete work items
├── meetings/                     # Visible — meeting notes + transcripts
├── reflections/                  # Visible — time-period reflections
├── artifacts/                    # Visible — deliverables
├── notes/                        # Visible — freeform content
├── people/                       # Visible — your contacts
├── derived/                      # Visible — decisions, risks, blockers, etc.
│
└── .clotho/                      # Hidden — machine-managed
    ├── data/                     # entities.db, extractions.db, tags, events
    ├── graph/                    # relations.db (graphqlite)
    ├── index/                    # search.db (FTS5, gitignored)
    ├── inbox/                    # Landing zone for external integrations
    └── config/                   # config.toml, ontology.toml
```

Content is browsable in any editor. Open `programs/` to see your portfolio. Open `tasks/` to see your work queue. Open `reflections/` to review your thinking over time.

## Entity Types

| Layer | Entities | Lifecycle |
|-------|----------|-----------|
| **Structural** | Program, Responsibility, Objective | active / inactive |
| **Execution** | Workstream, Task | Task: todo → doing → blocked → done |
| **Capture** | Meeting, Transcript, Note, Reflection, Artifact | — |
| **Derived** | Decision, Risk, Blocker, Question, Insight | draft (from extraction) |
| **Cross-cutting** | Person | — |

## Relation Types

| Relation | Meaning | Example |
|----------|---------|---------|
| `belongs_to` | Ownership | Task → Program |
| `relates_to` | Topical connection | Workstream → Program |
| `delivers` | Evidence of completion | Artifact → Objective |
| `spawned_from` | Origin | Note → Meeting |
| `extracted_from` | Extraction provenance | Decision → Transcript |
| `has_decision` | Contains | Meeting → Decision |
| `has_risk` | Flags | Program → Risk |
| `blocked_by` | Impediment | Task → Blocker |
| `mentions` | Reference | Transcript → Person |

## Architecture

```
clotho/
├── clotho-core      # Domain model, traits, graph (graphqlite)
├── clotho-store     # Storage: SQLite, JSONL, FTS5, content, federation
├── clotho-cli       # 19 CLI commands
├── clotho-mcp       # 18 MCP tools (rust-mcp-sdk)
├── clotho-sync      # Git sync (libgit2)
├── clotho-tests     # E2E integration tests
└── plugins/clotho   # Claude Code plugin (skills, agents, hooks)
```

## License

Apache 2.0
