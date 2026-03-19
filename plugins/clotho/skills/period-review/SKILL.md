---
name: clotho-period-review
description: "Use when the user says 'quarterly review', 'annual review', 'half-year review', 'period review', 'self-assessment', 'performance review', 'QBR', or wants a comprehensive review of a longer time period."
---

# Period Review

Quarterly/half-year/annual ceremony. Deep retrospective analysis for business reviews, self-assessments, and performance reviews. You gather scope, launch the period-compiler agent for analysis, then collaborate extensively with the user on the narrative.

The workspace is set automatically. Use `clotho_set_workspace` if needed.

## Step 1: What period?

Ask:
> "What time period are we reviewing?"
> - Q1 / Q2 / Q3 / Q4 (this year)
> - H1 / H2 (this year)
> - Full year
> - Custom date range

## Step 2: Which programs?

Query active programs:
```
clotho_list_entities(entity_type: "Program")
```

Ask:
> "Which programs should this cover?"
> 1. All programs
> 2. [list each program for selection]

## Step 3: Focus areas?

Ask:
> "What should this review focus on?"
> - **Accomplishments** — what was achieved
> - **Challenges** — what was hard, what didn't work
> - **Growth** — what was learned, how you developed
> - **All of the above** (recommended for self-assessments)

## Step 4: Context?

Ask:
> "What's this review for?"
> - **QBR** — quarterly business review for leadership
> - **Self-assessment** — personal performance review
> - **Performance review** — for manager/HR
> - **Stakeholder update** — broader audience
> - **Personal reflection** — just for you

This affects tone and framing. A QBR focuses on business outcomes. A self-assessment highlights personal contributions and growth.

## Step 5: Launch agent

Launch the **period-compiler** agent with:
- Period date range
- Selected program IDs
- Focus areas
- Context/purpose

## Step 6: Collaborative narrative

The period compiler returns a structured draft. This is NOT a "review and approve" — this is a **collaboration**. The user should actively shape the narrative.

Present each section and ask:
> "Here's the **Accomplishments** section. Want to add, adjust, or reframe anything?"

Then:
> "Here's the **Decisions & Outcomes** section. Any decisions where the outcome was different than expected?"

Then:
> "Here's the **Challenges** section. Anything you'd frame differently?"

## Step 7: Personal reflection

Ask:
> "**What would you do differently?** Looking back on this period, any approaches you'd change?"

> "**What should next period focus on?** Given where things stand, what are your top priorities going forward?"

Incorporate responses into the reflection.

## Step 8: Finalize

Update the Reflection entity with the collaboratively refined content.

Sync: `clotho_sync()`

> "Period review captured. This covers [period] across [N programs]."
