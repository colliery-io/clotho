# Clotho

<div align="center">
  <strong>Personal work and time management through reflection, transcripts, and sense-making.</strong>
  <br><br>
  Spin the raw threads of meetings, notes, and artifacts into a coherent narrative of your work.
</div>

## Install

Requires Rust toolchain ([rustup.rs](https://rustup.rs)) and [Claude Code](https://docs.anthropic.com/en/docs/claude-code).

```bash
curl -fsSL https://raw.githubusercontent.com/colliery-io/clotho/main/scripts/install.sh | sh
```

This will:
1. Build and install `clotho` and `clotho-mcp` to `~/.local/bin/`
2. Initialize a workspace at `~/.clotho/`
3. Install the Clotho Claude Code plugin (MCP server, skills, hooks, agents)

## Quick Start

Just run:

```bash
clotho
```

This launches the Clotho TUI — a three-panel terminal interface with:
- **Entities panel** (left) — browse your entities by type
- **Content panel** (top-right) — tabbed viewer/editor for entities and surfaces
- **Chat panel** (bottom-right) — embedded Claude session with full MCP access

Claude can push surfaces (daily briefings, meeting notes, checklists) directly to your TUI. You edit them inline, and Claude can read your edits back.

### Keybindings

| Key | Action |
|-----|--------|
| `Ctrl+Tab` | Switch panel focus |
| `Ctrl+C` / `Ctrl+Q` | Quit (Ctrl+C forwards to Claude when chat is focused) |
| `?` | Show help overlay |

**Entities panel:** `j/k` move, `Enter` open, `</>`  resize panel

**Content panel (command mode):** `h/l` switch tabs, `j/k` scroll, `i` edit, `w` close tab, `x` toggle checkbox, `s` save, `g/G` top/bottom

**Content panel (edit mode):** type normally, `Esc` exit to command mode, `Ctrl+S` save

## TUI + Surfaces

The TUI is a persistent surface for things that don't belong in a chat stream:

- **Daily briefings** — Claude assembles it, pushes to your TUI, you check things off
- **Meeting prep / 1:1 notes** — structured notes you can mark up during meetings
- **Active todo lists** — Claude adds items as they come up, you toggle them done
- **Status updates** — quick summaries pinned for reference

Surfaces are stored in the database — searchable and retrievable. Close a tab when you're done; the content persists for later reference.

```
clotho tui                    # Launch TUI (default workspace ~/.clotho)
clotho tui -w /path/to/.clotho  # Custom workspace
clotho tui -- -c              # Pass -c to Claude (continue last session)
clotho tui -- -r              # Pass -r to Claude (resume)
```

## CLI

All entity management is also available via CLI subcommands:

```bash
clotho create program --title "Monolith Breakup"
clotho create task --title "Write migration RFC"
clotho create person --title "Alice" --email "alice@example.com"
clotho list --type Task
clotho search "migration strategy"
clotho get <entity-id>
clotho relate <task-id> belongs_to <program-id>
clotho query "MATCH (t:Task)-[:BLOCKED_BY]->(b:Blocker) RETURN t.title, b.title"
clotho reflect --period weekly
clotho sync
```

## Claude Code Plugin

The plugin is installed automatically by the install script. It includes:

### Skills

| Skill | Purpose |
|-------|---------|
| `workspace-management` | Entity CRUD, workspace management |
| `graph-queries` | Relations, Cypher queries |
| `extraction` | In-session speech act extraction |
| `reflection` | Guided reflection creation |
| `transcript-ingestor` | Single transcript processing |

### Ceremonies

| Command | When | What it does |
|---------|------|-------------|
| `/daily-debrief` | End of day | Scans inbox, ingests materials, updates tasks, extracts from transcripts |
| `/daily-brief` | Start of day | Prioritized view: blocked items, due dates, stale tasks, open risks |
| `/weekly-review` | End of week | Guided reflection, pattern identification, problem areas |
| `/report` | As needed | Audience-appropriate status reports |
| `/period-review` | Quarterly+ | Deep retrospective with decision outcome tracking |

### MCP Tools (28)

**Session:** `clotho_set_workspace`

**Read:** `clotho_search`, `clotho_query`, `clotho_read_entity`, `clotho_list_entities`, `clotho_get_relations`, `clotho_workspace_summary`, `clotho_list_unprocessed`, `clotho_get_ontology`, `clotho_search_ontology`, `clotho_check_processed`, `clotho_read_surface`, `clotho_list_surfaces`

**Write:** `clotho_init`, `clotho_capture`, `clotho_capture_directory`, `clotho_create_entity`, `clotho_update_entity`, `clotho_delete_entity`, `clotho_create_note`, `clotho_create_reflection`, `clotho_create_relation`, `clotho_batch_create_relations`, `clotho_delete_relation`, `clotho_update_ontology`, `clotho_mark_processed`, `clotho_sync`, `clotho_push_surface`

## Entity Types

| Layer | Entities | Lifecycle |
|-------|----------|-----------|
| **Structural** | Program, Responsibility, Objective | active / inactive |
| **Execution** | Workstream, Task | Task: todo -> doing -> blocked -> done |
| **Capture** | Meeting, Transcript, Note, Reflection, Artifact, Reference | — |
| **Derived** | Decision, Risk, Blocker, Question, Insight | draft (from extraction) |
| **Cross-cutting** | Person | — |

## Relation Types

| Relation | Meaning | Example |
|----------|---------|---------|
| `belongs_to` | Ownership | Task -> Program |
| `relates_to` | Topical connection | Workstream -> Program |
| `delivers` | Evidence of completion | Artifact -> Objective |
| `spawned_from` | Origin | Note -> Meeting |
| `extracted_from` | Extraction provenance | Decision -> Transcript |
| `has_decision` | Contains | Meeting -> Decision |
| `has_risk` | Flags | Program -> Risk |
| `blocked_by` | Impediment | Task -> Blocker |
| `mentions` | Reference | Transcript -> Person |

## Workspace Layout

```
~/.clotho/                        # Workspace root
├── content/                      # Entity content files
│   ├── programs/
│   ├── responsibilities/
│   ├── objectives/
│   ├── workstreams/
│   ├── tasks/
│   ├── meetings/
│   ├── reflections/
│   ├── artifacts/
│   ├── references/
│   ├── notes/
│   ├── people/
│   └── derived/
├── data/                         # entities.db, events, tags
├── graph/                        # relations.db (graphqlite)
├── index/                        # search.db (FTS5)
├── inbox/                        # Landing zone for integrations
├── config/                       # config.toml, ontology.toml
└── tui-state.json                # TUI display state (tabs, scroll)
```

## Architecture

```
clotho/
├── clotho-core      # Domain model, traits, graph (graphqlite)
├── clotho-store     # Storage: SQLite, JSONL, FTS5, content, migrations
├── clotho-tui       # Terminal UI: ratatui, embedded PTY, surfaces
├── clotho-cli       # CLI commands + TUI subcommand
├── clotho-mcp       # MCP server (rust-mcp-sdk)
├── clotho-sync      # Git sync (libgit2)
├── clotho-tests     # E2E integration tests
└── plugins/clotho   # Claude Code plugin (skills, agents, hooks)
```

## Development

```bash
# Build
cargo build --workspace

# Test
cargo test --workspace

# Install locally from source
scripts/install.sh --local
```

## Migrating from an existing workspace

If you have an existing Clotho workspace (e.g., `~/Desktop/my-work/.clotho`):

```bash
# Initialize the new default workspace
clotho init --path ~

# Copy content files
cp -r ~/Desktop/my-work/notes/* ~/.clotho/content/notes/
cp -r ~/Desktop/my-work/tasks/* ~/.clotho/content/tasks/
# ... etc for each content dir

# Copy databases
cp ~/Desktop/my-work/.clotho/data/entities.db ~/.clotho/data/
cp ~/Desktop/my-work/.clotho/graph/relations.db ~/.clotho/graph/
```

## License

Apache 2.0
