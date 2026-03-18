---
name: clotho-transcript-ingestor
description: "Use when the user says 'process this transcript', 'extract from this meeting', 'ingest transcript', 'here's a transcript', or provides a meeting transcript/notes for extraction. Also triggered when a file is provided with meeting content."
---

# Transcript Ingestor

Single-transcript processing. The user has a transcript or meeting notes — process it, extract signals, and link everything. This is the manual/ad-hoc version of what the daily debrief does in batch.

**You need the workspace_path.** If not known, find the `.clotho/` directory relative to the current working directory.

## Step 1: Ingest the transcript

Determine how the user is providing the content:

**File path:**
```
clotho_ingest(workspace_path, file_path: "<path>", entity_type: "transcript", title: "<meeting name>")
```

**Pasted text:**
```
clotho_create_entity(workspace_path, entity_type: "transcript", title: "<meeting name>", content: "<pasted text>")
```

**Verbal description:**
```
clotho_create_note(workspace_path, title: "<meeting name> - Notes", content: "<what they described>")
```

If the user doesn't provide a title, infer from the content (first line, meeting subject, attendee names + date).

## Step 2: Identify context

Ask (or infer from content):
> "Which program does this meeting relate to?"

Present active programs:
```
clotho_list_entities(workspace_path, entity_type: "Program", status: "active")
```

If the user specifies a program, read its content to understand the extraction lens:
```
clotho_read_entity(workspace_path, entity_id: "<program_id>")
```

If the user says "not sure" or it spans multiple, proceed with general extraction and route later.

## Step 3: Create meeting entity (if not already)

If this was a meeting (not just standalone notes), ensure a Meeting entity exists:
```
clotho_create_entity(workspace_path, entity_type: "meeting", title: "<meeting name>")
```

Link the transcript to the meeting:
```
clotho_create_relation(workspace_path, source_id: "<transcript_id>", relation_type: "spawned_from", target_id: "<meeting_id>")
```

## Step 4: Launch extraction

Launch the **debrief-processor** agent with:
- The single transcript/note entity ID
- The program context (if identified)
- The workspace_path

The agent will:
- Read program context to understand what signals matter
- Extract speech acts and domain-specific signals
- Create entities with EXTRACTED_FROM relations
- Identify and link people
- Route to programs via BELONGS_TO

## Step 5: Review

Present the agent's extraction summary. Ask:
> "Does this capture everything? Anything to add or adjust?"

Handle corrections, then:
```
clotho_sync(workspace_path)
```

## Idempotency

If the user asks to process a transcript that's already been extracted (has EXTRACTED_FROM relations pointing to it), warn:
> "This transcript has already been processed. N entities were extracted from it. Want to re-extract (may create duplicates) or skip?"

Check with:
```
clotho_get_relations(workspace_path, entity_id: "<transcript_id>")
```
Look for incoming EXTRACTED_FROM relations.
