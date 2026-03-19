---
name: clotho-daily-brief
description: "Use when the user says 'morning', 'daily brief', 'what should I focus on', 'start of day', 'what's on today', or wants a prioritized view of today's work."
---

# Daily Brief

Start-of-day ceremony. No agent needed — this is a pure query + synthesis skill. Present the user with a prioritized view of what needs attention today.

The workspace is set automatically. Use `clotho_set_workspace` if needed.

## Step 1: Quick check-in

Ask:
> "Good morning. Anything new before we look at the day?"

Handle any quick input, then proceed.

## Step 2: Gather state (all automated queries)

Run these queries in parallel:

### Inbox
```
Use Bash to: ls -la <workspace_path>/.clotho/inbox/ 2>/dev/null
```

### Blocked tasks
```
clotho_list_entities(state: "blocked")
```
For each blocked task, get what's blocking it:
```
clotho_get_relations(entity_id: "<blocked_task_id>")
```

### Active tasks
```
clotho_list_entities(state: "doing")
```

### Todo tasks (check for stale)
```
clotho_list_entities(state: "todo")
```

### Open questions and active risks
```
clotho_list_entities(entity_type: "Question")
clotho_list_entities(entity_type: "Risk")
```

## Step 3: Synthesize and present

Present in this priority order:

### 1. Inbox Alert (if items)
> "You have N unprocessed items in your inbox."

### 2. Blocked Items (highest urgency)
> "**Blocked:**
> - [task title] — blocked by: [blocker title]
> - ..."

### 3. Due Today / This Week
If deadline data is available in entity metadata, surface items due today and this week.

### 4. In Progress
> "**In progress:**
> - [task title]
> - ..."

### 5. Due This Week (horizon from last debrief)
Surface items with upcoming deadlines.

### 6. Open Questions / Active Risks
> "**Needs attention:**
> - [Question] How do we handle session store migration?
> - [Risk] Database migration may cause downtime"

### 7. Stale Items
Flag todo tasks created more than 7 days ago:
> "**Stale (>7 days in todo):**
> - [task title] (created N days ago)"

## Done

End with:
> "That's your day. What do you want to tackle first?"

No sync needed — this is read-only.
