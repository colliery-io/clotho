---
name: clotho-weekly-review
description: "Use when the user says 'weekly review', 'weekly reflection', 'review my week', 'end of week', 'retrospective', or wants to reflect on the past week's work."
---

# Weekly Review

End-of-week ceremony. You gather the week's data, guide the user through reflection questions, then launch the review-compiler agent to identify patterns and create a Reflection entity.

The workspace is set automatically. Use `clotho_set_workspace` if needed.

## Step 1: Gather the week's data

Run these queries to build the week summary:

```
clotho_list_entities(state: "done")
```
Filter to updated_at within this week → **completed tasks**

```
clotho_list_entities(state: "doing")
```
Filter to created_at this week → **newly started tasks**

```
clotho_list_entities(state: "blocked")
```
→ **still blocked**

```
clotho_list_entities(entity_type: "Decision")
```
Filter to created_at this week → **decisions made**

```
clotho_list_entities(entity_type: "Risk")
```
Filter to created_at this week → **new risks**

```
clotho_list_entities(entity_type: "Meeting")
```
Filter to created_at this week → **meetings held**

## Step 2: Present week summary

> **This Week in Review**
>
> **Completed:** N tasks
> - [list titles]
>
> **Started:** M tasks
> - [list titles]
>
> **Still blocked:** K tasks
> - [list with what's blocking them]
>
> **Decisions made:** J
> - [list titles]
>
> **New risks:** L
> - [list titles]
>
> **Meetings:** P
> - [list titles]

## Step 3: Guided reflection

Ask these questions one at a time. Wait for the user to respond to each before moving on.

1. > "**What went well this week?** Any wins, breakthroughs, or things that clicked?"

2. > "**What was harder than expected?** Anything that took more effort or time than planned?"

3. > "**Do you see any patterns?** Recurring blockers, programs that are consistently behind, themes across meetings?"

4. > "**Any programs falling behind?** Based on what we see, anything you're concerned about?"

Capture the user's responses — these become part of the reflection content.

## Step 4: Consolidation pass

Launch the **entity-consolidator** agent to clean up the workspace:
- Find and merge duplicate risks, blockers, decisions, questions
- Suggest archival of done tasks, resolved blockers, answered questions
- Create summary notes for clusters of related items

Present the consolidation plan and wait for user approval before executing.

## Step 5: Launch review agent

Launch the **review-compiler** agent with:
- The week's data summary
- The user's reflection responses
- Results of the consolidation pass

The agent will:
- Analyze patterns across programs
- Identify problem areas (growing blockers, stalled workstreams, aging risks)
- Create a Reflection entity with structured content
- Link it to relevant programs

## Step 6: Review and forward focus

After the agent completes, present its findings. Then ask:

> "**What should next week focus on?** Given what we see, what are your top priorities?"

Capture this as part of the reflection content. Update the Reflection entity if needed.

End with:
> "Weekly reflection captured. Workspace cleaned up. Have a good weekend."

Sync: `clotho_sync()`
