#!/usr/bin/env bash
# Clotho session-start hook
# Detects .clotho/ directory and injects workspace context

set -euo pipefail

# Find .clotho directory by walking up from cwd
find_clotho_dir() {
    local dir="$PWD"
    while [ "$dir" != "/" ]; do
        if [ -d "$dir/.clotho" ]; then
            echo "$dir/.clotho"
            return 0
        fi
        dir="$(dirname "$dir")"
    done
    return 1
}

CLOTHO_DIR=$(find_clotho_dir 2>/dev/null || echo "")

if [ -z "$CLOTHO_DIR" ]; then
    # No Clotho workspace found — minimal context
    CONTEXT="No Clotho workspace detected. Use \`clotho init\` or the \`clotho_init\` MCP tool to create one."
else
    WORKSPACE_ROOT="$(dirname "$CLOTHO_DIR")"

    # Count entities if entities.db exists
    ENTITY_COUNT=""
    if [ -f "$CLOTHO_DIR/data/entities.db" ]; then
        ENTITY_COUNT=$(sqlite3 "$CLOTHO_DIR/data/entities.db" "SELECT count(*) FROM entities;" 2>/dev/null || echo "0")
    fi

    CONTEXT="## Clotho Workspace Detected

**Path**: \`$CLOTHO_DIR\`

**Entities**: ${ENTITY_COUNT:-0}

### Available MCP Tools

**Read-only:**
- \`clotho_search\` — Full-text keyword search
- \`clotho_query\` — Cypher graph queries
- \`clotho_read_entity\` — Read entity by ID
- \`clotho_list_entities\` — List with filters
- \`clotho_get_relations\` — Show entity relations

**Write:**
- \`clotho_init\` — Initialize workspace
- \`clotho_ingest\` — Ingest a file
- \`clotho_create_entity\` — Create any entity type
- \`clotho_update_entity\` — Update entity fields
- \`clotho_delete_entity\` — Delete entity
- \`clotho_create_note\` — Create a note
- \`clotho_create_reflection\` — Create a reflection
- \`clotho_create_relation\` — Create graph edge
- \`clotho_delete_relation\` — Remove graph edge
- \`clotho_sync\` — Git sync workspace

All tools require \`workspace_path\` parameter set to \`$WORKSPACE_ROOT\`.

### Entity Types

**Structural** (what you do): Program, Responsibility, Objective
**Execution** (work in motion): Workstream, Task
**Capture** (raw material): Meeting, Transcript, Note, Reflection, Artifact
**Derived** (sense-making): Decision, Risk, Blocker, Question, Insight
**Cross-cutting**: Person

### Relation Types

belongs_to, relates_to, delivers, spawned_from, extracted_from, has_decision, has_risk, blocked_by, mentions, has_cadence, has_deadline, has_schedule"
fi

# Output as JSON for Claude Code hook system
cat <<EOF
{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": $(echo "$CONTEXT" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read()))')
  }
}
EOF
