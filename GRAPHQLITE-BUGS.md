# graphqlite — Cypher Engine Issues from Clotho Usage

These are bugs/limitations observed while using graphqlite's Cypher query engine in production across ~33 queries over 2 real-world sessions. **8 of 33 queries failed (24% error rate)**, causing the user to largely abandon the query tool in session 2.

## Bug 1: `count()` aggregate returns empty results

**Query:**
```cypher
MATCH (n) WHERE n.entity_type = 'Decision' RETURN count(n) AS total
```

**Expected:** `| total | 12 |`

**Actual:** Returns column headers but no rows — empty result set. This happens consistently with `count()` in RETURN clauses.

**Impact:** Cannot use the query tool for analytics or dashboard-style summaries. The agent had to fall back to `list_entities` and count results manually.

---

## Bug 2: Property-match syntax `{key: 'value'}` causes SQLite syntax error

**Query:**
```cypher
MATCH (n {entity_type: 'Decision'}) RETURN n.id, n.title
```

**Error:**
```
graph query failed: SQLite error: Line 1: syntax error, unexpected ':'
```

**Expected:** Standard Cypher property-match syntax should work. The `:` in `{entity_type: 'Decision'}` is valid Cypher but the SQL translator chokes on it.

**Workaround:** Use `WHERE` clause instead:
```cypher
MATCH (n) WHERE n.entity_type = 'Decision' RETURN n.id, n.title
```
This works, but the property-match form is idiomatic Cypher and users/agents reach for it first.

---

## Bug 3: `OPTIONAL MATCH` produces no results

**Query:**
```cypher
MATCH (d) WHERE d.entity_type = 'Decision'
OPTIONAL MATCH (d)-[:BELONGS_TO]->(p)
RETURN d.title, p.title AS program
```

**Expected:** Decisions without a BELONGS_TO relation should return with `program = null`.

**Actual:** Returns empty result set — no rows at all, even for decisions that DO have BELONGS_TO relations.

**Impact:** Cannot write queries that check for missing relations (orphan detection), which is a core use case for workspace health checks.

---

## Bug 4: `WHERE NOT ... EXISTS` pattern fails silently

**Query:**
```cypher
MATCH (t) WHERE t.entity_type = 'Transcript'
WHERE NOT EXISTS { MATCH (e)-[:EXTRACTED_FROM]->(t) }
RETURN t.title
```

**Expected:** Return transcripts that have NOT been extracted from (no incoming EXTRACTED_FROM edges).

**Actual:** Either syntax error or empty results (varies by exact formulation).

**Impact:** Cannot query for "unprocessed" items — a critical workflow for the extraction pipeline. The user asked "do we need to re-process any transcripts?" and the query engine couldn't answer.

---

## Bug 5: Undirected match `(a)--(b)` not supported

**Query:**
```cypher
MATCH (a)--(b) WHERE a.id = '<uuid>' RETURN b.title
```

**Error:** Syntax error or no results.

**Workaround:** Use `UNION` of both directions:
```cypher
MATCH (a)-[]->(b) WHERE a.id = '<uuid>' RETURN b.title
UNION
MATCH (a)<-[]-(b) WHERE a.id = '<uuid>' RETURN b.title
```

**Impact:** Forces verbose queries for any bidirectional traversal.

---

## Summary

The Cypher-over-SQLite translation layer handles basic `MATCH (n) WHERE ... RETURN` patterns well, but breaks on:

| Feature | Status |
|---------|--------|
| `MATCH ... WHERE ... RETURN` | Works |
| `MATCH (a)-[r:TYPE]->(b)` | Works |
| `count()` aggregates | **Broken** — returns empty |
| `{key: 'value'}` property match | **Broken** — SQLite syntax error on `:` |
| `OPTIONAL MATCH` | **Broken** — returns empty |
| `WHERE NOT EXISTS` | **Broken** — syntax error or empty |
| `(a)--(b)` undirected | **Not supported** |

The working subset is essentially: node queries with WHERE filters, directed edge traversal, and RETURN of individual properties. Anything beyond that is unreliable.

These were observed with graphqlite v0.3.7, rusqlite 0.31, from the [colliery-io/clotho](https://github.com/colliery-io/clotho) project.
