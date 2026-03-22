---
name: clotho-extraction
description: "Use when the user asks to 'extract from transcript', 'find decisions', 'identify risks', 'what action items', 'extract insights', 'process meeting notes', or wants AI-assisted extraction of structured entities from unstructured content."
---

# In-Session Extraction from Transcripts

Clotho's extraction pipeline runs within Claude sessions — you ARE the extraction engine. When a user provides a transcript or meeting notes, analyze the content and create structured entities.

## Speech Act Ontology

Look for these patterns in transcripts:

| Speech Act | Signal Patterns | Create As |
|------------|-----------------|-----------|
| **Commit** | "I'll do X", "I can take that", "Let me handle" | Task (speaker owns) |
| **Decide** | "We're going with X", "Decision is Y", "We've decided" | Decision |
| **Risk** | "The concern is...", "Risk here is...", "I'm worried about" | Risk |
| **Block** | "We're stuck on...", "Blocked by...", "Can't proceed until" | Blocker |
| **Question** | "We need to figure out...", "Open question:", "How do we..." | Question |
| **Insight** | "What we learned...", "Key takeaway...", "Interesting finding" | Insight |
| **Delegate** | "Can you take this?", "Assigning to X" | Task (target owns) |
| **Request** | "I need X from you", "Can you get me..." | Task (inbound) |
| **Follow-up** | "Let's schedule a meeting about...", "Set up time for...", "Let's follow up on...", "Circle back on..." | Task |
| **Next steps** | "Next steps:", "Action items:", "By next week we need..." | Task(s) |

## Before Starting

Check the extraction queue:
```
clotho_list_unprocessed()
```

If there are multiple unprocessed transcripts, **ask the user**:
> "There are N unprocessed transcripts. Want to process them one at a time, or all at once?"

Default to **one at a time** for 5+ transcripts — batch extraction tends to overwhelm with questions.

### Skip patterns

Check for `.clotho/extraction-config.toml` in the workspace. If it exists, read it for skip patterns:

```toml
[extraction]
# Transcripts matching these title patterns are auto-skipped (glob syntax)
skip_titles = ["*1:1*", "*sport*", "*standup - DS Infra*"]
# Mark skipped transcripts as processed so they don't reappear in the queue
mark_skipped = true
```

When processing, if a transcript title matches any skip pattern, skip it and (if `mark_skipped = true`) auto-mark as processed. Report skipped items in the summary.

If the user tells you to skip certain meeting types during the session, **offer to add them to the config**:
> "Want me to add '*sport*' to your extraction skip patterns so it's automatic next time?"

## Extraction Workflow

1. **Read the transcript** — Capture it first if not already in the workspace:
   ```
   clotho_capture(file_path: "transcript.md", entity_type: "transcript", title: "Sprint Planning")
   ```

2. **Identify speech acts** — Read through the content and identify each speech act.

3. **Create entities** — For each identified item:
   ```
   clotho_create_entity(entity_type: "decision", title: "Go with microservice approach")
   clotho_create_entity(entity_type: "risk", title: "Database migration complexity")
   clotho_create_entity(entity_type: "task", title: "Write migration RFC")
   ```

4. **Create relations** — Link extracted entities back to the transcript. Use batch tool for efficiency:
   ```
   clotho_batch_create_relations(relations: [
     {"source_id": "<decision_id>", "relation_type": "extracted_from", "target_id": "<transcript_id>"},
     {"source_id": "<task_id>", "relation_type": "extracted_from", "target_id": "<transcript_id>"},
     {"source_id": "<task_id>", "relation_type": "belongs_to", "target_id": "<program_id>"}
   ])
   ```

5. **Link people** — Create Person entities and MENTIONS relations:
   ```
   clotho_create_entity(entity_type: "person", title: "Alice")
   clotho_batch_create_relations(relations: [
     {"source_id": "<transcript_id>", "relation_type": "mentions", "target_id": "<person_id>"}
   ])
   ```

6. **Mark processed** — After extraction is complete:
   ```
   clotho_mark_processed(entity_id: "<transcript_id>", process_name: "extraction")
   ```

## Important Principles

- **Human-in-the-loop**: Present your extractions to the user for review before creating entities. Say what you found and ask for confirmation.
- **Confidence**: Be explicit about certainty. "I'm confident this is a decision" vs "This might be a risk, but it could also be a concern without action."
- **Don't over-extract**: Not every statement is a speech act. "Updates" (status reports) don't need entities — they're context.
- **Do capture follow-ups**: "Let's schedule a meeting about X" is an action item, not admin noise. Capture it.
- **Watch the wrap-up**: The end of a meeting is often the richest section for action items and next steps.
- **Preserve provenance**: Always create EXTRACTED_FROM relations back to the source transcript.
