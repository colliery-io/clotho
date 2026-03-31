---
name: clotho-workspace
description: "Use when the user asks to 'init clotho', 'create a workspace', 'capture a file', 'create a program', 'create a task', 'add a person', 'list entities', 'delete entity', 'update entity', or any entity CRUD operation in a Clotho workspace."
---

# Clotho Workspace Management

## Initialize a Workspace

```
clotho_init(path: "/path/to/project")
```

This creates `.clotho/` with: content/, data/, graph/, index/, config/.

## Create Entities

Use `clotho_create_entity` for all 15 entity types:

### Structural Layer — "What you do"
```
clotho_create_entity(entity_type: "program", title: "Technical Education")
clotho_create_entity(entity_type: "responsibility", title: "Team Mentorship")
clotho_create_entity(entity_type: "objective", title: "Reduce deploy time", parent_id: "<program_id>")
```

### Execution Layer — "Work in motion"
```
clotho_create_entity(entity_type: "workstream", title: "API Redesign")
clotho_create_entity(entity_type: "task", title: "Write RFC")
```

### Capture Layer — "Raw material"
```
clotho_create_note(title: "Meeting Notes", content: "# Notes\n...")
clotho_create_reflection(period: "weekly")
clotho_capture(file_path: "/path/to/transcript.md", entity_type: "transcript")
```

### People
```
clotho_create_entity(entity_type: "person", title: "Alice", email: "alice@example.com")
```

## Modeling Responsibilities and Direct Reports

Responsibilities represent ongoing obligations that never complete. They serve as organizational anchors in the navigator — entities linked to a responsibility via `belongs_to` appear nested under it.

### Common responsibility patterns

**Direct Reports** — Create a responsibility for managing direct reports. Link 1:1 meeting notes, performance notes, and related tasks to it:
```
clotho_create_entity(entity_type: "responsibility", title: "Direct Reports")
clotho_create_note(title: "1:1 with Alice - 2026-03-28", content: "...", parent_id: "<direct_reports_id>")
clotho_create_entity(entity_type: "task", title: "Write Alice's performance review", parent_id: "<direct_reports_id>")
```

This produces a navigator tree like:
```
▾ Responsibilities
  ▾ Direct Reports
    1:1 with Alice - 2026-03-28
    1:1 with Bob - 2026-04-01
    Write Alice's performance review
```

**Per-person responsibilities** — For heavy management load, create a responsibility per person:
```
clotho_create_entity(entity_type: "responsibility", title: "1:1s — Alice Chen")
clotho_create_entity(entity_type: "responsibility", title: "1:1s — Bob Martinez")
```

**Other common responsibilities:**
- "Hiring" — interview notes, pipeline tasks, hiring decisions
- "Budget Management" — cost reports, approval tasks, financial decisions
- "On-Call" — incident notes, runbook tasks, escalation decisions
- "Team Ceremonies" — sprint retros, planning notes, team health tasks

### Linking people to responsibilities
People entities are separate from responsibilities. To connect them:
```
clotho_create_relation(source_id: "<person_id>", relation_type: "relates_to", target_id: "<responsibility_id>")
```

This doesn't nest them in the navigator (only `belongs_to` does that) but makes the relationship queryable in the graph.

### Derived Layer — "Sense-making" (from extraction)
```
clotho_create_entity(entity_type: "decision", title: "Go with option B")
clotho_create_entity(entity_type: "risk", title: "Rate limiting at scale")
clotho_create_entity(entity_type: "blocker", title: "Shared database coupling")
clotho_create_entity(entity_type: "question", title: "How to handle session store?")
clotho_create_entity(entity_type: "insight", title: "Strangler fig pattern works well")
```

## Default States

| Layer | Default Status/State |
|-------|---------------------|
| Structural (Program, Responsibility, Objective, Workstream) | `active` |
| Task | `todo` |
| Derived (Decision, Risk, Blocker, Question, Insight) | `draft` |
| Capture + Person | none |

## Entity CRUD

```
clotho_read_entity(entity_id: "<uuid>")
clotho_update_entity(entity_id: "<uuid>", title: "New Title", status: "inactive")
clotho_delete_entity(entity_id: "<uuid>")
clotho_list_entities(entity_type: "Task", state: "doing")
clotho_search(query: "deployment strategy")
```

## Sync

After making changes, sync to git:
```
clotho_sync(prune: true)
```
