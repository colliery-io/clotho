---
name: clotho-daily-debrief
description: "Use when the user says 'end of day', 'daily debrief', 'process today', 'what happened today', 'dump my day', 'debrief', or wants to capture everything from today into Clotho."
---

# Daily Debrief

End-of-day ceremony. Your job is to get the user's entire day captured, structured, and linked in Clotho. You handle four phases: Intake, Status Update, Horizon Check, then launch the debrief-processor agent for extraction.

**You need the workspace_path.** If not known, find the `.clotho/` directory relative to the current working directory.

## Phase 1: Intake

### Step 1: Check what's already here

1. Scan `.clotho/inbox/` for unprocessed files:
   ```
   Use Bash to: ls -la <workspace_path>/.clotho/inbox/
   ```
   If files exist, report what's there.

2. Query entities created today:
   ```
   clotho_list_entities(workspace_path, entity_type: "Meeting")
   clotho_list_entities(workspace_path, entity_type: "Transcript")
   clotho_list_entities(workspace_path, entity_type: "Note")
   ```
   Filter results to items with today's date in created_at.

3. Present to user:
   > "Here's what I already have from today: [list meetings/transcripts/notes]. Your inbox has N unprocessed files."

### Step 2: Ingest inbox

For each file in `.clotho/inbox/`:
- Infer type from filename/extension (.transcript.md → transcript, .md → note, .ics → meeting)
- Ingest: `clotho_ingest(workspace_path, file_path, entity_type, title)`
- After ingesting, move the file out of inbox (or note it's been processed)

### Step 3: Gather additional materials

Ask the user:
> "Were there any other meetings or conversations today not captured above? You can:
> - Paste a transcript or meeting summary
> - Point me to a file
> - Just tell me what happened and I'll capture it as a note"

For each material provided:
- Pasted text → `clotho_create_note(workspace_path, title, content)` or `clotho_create_entity(workspace_path, entity_type: "transcript", title, content)`
- File reference → `clotho_ingest(workspace_path, file_path, entity_type)`
- Verbal description → `clotho_create_note(workspace_path, title: "EOD note: <topic>", content: <what they said>)`

Keep asking until the user says they're done.

## Phase 2: Status Update

### Step 4: Task review

1. Query active and pending tasks:
   ```
   clotho_list_entities(workspace_path, state: "doing")
   clotho_list_entities(workspace_path, state: "todo")
   ```

2. Present as a bulk list:
   > "Here are your active/pending tasks. What moved today?"
   >
   > **Doing:**
   > - [title] (id)
   > - [title] (id)
   >
   > **Todo:**
   > - [title] (id)
   >
   > Tell me what changed: completed, blocked, started, or no change.

3. For each update the user provides:
   - Completed → `clotho_update_entity(workspace_path, entity_id, state: "done")`
   - Blocked → `clotho_update_entity(workspace_path, entity_id, state: "blocked")` + ask what's blocking, create Blocker entity + BLOCKED_BY relation
   - Started → `clotho_update_entity(workspace_path, entity_id, state: "doing")`

### Step 5: Ad-hoc items

Ask:
> "Any decisions, risks, or blockers from outside meetings today?"

For each:
- Decision → `clotho_create_entity(workspace_path, entity_type: "decision", title)`
- Risk → `clotho_create_entity(workspace_path, entity_type: "risk", title)`
- Blocker → `clotho_create_entity(workspace_path, entity_type: "blocker", title)`

Link to relevant programs if the user specifies context.

## Phase 3: Horizon Check

### Step 6: Look ahead

1. Query tasks with upcoming deadlines:
   ```
   clotho_query(workspace_path, cypher: "MATCH (t:Task) RETURN t.id, t.title, t.entity_type")
   ```
   Check metadata for deadline fields. Also:
   ```
   clotho_list_entities(workspace_path, state: "todo")
   ```
   Check created_at — flag any todo tasks older than 7 days as stale.

2. Check program health:
   ```
   clotho_list_entities(workspace_path, entity_type: "Program")
   ```
   For each program, check if it has any tasks updated in the last 2 weeks. Flag programs with no recent activity.

3. Present:
   > "**Looking ahead:**
   > - N items due this week
   > - M items due next week
   > - K tasks have been in todo for >7 days: [list]
   > - Program X has had no task activity in 2 weeks
   >
   > Want to reprioritize anything or flag something?"

Handle any reprioritization the user requests.

## Phase 4: Extract & Wrap

### Step 7: Launch agent

Launch the **debrief-processor** agent. Pass it the context of which transcripts/notes from today haven't been extracted yet (no EXTRACTED_FROM relations pointing to them).

The agent will:
- Read transcript/note content
- Identify speech acts (decisions, risks, tasks, blockers, questions, insights)
- Create derived entities with EXTRACTED_FROM relations
- Identify and link people (MENTIONS)
- Suggest BELONGS_TO relations to programs

### Step 8: Review and sync

After the agent completes:
1. Present the extraction summary
2. Ask: "Does this look right? Anything to adjust?"
3. Handle any corrections
4. Sync: `clotho_sync(workspace_path)`
5. Confirm: "Today is captured. Good night."
