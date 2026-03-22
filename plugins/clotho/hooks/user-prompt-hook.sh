#!/usr/bin/env bash
# Clotho UserPromptSubmit hook
# Detects work-related signals in user messages and injects Clotho tool instructions
# This is the main mechanism for making Clotho "come forward" — every user message
# gets checked for patterns that should trigger Clotho tool usage.

set -euo pipefail

# Read the prompt from stdin JSON
INPUT=$(cat)
PROMPT=$(echo "$INPUT" | jq -r '.prompt // ""')

# Lowercase for matching
PROMPT_LOWER=$(echo "$PROMPT" | tr '[:upper:]' '[:lower:]')

CONTEXT=""

# --- Meeting / Transcript signals ---
if echo "$PROMPT_LOWER" | grep -qE '(meeting|transcript|call with|sync with|standup|retro|1:1|one.on.one|had a chat|spoke with|met with|talked to|conversation with)'; then
    CONTEXT="${CONTEXT}The user is talking about a meeting or conversation. Use Clotho to capture it:
- If they have a file/transcript → \`clotho_capture(file_path, entity_type: \"transcript\")\`
- If they're describing what happened → \`clotho_create_note(title, content)\` or \`clotho_create_entity(entity_type: \"meeting\")\`
- After capturing, offer to extract signals (decisions, tasks, risks) using the extraction skill
- Link to the relevant Program via \`belongs_to\`
"
fi

# --- Decision signals ---
if echo "$PROMPT_LOWER" | grep -qE '(we decided|decision|went with|chose|agreed to|commitment|committed to|we.re going with)'; then
    CONTEXT="${CONTEXT}The user mentioned a decision. Capture it in Clotho:
- \`clotho_create_entity(entity_type: \"decision\", title: \"<specific decision>\")\`
- Link it to the relevant Program and the source meeting/transcript
"
fi

# --- Task / Action item signals ---
if echo "$PROMPT_LOWER" | grep -qE '(need to|action item|follow.up|todo|to.do|task|should do|gotta|have to|assigned|delegate|next step)'; then
    CONTEXT="${CONTEXT}The user mentioned a task or action item. Capture it in Clotho:
- \`clotho_create_entity(entity_type: \"task\", title: \"<specific task>\")\`
- Link to the relevant Program and assign to a Person if mentioned
"
fi

# --- Risk / Blocker signals ---
if echo "$PROMPT_LOWER" | grep -qE '(risk|concern|worried|blocker|blocked|stuck|impediment|problem|issue|can.t proceed)'; then
    CONTEXT="${CONTEXT}The user mentioned a risk or blocker. Capture it in Clotho:
- Risk: \`clotho_create_entity(entity_type: \"risk\", title: \"<specific risk>\")\`
- Blocker: \`clotho_create_entity(entity_type: \"blocker\", title: \"<specific blocker>\")\`
- Link to the relevant Program
"
fi

# --- Status / Review signals ---
if echo "$PROMPT_LOWER" | grep -qE '(status|how.s|what.s going on|update me|where are we|what.s happening|overview|summary|review|check in|debrief|brief me|catch me up)'; then
    CONTEXT="${CONTEXT}The user wants a status update. Use Clotho as the source of truth:
- Start with \`clotho_workspace_summary\` for the big picture
- Use \`clotho_list_entities\` to drill into specific types
- Check \`clotho_list_unprocessed\` for pending extraction work
- Surface blocked tasks and open risks
- Do NOT summarize from memory — query Clotho
"
fi

# --- People signals ---
if echo "$PROMPT_LOWER" | grep -qE '(hired|new team|direct report|manages|reports to|team member|colleague|stakeholder)'; then
    CONTEXT="${CONTEXT}The user mentioned a person in a work context. Ensure they exist in Clotho:
- \`clotho_search(query: \"<person name>\")\` to check
- \`clotho_create_entity(entity_type: \"person\", title: \"<name>\", email: \"<email>\")\` if new
- Link to relevant Programs or Responsibilities via \`mentions\` or \`belongs_to\`
"
fi

# --- Reflection / Thinking signals ---
if echo "$PROMPT_LOWER" | grep -qE '(thinking about|reflecting|looking back|retrospective|what i learned|takeaway|insight|observation|pattern i.m seeing)'; then
    CONTEXT="${CONTEXT}The user is reflecting. Capture insights in Clotho:
- For insights: \`clotho_create_entity(entity_type: \"insight\", title: \"<insight>\")\`
- For broader reflection: \`clotho_create_reflection(title: \"<reflection title>\", content: \"<content>\")\`
- Link to relevant Programs
"
fi

# --- File / Document signals ---
if echo "$PROMPT_LOWER" | grep -qE '(this file|this document|this transcript|process this|capture this|here.s a|attached|pasted)'; then
    CONTEXT="${CONTEXT}The user is providing content to capture. Use Clotho:
- For files: \`clotho_capture(file_path, entity_type)\`
- For directories: \`clotho_capture_directory(path, pattern, entity_type)\`
- After capture, offer to extract signals
"
fi

# --- Schedule / Calendar signals ---
if echo "$PROMPT_LOWER" | grep -qE '(schedule|calendar|book a meeting|set up time|find a slot|when.s.*free|availability)'; then
    CONTEXT="${CONTEXT}The user is discussing scheduling. Check Clotho for relevant People entities with email addresses:
- \`clotho_list_entities(entity_type: \"Person\")\` to find participants
- Person entities may have email addresses in metadata for calendar lookups
"
fi

# Only output if we have context to inject
if [ -n "$CONTEXT" ]; then
    FULL_CONTEXT="## Clotho: Work Signal Detected

${CONTEXT}
**Remember:** Clotho is the user's work management system. Capture work artifacts there, not in conversation memory."

    cat <<EOF
{
  "hookSpecificOutput": {
    "hookEventName": "UserPromptSubmit",
    "additionalContext": $(echo "$FULL_CONTEXT" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read()))')
  }
}
EOF
else
    # No work signals detected — no context to inject
    echo "{}"
fi
