# Clotho MCP Server

Clotho is a personal work and time management system. It captures the flow of work through meetings, transcripts, notes, and reflections, then extracts structured meaning (decisions, risks, tasks) and connects everything in a queryable graph.

## Workspace

The workspace auto-detects on server startup. If not detected, use `clotho_set_workspace` to set the path. All other tools use the session workspace automatically.

## Domain Model

Clotho organizes work in four layers:

### Structural Layer — "What you do"

| Entity | Purpose | Default Status |
|--------|---------|---------------|
| **Program** | Strategic initiative with objectives (e.g., "Monolith Breakup") | active |
| **Responsibility** | Ongoing role obligation that never completes. Includes work obligations (e.g., "Budget Management"), people/HR obligations (e.g., "Direct Reports", "1:1s"), and practice obligations (e.g., "Team Mentorship", "Hiring"). | active |
| **Objective** | Outcome within a program. Use `parent_id` to link to its Program. | active |

### Execution Layer — "Work in motion"

| Entity | Purpose | Default State |
|--------|---------|--------------|
| **Workstream** | Long-running work thread (e.g., "API Redesign") | active |
| **Task** | Discrete work item. States: `todo → doing → blocked → done` | todo |

### Capture Layer — "Raw material"

| Entity | Purpose |
|--------|---------|
| **Meeting** | Container for a meeting occurrence |
| **Transcript** | Raw meeting content (source for extraction) |
| **Note** | Freeform authored content |
| **Reflection** | Time-period bound thinking (daily/weekly/monthly/quarterly/adhoc) |
| **Artifact** | Deliverable output (docs, PRs, presentations). Use `delivers` relation to link to objectives. |
| **Reference** | External input that informs work (Jira tickets, repos, Confluence pages, Slack threads). Has optional `url` field. |

### Derived Layer — "Sense-making"

Extracted from transcripts by the debrief-processor agent. All start as `draft`.

| Entity | Purpose |
|--------|---------|
| **Decision** | Recorded decision point |
| **Risk** | Identified risk |
| **Blocker** | Impediment to progress |
| **Question** | Open question requiring resolution |
| **Insight** | Learning or observation worth preserving |

### Cross-cutting

| Entity | Purpose |
|--------|---------|
| **Person** | Someone mentioned in transcripts/notes. Has optional `email` field. |

## Relations

Connect entities with typed graph edges:

| Relation | From → To | When to use |
|----------|-----------|-------------|
| `belongs_to` | Task/Objective/Note → Program/Responsibility | Ownership |
| `relates_to` | Any → Workstream/Program | Topical connection |
| `delivers` | Artifact → Task/Objective | Evidence of completion |
| `spawned_from` | Note/Task → Meeting | Origin tracking |
| `extracted_from` | Decision/Risk/etc. → Transcript | Extraction provenance |
| `has_decision` | Meeting → Decision | Meeting produced a decision |
| `has_risk` | Any → Risk | Flags a risk |
| `blocked_by` | Task → Blocker | Task is blocked |
| `mentions` | Transcript/Note → Person/Program | Content references entity |

## Ontology

Each Program/Responsibility has an extraction ontology — keywords, signal types (technical/social), and involved people. This guides transcript extraction.

- `clotho_get_ontology` — Read a program's ontology
- `clotho_update_ontology` — Add keywords, signal types, people
- `clotho_search_ontology` — Find which programs care about a topic

The ontology grows over time. After extraction, suggest new keywords to the user.

## Surfaces — Notes for the User

Surfaces are text blobs you push to the user's TUI. They appear as tabs the user can view and edit inline. **Use surfaces whenever you need to put persistent, visible information in front of the user** — things that shouldn't disappear when the chat scrolls.

### When to use surfaces
- **Daily briefings** — assemble priorities, blocked items, upcoming deadlines
- **Meeting prep / 1:1 notes** — structured agendas the user can mark up during meetings
- **Checklists** — todo items with `[ ]` / `[x]` checkboxes the user toggles
- **Status summaries** — program health, risk overviews, weekly snapshots
- **Any content the user asked you to "put somewhere visible"**

### Surface tools
- `clotho_push_surface(title, content, surface_type?, replace?)` — Create or replace a surface. Use `replace: true` to update an existing surface with the same title.
- `clotho_read_surface(id_or_title)` — Read a surface back, including user edits
- `clotho_list_surfaces(status?, surface_type?, search?)` — List surfaces

### How surfaces work
- Surfaces are stored in the database (persistent, searchable, retrievable)
- They appear in the TUI's "Surfaces" section in the navigator
- The user can edit them inline and save changes
- You can read them back later to see user edits (e.g., "I marked up the briefing")
- Closing a surface tab marks it as closed but doesn't delete it — still searchable

### Surface types
Use `surface_type` to hint the purpose: `briefing`, `meeting-notes`, `checklist`, `freeform`

## Processing Log

Track what processes have been run against entities to prevent duplicate extraction.

- `clotho_check_processed` — Has this transcript been extracted already?
- `clotho_mark_processed` — Record that extraction was done

## Tools

All tools that take entity IDs accept full UUIDs or prefixes (e.g., `f47ac10b`). Ambiguous prefixes return all matches.

### Session
- `clotho_set_workspace(path)` — Set workspace for this session

### Read
- `clotho_workspace_summary()` — High-level overview: entity counts, blocked tasks, unprocessed transcripts, recent activity. **Use this first.**
- `clotho_search(query, limit?)` — Full-text keyword search
- `clotho_query(cypher)` — Raw Cypher graph query
- `clotho_read_entity(entity_id, include_relations?)` — Read entity metadata + content. Set `include_relations: true` to also get graph edges.
- `clotho_list_entities(entity_type?, status?, state?)` — List with filters
- `clotho_list_unprocessed(entity_type?)` — Show transcripts/notes awaiting extraction
- `clotho_get_relations(entity_id)` — Show all relations for entity
- `clotho_get_ontology(entity_id)` — Get extraction ontology
- `clotho_search_ontology(query)` — Search across ontologies
- `clotho_check_processed(entity_id, process_name?)` — Check processing history

### Write
- `clotho_init(path)` — Initialize workspace
- `clotho_capture(file_path, entity_type?, title?)` — Capture a file
- `clotho_capture_directory(path, pattern?, entity_type?)` — Capture all matching files from a directory
- `clotho_create_entity(entity_type, title, status?, state?, email?, url?, parent_id?, content?)` — Create any entity
- `clotho_create_note(title, content, parent_id?)` — Create a note, optionally linked to a parent
- `clotho_create_reflection(period, title?, program_id?)` — Create reflection
- `clotho_update_entity(entity_id, title?, status?, state?, content?, email?, url?)` — Update entity fields or content
- `clotho_delete_entity(entity_id)` — Delete from all backends
- `clotho_create_relation(source_id, relation_type, target_id)` — Create graph edge
- `clotho_batch_create_relations(relations)` — Create multiple relations in one call. Each: `{source_id, relation_type, target_id}`
- `clotho_delete_relation(source_id, relation_type, target_id)` — Remove graph edge
- `clotho_update_ontology(entity_id, add_keywords?, ...)` — Update ontology
- `clotho_mark_processed(entity_id, process_name, ...)` — Record processing
- `clotho_archive_entity(entity_id)` — Archive entity (set inactive, hidden from TUI)
- `clotho_push_surface(title, content, surface_type?, replace?)` — Push text to user's TUI
- `clotho_read_surface(id_or_title)` — Read surface (including user edits)
- `clotho_list_surfaces(status?, surface_type?, search?)` — List surfaces
- `clotho_sync(prune?, keep?)` — Git sync

## Common Workflows

### Set up work structure
1. Create Programs for your strategic initiatives
2. Create Responsibilities for your ongoing obligations
3. Create Objectives under Programs (use `parent_id`)
4. Create People you work with (include `email` for calendar integration)
5. Set ontologies on each Program (`clotho_update_ontology`)

### Capture materials
1. Use `clotho_capture` for single files (transcripts, notes)
2. Use `clotho_capture_directory` for bulk ingestion (e.g., a folder of meeting transcripts)
3. Use `clotho_create_note` for inline content (use `parent_id` to auto-link)
4. Use `clotho_create_entity(type: "reference", url: "...")` for external links (Jira, repos, docs)

### Extract from transcripts
1. Check the queue: `clotho_list_unprocessed`
2. Load ontologies: `clotho_get_ontology` for relevant programs
3. Read transcript: `clotho_read_entity`
4. Create entities for each speech act found (Decision, Risk, Task, etc.)
5. Link everything in one call: `clotho_batch_create_relations`
6. Mark as processed: `clotho_mark_processed` (auto-done by debrief-processor agent)
7. Suggest new ontology keywords if found

### Query work state
- Overview: `clotho_workspace_summary`
- Extraction queue: `clotho_list_unprocessed`
- Blocked tasks: `clotho_list_entities(state: "blocked")`
- Active risks: `clotho_list_entities(entity_type: "Risk")`
- Entity with relations: `clotho_read_entity(entity_id, include_relations: true)`
- What belongs to a program: `clotho_query("MATCH (n)-[:BELONGS_TO]->(p {id: 'X'}) RETURN n")`
