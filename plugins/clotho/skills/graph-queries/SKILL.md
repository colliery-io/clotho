---
name: clotho-graph
description: "Use when the user asks to 'relate entities', 'create a relation', 'show relations', 'query the graph', 'what belongs to', 'what's blocking', 'who is mentioned', or any graph/relation operation in Clotho."
---

# Clotho Graph Queries & Relations

## Creating Relations

Use `clotho_create_relation` to connect entities:

```
clotho_create_relation(source_id: "<uuid>", relation_type: "belongs_to", target_id: "<uuid>")
```

### Available Relation Types

| Relation | From → To | Meaning |
|----------|-----------|---------|
| `belongs_to` | Task/Objective/Note → Program/Responsibility | Ownership |
| `relates_to` | Any → Workstream | Topical connection |
| `delivers` | Artifact → Task/Objective | Evidence of completion |
| `spawned_from` | Note/Task → Meeting | Origin tracking |
| `extracted_from` | Decision/Risk/etc. → Transcript | Extraction provenance |
| `has_decision` | Meeting/Transcript → Decision | Contains decision |
| `has_risk` | Any → Risk | Flags risk |
| `blocked_by` | Task → Blocker | Impediment |
| `mentions` | Transcript/Note → Person/Program | Reference |
| `has_cadence` | Program/Workstream/Task → (self) | Recurring schedule |
| `has_deadline` | Objective/Task/Risk/Blocker → (self) | Due date |
| `has_schedule` | Task/Meeting → (self) | Scheduled time |

## Viewing Relations

```
clotho_get_relations(entity_id: "<uuid>")
```

Returns both outgoing and incoming relations.

## Removing Relations

```
clotho_delete_relation(source_id: "<uuid>", relation_type: "belongs_to", target_id: "<uuid>")
```

## Cypher Queries

For complex graph queries, use raw Cypher:

```
clotho_query(cypher: "MATCH (t:Task)-[:BLOCKED_BY]->(b:Blocker) RETURN t.title, b.title")
```

### Useful Query Patterns

**What belongs to a program?**
```cypher
MATCH (n)-[:BELONGS_TO]->(p {id: '<program_id>'}) RETURN n.title, n.entity_type
```

**What's blocking tasks?**
```cypher
MATCH (t:Task)-[:BLOCKED_BY]->(b:Blocker) RETURN t.title AS task, b.title AS blocker
```

**What decisions came from a meeting?**
```cypher
MATCH (m:Meeting)-[:HAS_DECISION]->(d:Decision) RETURN m.title, d.title
```

**Who is mentioned in transcripts?**
```cypher
MATCH (t:Transcript)-[:MENTIONS]->(p:Person) RETURN t.title, p.title
```

**Trace extraction provenance:**
```cypher
MATCH (d:Decision)-[:EXTRACTED_FROM]->(t:Transcript) RETURN d.title, t.title
```
