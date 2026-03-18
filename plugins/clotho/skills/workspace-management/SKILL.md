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
clotho_create_entity(workspace_path, entity_type: "program", title: "Technical Education")
clotho_create_entity(workspace_path, entity_type: "responsibility", title: "Team Mentorship")
clotho_create_entity(workspace_path, entity_type: "objective", title: "Reduce deploy time", parent_id: "<program_id>")
```

### Execution Layer — "Work in motion"
```
clotho_create_entity(workspace_path, entity_type: "workstream", title: "API Redesign")
clotho_create_entity(workspace_path, entity_type: "task", title: "Write RFC")
```

### Capture Layer — "Raw material"
```
clotho_create_note(workspace_path, title: "Meeting Notes", content: "# Notes\n...")
clotho_create_reflection(workspace_path, period: "weekly")
clotho_capture(workspace_path, file_path: "/path/to/transcript.md", entity_type: "transcript")
```

### People
```
clotho_create_entity(workspace_path, entity_type: "person", title: "Alice", email: "alice@example.com")
```

### Derived Layer — "Sense-making" (from extraction)
```
clotho_create_entity(workspace_path, entity_type: "decision", title: "Go with option B")
clotho_create_entity(workspace_path, entity_type: "risk", title: "Rate limiting at scale")
clotho_create_entity(workspace_path, entity_type: "blocker", title: "Shared database coupling")
clotho_create_entity(workspace_path, entity_type: "question", title: "How to handle session store?")
clotho_create_entity(workspace_path, entity_type: "insight", title: "Strangler fig pattern works well")
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
clotho_read_entity(workspace_path, entity_id: "<uuid>")
clotho_update_entity(workspace_path, entity_id: "<uuid>", title: "New Title", status: "inactive")
clotho_delete_entity(workspace_path, entity_id: "<uuid>")
clotho_list_entities(workspace_path, entity_type: "Task", state: "doing")
clotho_search(workspace_path, query: "deployment strategy")
```

## Sync

After making changes, sync to git:
```
clotho_sync(workspace_path, prune: true)
```
