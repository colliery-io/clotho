---
id: cadence-driven-agent-ceremonies
level: initiative
title: "Cadence-Driven Agent Ceremonies"
short_code: "CLO-I-0010"
created_at: 2026-03-17T15:16:33.187757+00:00
updated_at: 2026-03-17T15:16:33.187757+00:00
parent: CLO-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
initiative_id: cadence-driven-agent-ceremonies
---

# Cadence-Driven Agent Ceremonies

## Context

Clotho's interaction model is cadence-driven — users interact at natural time rhythms, not ad-hoc moments. Each ceremony has a skill (interactive entry point that gathers context and asks questions) and optionally an agent (autonomous processor that does the heavy multi-step work).

The daily debrief is the primary ingestion point. Users dump their day — transcripts, notes, status updates, verbal descriptions — and the ceremony structures it all. The inbox (`.clotho/inbox/`, from CLO-I-0009) serves as the accumulation point for materials between ceremonies.

Depends on: CLO-I-0009 (Visible Content Layout + Inbox) for the inbox and visible content directories.

## Goals & Non-Goals

**Goals:**
- 5 ceremony skills as Claude Code plugin slash commands
- 4 ceremony agents for autonomous processing
- Inbox scanning: detect and present unprocessed materials from `.clotho/inbox/`
- In-session extraction: speech act identification from transcripts (replaces CLO-I-0004)
- Cadence coverage: daily (2x), weekly, program-scoped, and period-scoped ceremonies

**Non-Goals:**
- External integration connectors (Otter.ai, Google Calendar, Slack, etc.)
- Automated scheduling (cron-like "run debrief at 5pm")
- Notification system

## Ceremonies

### 1. Daily Debrief (end of day)

**Trigger:** `/daily-debrief`, "end of day", "process today"
**Cadence:** Daily, evening
**Components:** Skill + Agent

**What it achieves:** Today's work is fully captured. No unprocessed materials, no stale task states, no unlinked entities. Exit state = graph is current.

**Skill (interactive):**
1. Scan `.clotho/inbox/` for unprocessed files — present what's accumulated
2. Query entities created today — show what's already captured
3. Prompt: "Anything else from today? Drop transcripts, notes, or just tell me what happened."
4. Accept materials: files, pasted text, verbal descriptions
5. Ingest all materials (inbox files + user-provided)
6. Query active/todo tasks — prompt for bulk status updates
7. Prompt: "Any decisions, risks, or blockers from outside meetings?"
8. Launch agent with gathered context

**Agent (autonomous):**
1. Read each unextracted transcript/note content
2. Identify speech acts (decisions, risks, tasks, blockers, questions, insights)
3. Create derived entities with EXTRACTED_FROM relations
4. Create/update Person entities with MENTIONS relations
5. Suggest BELONGS_TO relations to programs (user confirms)
6. Update task states from user input
7. Present summary for review
8. Sync to git

### 2. Daily Brief (start of day)

**Trigger:** `/daily-brief`, "what should I focus on today", "morning"
**Cadence:** Daily, morning
**Components:** Skill only (no agent needed)

**What it achieves:** User knows what's critical today. Prioritized view of due items, blocked work, pending reviews, and unprocessed inbox items.

**Skill:**
1. Query tasks due today (HAS_DEADLINE), tasks in 'doing' state
2. Query open blockers, unresolved questions
3. Check inbox for unprocessed materials from overnight
4. Check for pending extraction drafts not yet reviewed
5. Synthesize into prioritized "focus today" view
6. Present: critical items first, then context

### 3. Weekly Review (end of week)

**Trigger:** `/weekly-review`, "weekly reflection", "review my week"
**Cadence:** Weekly
**Components:** Skill + Agent

**What it achieves:** User has a reflection covering the week's progress, patterns, and problem areas. Blockers and risks are identified and classified.

**Skill (interactive):**
1. Query: tasks completed this week, tasks started, tasks blocked
2. Query: decisions made, risks identified, new blockers
3. Query: meetings held, reflections already created
4. Present week summary
5. Guide reflection: "What went well?", "What was harder than expected?", "What patterns do you see?"
6. Ask about problem areas: "Any programs falling behind?", "Recurring blockers?"
7. Launch agent with responses

**Agent (autonomous):**
1. Aggregate week's data across all programs
2. Identify patterns: programs with growing blockers, stalled workstreams, concentration of risks
3. Create Reflection entity with structured content
4. Link reflection to relevant programs (RELATES_TO)
5. Present for review

### 4. Report Builder (as needed)

**Trigger:** `/report`, "status report for X", "update for my boss"
**Cadence:** As needed (weekly, biweekly, monthly per program)
**Components:** Skill + Agent

**What it achieves:** Audience-appropriate status report for a program or set of programs.

**Skill (interactive):**
1. Ask: which program(s)?
2. Ask: what time period?
3. Ask: who is the audience? (boss, stakeholders, team)
4. Ask: any specific items to highlight or omit?
5. Launch agent

**Agent (autonomous):**
1. Aggregate: objectives progress, task state distribution, completed work
2. Surface: decisions made, risks active, blockers unresolved
3. Format for audience: executive summary for boss, detailed for team
4. Create Artifact entity with the report content
5. Link to program(s) with DELIVERS relation
6. Present for review and refinement

### 5. Period Compilation (quarterly+)

**Trigger:** `/period-review`, "quarterly review", "annual review", "half-year"
**Cadence:** Quarterly, half-yearly, yearly
**Components:** Skill + Agent

**What it achieves:** Comprehensive narrative of what was accomplished, decided, and learned over a longer period. Suitable for business reviews, self-assessments, performance reviews.

**Skill (interactive):**
1. Ask: which period? (Q1, H1, 2025, custom date range)
2. Ask: which programs to include? (all, or specific)
3. Ask: focus areas? (accomplishments, challenges, growth)
4. Launch agent

**Agent (autonomous):**
1. Deep aggregation: objectives met vs. missed, artifacts delivered
2. Decision analysis: what was decided and outcomes observed
3. Risk retrospective: risks that materialized vs. mitigated
4. Pattern identification across programs and time
5. Create Reflection entity (quarterly/adhoc period type)
6. Generate narrative sections: accomplishments, challenges, decisions, learnings, next period focus
7. Present for extensive user collaboration and refinement

## Architecture

### Skill + Agent Pattern

Each ceremony is a **skill + agent pair** in the Claude Code plugin:

```
skills/daily-debrief/SKILL.md     → Entry point, interactive
agents/debrief-processor.md       → Autonomous processing

skills/daily-brief/SKILL.md       → Entry point (skill-only, no agent)

skills/weekly-review/SKILL.md     → Entry point, interactive
agents/review-compiler.md         → Aggregation + pattern ID

skills/report/SKILL.md            → Entry point, interactive
agents/report-builder.md          → Aggregation + formatting

skills/period-review/SKILL.md     → Entry point, interactive
agents/period-compiler.md         → Deep analysis + narrative
```

The skill handles human-in-the-loop (gathering context, asking questions, confirming results). The agent handles machine work (ingesting, extracting, aggregating, linking).

### Inbox Integration

The daily debrief is the primary inbox processor:
1. External integrations push files to `.clotho/inbox/` continuously
2. Debrief skill scans inbox, presents accumulated files
3. Agent ingests files → entities created → content moved to visible dirs
4. Inbox empties as files are processed

Other ceremonies also check inbox (daily brief warns about unprocessed items, weekly review flags items never processed).

## Alternatives Considered

- **Ad-hoc interaction model** — user calls tools individually when they need them. Rejected: too much friction, users won't maintain the graph. Ceremonies create natural touch points.
- **Automated daemon** — processes inbox automatically without user involvement. Rejected: violates human-in-the-loop principle, user should review what's captured.
- **Single "do everything" ceremony** — one command that handles all cadences. Rejected: different cadences need different depth. Daily is quick capture, quarterly is deep analysis.

## Implementation Plan

1. Daily Debrief skill + debrief-processor agent
2. Daily Brief skill (no agent)
3. Weekly Review skill + review-compiler agent
4. Report Builder skill + report-builder agent
5. Period Compilation skill + period-compiler agent
6. Inbox scanning utility (shared across ceremonies)
7. Integration tests for each ceremony flow