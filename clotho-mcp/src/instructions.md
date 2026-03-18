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
| **Responsibility** | Ongoing role obligation that never completes (e.g., "Team Mentorship") | active |
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
| **Artifact** | Deliverable with source link (docs, PRs, presentations) |

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
| **Person** | Someone mentioned in transcripts/notes. Has optional email for matching. |

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

## Processing Log

Track what processes have been run against entities to prevent duplicate extraction.

- `clotho_check_processed` — Has this transcript been extracted already?
- `clotho_mark_processed` — Record that extraction was done

## Tools

### Session
- `clotho_set_workspace(path)` — Set workspace for this session

### Read
- `clotho_search(query, limit?)` — Full-text keyword search
- `clotho_query(cypher)` — Raw Cypher graph query
- `clotho_read_entity(entity_id)` — Read entity metadata + content
- `clotho_list_entities(entity_type?, status?, state?)` — List with filters
- `clotho_get_relations(entity_id)` — Show all relations for entity
- `clotho_get_ontology(entity_id)` — Get extraction ontology
- `clotho_search_ontology(query)` — Search across ontologies
- `clotho_check_processed(entity_id, process_name?)` — Check processing history

### Write
- `clotho_init(path)` — Initialize workspace
- `clotho_capture(file_path, entity_type?, title?)` — Capture a file
- `clotho_create_entity(entity_type, title, status?, state?, email?, parent_id?, content?)` — Create any entity
- `clotho_create_note(title, content)` — Create a note
- `clotho_create_reflection(period, title?, program_id?)` — Create reflection
- `clotho_update_entity(entity_id, title?, status?, state?)` — Update entity
- `clotho_delete_entity(entity_id)` — Delete from all backends
- `clotho_create_relation(source_id, relation_type, target_id)` — Create graph edge
- `clotho_delete_relation(source_id, relation_type, target_id)` — Remove graph edge
- `clotho_update_ontology(entity_id, add_keywords?, ...)` — Update ontology
- `clotho_mark_processed(entity_id, process_name, ...)` — Record processing
- `clotho_sync(prune?, keep?)` — Git sync

## Common Workflows

### Set up work structure
1. Create Programs for your strategic initiatives
2. Create Responsibilities for your ongoing obligations
3. Create Objectives under Programs (use `parent_id`)
4. Create People you work with
5. Set ontologies on each Program (`clotho_update_ontology`)

### Capture materials
1. Use `clotho_capture` for files (transcripts, notes)
2. Use `clotho_create_note` for inline content
3. Use `clotho_create_entity` for any other entity type

### Extract from transcripts
1. Check if already processed: `clotho_check_processed`
2. Load ontologies: `clotho_get_ontology` for relevant programs
3. Read transcript: `clotho_read_entity`
4. Create entities for each speech act found (Decision, Risk, Task, etc.)
5. Create `extracted_from` relation back to transcript
6. Create `mentions` relations for people
7. Create `belongs_to` relations to programs
8. Mark as processed: `clotho_mark_processed`
9. Suggest new ontology keywords if found

### Query work state
- Blocked tasks: `clotho_list_entities(state: "blocked")`
- Active risks: `clotho_list_entities(entity_type: "Risk")`
- What belongs to a program: `clotho_query("MATCH (n)-[:BELONGS_TO]->(p {id: 'X'}) RETURN n")`
- Who is mentioned: `clotho_get_relations(transcript_id)` and filter for MENTIONS
