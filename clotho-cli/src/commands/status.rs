use clap::Args;

use clotho_store::data::entities::EntityStore;
use clotho_store::data::processing::ProcessingLog;
use clotho_store::workspace::Workspace;

#[derive(Args)]
pub struct StatusArgs {}

pub fn run(_args: StatusArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let store = EntityStore::open(&ws.data_path().join("entities.db"))?;

    let all = store.list_all()?;

    // Count by type
    let programs: Vec<_> = all.iter().filter(|r| r.entity_type == "Program").collect();
    let responsibilities: Vec<_> = all
        .iter()
        .filter(|r| r.entity_type == "Responsibility")
        .collect();
    let objectives: Vec<_> = all
        .iter()
        .filter(|r| r.entity_type == "Objective")
        .collect();
    let workstreams: Vec<_> = all
        .iter()
        .filter(|r| r.entity_type == "Workstream")
        .collect();
    let tasks: Vec<_> = all.iter().filter(|r| r.entity_type == "Task").collect();
    let meetings: Vec<_> = all.iter().filter(|r| r.entity_type == "Meeting").collect();
    let transcripts: Vec<_> = all
        .iter()
        .filter(|r| r.entity_type == "Transcript")
        .collect();
    let notes: Vec<_> = all.iter().filter(|r| r.entity_type == "Note").collect();
    let reflections: Vec<_> = all
        .iter()
        .filter(|r| r.entity_type == "Reflection")
        .collect();
    let people: Vec<_> = all.iter().filter(|r| r.entity_type == "Person").collect();
    let decisions: Vec<_> = all.iter().filter(|r| r.entity_type == "Decision").collect();
    let risks: Vec<_> = all.iter().filter(|r| r.entity_type == "Risk").collect();
    let blockers: Vec<_> = all.iter().filter(|r| r.entity_type == "Blocker").collect();
    let questions: Vec<_> = all.iter().filter(|r| r.entity_type == "Question").collect();
    let insights: Vec<_> = all.iter().filter(|r| r.entity_type == "Insight").collect();

    // Task state counts
    let tasks_todo = tasks
        .iter()
        .filter(|t| t.task_state.as_deref() == Some("todo"))
        .count();
    let tasks_doing = tasks
        .iter()
        .filter(|t| t.task_state.as_deref() == Some("doing"))
        .count();
    let tasks_blocked = tasks
        .iter()
        .filter(|t| t.task_state.as_deref() == Some("blocked"))
        .count();
    let tasks_done = tasks
        .iter()
        .filter(|t| t.task_state.as_deref() == Some("done"))
        .count();

    // Check unprocessed transcripts
    let processing_log = ProcessingLog::open(&ws.data_path().join("entities.db")).ok();
    let unprocessed_transcripts = if let Some(ref log) = processing_log {
        let transcript_ids: Vec<&str> = transcripts.iter().map(|t| t.id.as_str()).collect();
        log.get_unprocessed("extraction", &transcript_ids)
            .unwrap_or_default()
            .len()
    } else {
        transcripts.len()
    };

    // Check inbox
    let inbox_count = std::fs::read_dir(ws.inbox_path())
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0);

    if json {
        let out = serde_json::json!({
            "workspace": ws.project_root().display().to_string(),
            "programs": programs.len(),
            "responsibilities": responsibilities.len(),
            "objectives": objectives.len(),
            "workstreams": workstreams.len(),
            "tasks": {
                "total": tasks.len(),
                "todo": tasks_todo,
                "doing": tasks_doing,
                "blocked": tasks_blocked,
                "done": tasks_done,
            },
            "meetings": meetings.len(),
            "transcripts": { "total": transcripts.len(), "unprocessed": unprocessed_transcripts },
            "notes": notes.len(),
            "reflections": reflections.len(),
            "people": people.len(),
            "decisions": decisions.len(),
            "risks": risks.len(),
            "blockers": blockers.len(),
            "questions": questions.len(),
            "insights": insights.len(),
            "inbox": inbox_count,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Clotho Workspace: {}\n", ws.project_root().display());

        // Programs with task summary
        if !programs.is_empty() {
            println!("Programs: {}", programs.len());
            for prog in &programs {
                let prog_status = prog.status.as_deref().unwrap_or("-");
                println!("  - {} ({})", prog.title, prog_status);
            }
            println!();
        }

        if !responsibilities.is_empty() {
            println!("Responsibilities: {}", responsibilities.len());
            for r in &responsibilities {
                println!("  - {}", r.title);
            }
            println!();
        }

        // Tasks
        if !tasks.is_empty() {
            println!(
                "Tasks: {} ({} todo, {} doing, {} blocked, {} done)",
                tasks.len(),
                tasks_todo,
                tasks_doing,
                tasks_blocked,
                tasks_done
            );
            if tasks_blocked > 0 {
                println!("  Blocked:");
                for t in tasks
                    .iter()
                    .filter(|t| t.task_state.as_deref() == Some("blocked"))
                {
                    println!("    - {}", t.title);
                }
            }
            println!();
        }

        // Capture stats
        let capture_total = meetings.len() + transcripts.len() + notes.len() + reflections.len();
        if capture_total > 0 {
            println!(
                "Captured: {} meetings, {} transcripts ({} unprocessed), {} notes, {} reflections",
                meetings.len(),
                transcripts.len(),
                unprocessed_transcripts,
                notes.len(),
                reflections.len()
            );
            println!();
        }

        // Derived
        let derived_total =
            decisions.len() + risks.len() + blockers.len() + questions.len() + insights.len();
        if derived_total > 0 {
            println!(
                "Derived: {} decisions, {} risks, {} blockers, {} questions, {} insights",
                decisions.len(),
                risks.len(),
                blockers.len(),
                questions.len(),
                insights.len()
            );
            println!();
        }

        // People
        if !people.is_empty() {
            println!("People: {}", people.len());
            println!();
        }

        // Inbox
        if inbox_count > 0 {
            println!("Inbox: {} files waiting", inbox_count);
            println!();
        }
    }

    Ok(())
}
