---
name: clotho-report
description: "Use when the user says 'status report', 'report for my boss', 'program update', 'stakeholder update', 'team status', or wants to generate a formatted report for a program."
---

# Report Builder

On-demand ceremony for generating audience-appropriate status reports. You gather scope from the user, then launch the report-builder agent.

**You need the workspace_path.** If not known, find the `.clotho/` directory relative to the current working directory.

## Step 1: Which program(s)?

Query active programs:
```
clotho_list_entities(workspace_path, entity_type: "Program", status: "active")
```

Present:
> "Which program(s) do you want to report on?"
> 1. [Program title]
> 2. [Program title]
> 3. All active programs

## Step 2: Time period?

Ask:
> "What time period should this cover?"
> - Last week
> - Last 2 weeks
> - Last month
> - Custom range

## Step 3: Audience?

Ask:
> "Who is this report for?"
> - **Boss** — executive summary, outcomes-focused, short
> - **Stakeholders** — balanced, progress + risks + timeline impacts
> - **Team** — detailed, task-level, who's doing what
> - **Custom** — tell me what you need

## Step 4: Highlights?

Ask:
> "Anything specific to highlight or downplay? Wins to emphasize? Sensitive items to handle carefully?"

## Step 5: Launch agent

Launch the **report-builder** agent with:
- Selected program(s)
- Time period
- Audience type
- Highlight/downplay guidance
- workspace_path

## Step 6: Review

Present the agent's report output. Ask:
> "Here's the draft report. Want to adjust anything before we save it?"

Handle refinements. The agent creates an Artifact entity — the user can iterate on the content.

Sync: `clotho_sync(workspace_path)`
