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

4. **Create relations** — Link extracted entities back to the transcript:
   ```
   clotho_create_relation(source_id: "<decision_id>", relation_type: "extracted_from", target_id: "<transcript_id>")
   ```

5. **Link people** — Create Person entities and MENTIONS relations:
   ```
   clotho_create_entity(entity_type: "person", title: "Alice")
   clotho_create_relation(source_id: "<transcript_id>", relation_type: "mentions", target_id: "<person_id>")
   ```

6. **Connect to structure** — Link tasks/decisions to programs if context is clear:
   ```
   clotho_create_relation(source_id: "<task_id>", relation_type: "belongs_to", target_id: "<program_id>")
   ```

## Important Principles

- **Human-in-the-loop**: Present your extractions to the user for review before creating entities. Say what you found and ask for confirmation.
- **Confidence**: Be explicit about certainty. "I'm confident this is a decision" vs "This might be a risk, but it could also be a concern without action."
- **Don't over-extract**: Not every statement is a speech act. "Updates" (status reports) don't need entities — they're context.
- **Preserve provenance**: Always create EXTRACTED_FROM relations back to the source transcript.
