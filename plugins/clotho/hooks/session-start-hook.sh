#!/usr/bin/env bash
# Clotho session-start hook
# Queries workspace state and tells Claude what to DO, not just what exists

set -euo pipefail

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
    CONTEXT="No Clotho workspace detected. If the user mentions work, meetings, tasks, or wants to organize their professional life, suggest initializing a Clotho workspace with \`clotho_init\`."
else
    DB="$CLOTHO_DIR/data/entities.db"

    # Query workspace state
    ENTITY_COUNT=0
    BLOCKED_TASKS=""
    ACTIVE_TASKS=""
    UNPROCESSED_COUNT=0
    RECENT=""

    if [ -f "$DB" ]; then
        ENTITY_COUNT=$(sqlite3 "$DB" "SELECT count(*) FROM entities;" 2>/dev/null || echo "0")
        BLOCKED_TASKS=$(sqlite3 "$DB" "SELECT id, title FROM entities WHERE task_state='blocked' LIMIT 5;" 2>/dev/null || echo "")
        ACTIVE_TASKS=$(sqlite3 "$DB" "SELECT count(*) FROM entities WHERE task_state='doing';" 2>/dev/null || echo "0")
        TODO_TASKS=$(sqlite3 "$DB" "SELECT count(*) FROM entities WHERE task_state='todo';" 2>/dev/null || echo "0")

        # Count unprocessed transcripts (no processing_log entry for 'extraction')
        if sqlite3 "$DB" "SELECT 1 FROM sqlite_master WHERE name='processing_log' LIMIT 1;" 2>/dev/null | grep -q 1; then
            UNPROCESSED_COUNT=$(sqlite3 "$DB" "SELECT count(*) FROM entities WHERE entity_type='Transcript' AND id NOT IN (SELECT DISTINCT entity_id FROM processing_log WHERE process_name='extraction');" 2>/dev/null || echo "0")
        else
            UNPROCESSED_COUNT=$(sqlite3 "$DB" "SELECT count(*) FROM entities WHERE entity_type='Transcript';" 2>/dev/null || echo "0")
        fi

        # Recent activity (last 5 updated entities)
        RECENT=$(sqlite3 -separator '|' "$DB" "SELECT entity_type, title, substr(updated_at, 1, 10) FROM entities ORDER BY updated_at DESC LIMIT 5;" 2>/dev/null || echo "")
    fi

    # Build actionable context
    CONTEXT="## Clotho Workspace Active

This is a **Clotho-managed workspace**. Clotho is the user's primary work management system. **Always use Clotho tools** when the user discusses work, meetings, tasks, decisions, people, risks, or anything related to their professional activities.

### Workspace State
- **${ENTITY_COUNT} entities** across programs, tasks, transcripts, decisions, etc.
- **${ACTIVE_TASKS} active tasks**, **${TODO_TASKS} queued**, **${BLOCKED_TASKS:+BLOCKED items exist — surface these first}${BLOCKED_TASKS:-0 blocked}**
- **${UNPROCESSED_COUNT} unprocessed transcripts** in the extraction queue"

    if [ -n "$BLOCKED_TASKS" ]; then
        CONTEXT="${CONTEXT}

### ⚠ Blocked Tasks (surface immediately)
$(echo "$BLOCKED_TASKS" | while IFS='|' read -r id title; do echo "- ${title} (\`${id:0:8}\`)"; done)"
    fi

    if [ "$UNPROCESSED_COUNT" -gt 0 ] 2>/dev/null; then
        CONTEXT="${CONTEXT}

### Extraction Queue
${UNPROCESSED_COUNT} transcripts awaiting extraction. Mention this to the user — they may want to run \`/daily-debrief\` or process them."
    fi

    CONTEXT="${CONTEXT}

### What to Do
1. **Start by calling \`clotho_workspace_summary\`** to get the full picture
2. If the user mentions a meeting or transcript → offer to capture/extract it
3. If the user discusses work outcomes → create entities (decisions, tasks, risks)
4. If the user asks about status → query Clotho, don't guess
5. Always link new entities to Programs via \`belongs_to\` relations
6. Use \`clotho_batch_create_relations\` for multiple links at once

### Key Behavioral Rules
- **Clotho is the source of truth** for the user's work. Don't track tasks, decisions, or risks in your own memory — put them in Clotho.
- **Capture aggressively.** When the user mentions something actionable (a decision, a risk, a follow-up), create the entity. It's easier to delete than to miss.
- **Link everything.** Orphan entities are useless. Every entity should connect to a program or person.
- **Surface the extraction queue.** If there are unprocessed transcripts, mention it naturally."
fi

# Output as JSON
cat <<EOF
{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": $(echo "$CONTEXT" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read()))')
  }
}
EOF
