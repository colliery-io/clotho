---
name: clotho-reflection
description: "Use when the user asks to 'create a reflection', 'weekly review', 'reflect on the week', 'retrospective', 'what did I accomplish', 'monthly review', or wants to create a time-period-bound reflection entry."
---

# Reflection Workflow

Reflections are time-period-bound thinking entries that help surface patterns across programs and responsibilities.

## Creating a Reflection

```
clotho_create_reflection(period: "weekly", title: "2025-W03 Reflection")
```

**Period types:** daily, weekly, monthly, quarterly, adhoc

This creates a markdown template at `.clotho/content/reflections/` with sections for:
- Period metadata
- Reflections (freeform thinking)
- Key Takeaways
- Action Items

## Guided Reflection Process

When helping a user reflect, follow this flow:

1. **Gather context** — Query what happened in the period:
   ```
   clotho_list_entities(entity_type: "Task", state: "done")
   clotho_search(query: "completed OR shipped OR finished")
   ```

2. **Surface decisions and risks** — What was decided? What risks emerged?
   ```
   clotho_query(cypher: "MATCH (d:Decision) RETURN d.title")
   clotho_query(cypher: "MATCH (r:Risk) RETURN r.title")
   ```

3. **Check blockers** — What's still stuck?
   ```
   clotho_list_entities(entity_type: "Task", state: "blocked")
   clotho_query(cypher: "MATCH (t:Task)-[:BLOCKED_BY]->(b) RETURN t.title, b.title")
   ```

4. **Create the reflection** with synthesized content.

5. **Link to programs** — Connect the reflection to relevant programs:
   ```
   clotho_create_relation(source_id: "<reflection_id>", relation_type: "relates_to", target_id: "<program_id>")
   ```

## Prompts for Reflection

Ask the user:
- "What went well this week?"
- "What was harder than expected?"
- "What did you learn?"
- "What would you do differently?"
- "What should you focus on next?"
