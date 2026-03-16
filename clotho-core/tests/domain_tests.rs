use chrono::Utc;
use clotho_core::domain::entities::capture::*;
use clotho_core::domain::entities::derived::*;
use clotho_core::domain::entities::execution::*;
use clotho_core::domain::entities::person::*;
use clotho_core::domain::entities::structural::*;
use clotho_core::domain::traits::*;
use clotho_core::domain::types::*;

// ===========================================================================
// Taskable state machine
// ===========================================================================

#[test]
fn task_starts_in_todo() {
    let task = Task::new("test task");
    assert_eq!(task.state(), TaskState::Todo);
}

#[test]
fn task_todo_to_doing() {
    let mut task = Task::new("test");
    assert!(task.transition(TaskState::Doing).is_ok());
    assert_eq!(task.state(), TaskState::Doing);
}

#[test]
fn task_doing_to_blocked() {
    let mut task = Task::new("test");
    task.transition(TaskState::Doing).unwrap();
    assert!(task.transition(TaskState::Blocked).is_ok());
    assert_eq!(task.state(), TaskState::Blocked);
}

#[test]
fn task_doing_to_done() {
    let mut task = Task::new("test");
    task.transition(TaskState::Doing).unwrap();
    assert!(task.transition(TaskState::Done).is_ok());
    assert_eq!(task.state(), TaskState::Done);
}

#[test]
fn task_blocked_to_doing() {
    let mut task = Task::new("test");
    task.transition(TaskState::Doing).unwrap();
    task.transition(TaskState::Blocked).unwrap();
    assert!(task.transition(TaskState::Doing).is_ok());
    assert_eq!(task.state(), TaskState::Doing);
}

#[test]
fn task_todo_to_done_invalid() {
    let mut task = Task::new("test");
    let result = task.transition(TaskState::Done);
    assert!(result.is_err());
    assert_eq!(task.state(), TaskState::Todo);
}

#[test]
fn task_todo_to_blocked_invalid() {
    let mut task = Task::new("test");
    let result = task.transition(TaskState::Blocked);
    assert!(result.is_err());
    assert_eq!(task.state(), TaskState::Todo);
}

#[test]
fn task_done_is_terminal() {
    let mut task = Task::new("test");
    task.transition(TaskState::Doing).unwrap();
    task.transition(TaskState::Done).unwrap();
    assert!(task.valid_transitions().is_empty());
    assert!(task.transition(TaskState::Todo).is_err());
    assert!(task.transition(TaskState::Doing).is_err());
    assert!(task.transition(TaskState::Blocked).is_err());
}

#[test]
fn task_valid_transitions() {
    let mut task = Task::new("test");
    assert_eq!(task.valid_transitions(), vec![TaskState::Doing]);

    task.transition(TaskState::Doing).unwrap();
    assert_eq!(task.valid_transitions(), vec![TaskState::Blocked, TaskState::Done]);

    task.transition(TaskState::Blocked).unwrap();
    assert_eq!(task.valid_transitions(), vec![TaskState::Doing]);
}

// ===========================================================================
// Extractable lifecycle
// ===========================================================================

#[test]
fn decision_starts_as_draft() {
    let d = Decision::draft("test decision", 0.9, None);
    assert_eq!(d.extraction_status(), ExtractionStatus::Draft);
}

#[test]
fn decision_draft_to_promoted() {
    let mut d = Decision::draft("test", 0.85, None);
    assert!(d.promote().is_ok());
    assert_eq!(d.extraction_status(), ExtractionStatus::Promoted);
}

#[test]
fn decision_draft_to_discarded() {
    let mut d = Decision::draft("test", 0.5, None);
    d.discard();
    assert_eq!(d.extraction_status(), ExtractionStatus::Discarded);
}

#[test]
fn promote_on_promoted_fails() {
    let mut d = Decision::draft("test", 0.9, None);
    d.promote().unwrap();
    let result = d.promote();
    assert!(result.is_err());
}

#[test]
fn promote_on_discarded_fails() {
    let mut d = Decision::draft("test", 0.3, None);
    d.discard();
    let result = d.promote();
    assert!(result.is_err());
}

#[test]
fn discard_on_discarded_is_noop() {
    let mut d = Decision::draft("test", 0.3, None);
    d.discard();
    let _updated = d.updated_at();
    // discard again — should be a no-op (status stays Discarded)
    d.discard();
    assert_eq!(d.extraction_status(), ExtractionStatus::Discarded);
}

#[test]
fn extractable_confidence_and_source_span() {
    let span = SourceSpan {
        transcript_id: EntityId::new(),
        start: 100,
        end: 200,
    };
    let d = Decision::draft("test", 0.87, Some(span.clone()));
    assert_eq!(d.confidence(), 0.87);
    assert_eq!(d.source_span(), Some(&span));
}

// ===========================================================================
// Activatable
// ===========================================================================

#[test]
fn program_starts_active() {
    let p = Program::new("test program");
    assert_eq!(p.status(), Status::Active);
}

#[test]
fn program_active_to_inactive() {
    let mut p = Program::new("test");
    p.set_status(Status::Inactive);
    assert_eq!(p.status(), Status::Inactive);
}

#[test]
fn program_inactive_to_active() {
    let mut p = Program::new("test");
    p.set_status(Status::Inactive);
    p.set_status(Status::Active);
    assert_eq!(p.status(), Status::Active);
}

// ===========================================================================
// Taggable
// ===========================================================================

#[test]
fn taggable_add_and_get() {
    let mut p = Program::new("test");
    p.add_tag(Tag::from("urgent"));
    p.add_tag(Tag::from("review"));
    assert_eq!(p.tags().len(), 2);
    assert_eq!(p.tags()[0].as_str(), "urgent");
}

#[test]
fn taggable_no_duplicates() {
    let mut p = Program::new("test");
    p.add_tag(Tag::from("urgent"));
    p.add_tag(Tag::from("urgent"));
    assert_eq!(p.tags().len(), 1);
}

#[test]
fn taggable_remove() {
    let mut p = Program::new("test");
    p.add_tag(Tag::from("urgent"));
    p.add_tag(Tag::from("review"));
    p.remove_tag("urgent");
    assert_eq!(p.tags().len(), 1);
    assert_eq!(p.tags()[0].as_str(), "review");
}

#[test]
fn taggable_remove_nonexistent_is_noop() {
    let mut p = Program::new("test");
    p.add_tag(Tag::from("urgent"));
    p.remove_tag("nonexistent");
    assert_eq!(p.tags().len(), 1);
}

// ===========================================================================
// Temporal traits
// ===========================================================================

#[test]
fn has_cadence_set_get() {
    let mut ws = Workstream::new("weekly sync");
    assert!(ws.cadence().is_none());

    let cadence = Cadence {
        frequency: Frequency::Weekly,
        cron: None,
        label: Some("weekly sync".into()),
        next_occurrence: None,
    };
    ws.set_cadence(Some(cadence));
    assert!(ws.cadence().is_some());
    assert_eq!(ws.cadence().unwrap().frequency, Frequency::Weekly);
}

#[test]
fn has_cadence_clear() {
    let mut ws = Workstream::new("test");
    ws.set_cadence(Some(Cadence {
        frequency: Frequency::Daily,
        cron: None,
        label: None,
        next_occurrence: None,
    }));
    ws.set_cadence(None);
    assert!(ws.cadence().is_none());
}

#[test]
fn has_deadline_set_get() {
    let mut obj = Objective::new("reduce latency", EntityId::new());
    assert!(obj.deadline().is_none());

    let dl = Utc::now();
    obj.set_deadline(Some(dl));
    assert_eq!(obj.deadline(), Some(dl));
}

#[test]
fn has_schedule_set_get() {
    let now = Utc::now();
    let mut task = Task::new("standup");
    assert!(task.scheduled_at().is_none());

    task.set_scheduled_at(Some(now));
    assert_eq!(task.scheduled_at(), Some(now));
}

// ===========================================================================
// Entity identity
// ===========================================================================

#[test]
fn entity_id_unique() {
    let a = EntityId::new();
    let b = EntityId::new();
    assert_ne!(a, b);
}

#[test]
fn entity_id_display() {
    let id = EntityId::new();
    let s = format!("{}", id);
    assert!(!s.is_empty());
    // UUID format: 8-4-4-4-12
    assert_eq!(s.len(), 36);
}

#[test]
fn entity_id_equality() {
    let id = EntityId::new();
    let cloned = id.clone();
    assert_eq!(id, cloned);
}

// ===========================================================================
// Serialization roundtrip
// ===========================================================================

#[test]
fn program_serde_roundtrip() {
    let mut p = Program::new("test program");
    p.add_tag(Tag::from("important"));
    let json = serde_json::to_string(&p).unwrap();
    let deserialized: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(p.title(), deserialized.title());
    assert_eq!(p.tags().len(), deserialized.tags().len());
}

#[test]
fn task_serde_roundtrip() {
    let mut t = Task::new("test task");
    t.transition(TaskState::Doing).unwrap();
    let json = serde_json::to_string(&t).unwrap();
    let deserialized: Task = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.state(), TaskState::Doing);
}

#[test]
fn meeting_serde_roundtrip() {
    let m = Meeting::new("standup", Utc::now());
    let json = serde_json::to_string(&m).unwrap();
    let deserialized: Meeting = serde_json::from_str(&json).unwrap();
    assert_eq!(m.title(), deserialized.title());
}

#[test]
fn decision_serde_roundtrip() {
    let d = Decision::draft("go with option A", 0.92, None);
    let json = serde_json::to_string(&d).unwrap();
    let deserialized: Decision = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.extraction_status(), ExtractionStatus::Draft);
    assert_eq!(deserialized.confidence(), 0.92);
}

#[test]
fn person_serde_roundtrip() {
    let p = Person::new("Alice").with_email("alice@example.com");
    let json = serde_json::to_string(&p).unwrap();
    let deserialized: Person = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.title(), "Alice");
    assert_eq!(deserialized.email, Some("alice@example.com".to_string()));
}

// ===========================================================================
// Trait composition (compile-time verification)
// ===========================================================================

fn assert_structural<T: Entity + Activatable + Relatable + Taggable + ContentBearing>(_: &T) {}
fn assert_taskable_entity<T: Entity + Taskable + Relatable + Taggable + ContentBearing + HasCadence + HasDeadline + HasSchedule>(_: &T) {}
fn assert_extractable_entity<T: Entity + Extractable + Relatable + Taggable>(_: &T) {}
fn assert_person_entity<T: Entity + Relatable + Taggable + ContentBearing>(_: &T) {}
fn assert_has_cadence<T: HasCadence>(_: &T) {}
fn assert_has_deadline<T: HasDeadline>(_: &T) {}
fn assert_has_schedule<T: HasSchedule>(_: &T) {}

#[test]
fn trait_composition_structural() {
    let p = Program::new("test");
    assert_structural(&p);
    assert_has_cadence(&p);

    let r = Responsibility::new("test");
    assert_structural(&r);
    assert_has_cadence(&r);

    let o = Objective::new("test", EntityId::new());
    assert_structural(&o);
    assert_has_deadline(&o);
}

#[test]
fn trait_composition_execution() {
    let ws = Workstream::new("test");
    assert_structural(&ws); // Workstream is also Activatable + all structural traits
    assert_has_cadence(&ws);

    let task = Task::new("test");
    assert_taskable_entity(&task);
}

#[test]
fn trait_composition_capture() {
    let m = Meeting::new("test", Utc::now());
    assert_has_schedule(&m);

    let a = Artifact::new("test", "https://example.com");
    assert_has_deadline(&a);
}

#[test]
fn trait_composition_derived() {
    let d = Decision::draft("test", 0.9, None);
    assert_extractable_entity(&d);

    let r = Risk::draft("test", 0.8, None);
    assert_extractable_entity(&r);
    assert_has_deadline(&r);

    let b = Blocker::draft("test", 0.7, None);
    assert_extractable_entity(&b);
    assert_has_deadline(&b);

    let q = Question::draft("test", 0.6, None);
    assert_extractable_entity(&q);
    assert_has_deadline(&q);

    let i = Insight::draft("test", 0.5, None);
    assert_extractable_entity(&i);
}

#[test]
fn trait_composition_person() {
    let p = Person::new("Alice");
    assert_person_entity(&p);
}
