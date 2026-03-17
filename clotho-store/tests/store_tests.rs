use chrono::Utc;
use tempfile::tempdir;

use clotho_core::domain::traits::*;
use clotho_core::domain::types::*;
use clotho_core::graph::GraphStore;

use clotho_store::content::ContentStore;
use clotho_store::data::entities::{EntityRow, EntityStore};
use clotho_store::data::extractions::{ExtractionRow, ExtractionStore};
use clotho_store::data::jsonl::{Event, EventStore, EventType, TagStore};
use clotho_store::index::SearchIndex;
use clotho_store::sync::StoreSync;
use clotho_store::workspace::Workspace;

// ===========================================================================
// Workspace init
// ===========================================================================

#[test]
fn workspace_init_creates_structure() {
    let tmp = tempdir().unwrap();
    let ws = Workspace::init(tmp.path()).unwrap();

    assert!(ws.path.exists());
    assert!(ws.project_root().join("meetings").is_dir());
    assert!(ws.project_root().join("reflections").is_dir());
    assert!(ws.project_root().join("artifacts").is_dir());
    assert!(ws.project_root().join("notes").is_dir());
    assert!(ws.project_root().join("people").is_dir());
    assert!(ws.data_path().is_dir());
    assert!(ws.graph_path().is_dir());
    assert!(ws.index_path().is_dir());
    assert!(ws.config_path().join("config.toml").is_file());
    assert!(ws.config_path().join("ontology.toml").is_file());
}

#[test]
fn workspace_init_fails_if_exists() {
    let tmp = tempdir().unwrap();
    Workspace::init(tmp.path()).unwrap();
    let result = Workspace::init(tmp.path());
    assert!(result.is_err());
}

#[test]
fn workspace_open_succeeds() {
    let tmp = tempdir().unwrap();
    Workspace::init(tmp.path()).unwrap();
    let ws = Workspace::open(tmp.path()).unwrap();
    assert!(ws.path.exists());
}

#[test]
fn workspace_open_fails_if_missing() {
    let tmp = tempdir().unwrap();
    let result = Workspace::open(tmp.path());
    assert!(result.is_err());
}

#[test]
fn workspace_read_config() {
    let tmp = tempdir().unwrap();
    let ws = Workspace::init(tmp.path()).unwrap();
    let config = ws.read_config().unwrap();
    assert_eq!(config.sync.debounce_seconds, 30);
}

// ===========================================================================
// Content round-trip
// ===========================================================================

#[test]
fn content_write_read_delete() {
    let tmp = tempdir().unwrap();
    let ws = Workspace::init(tmp.path()).unwrap();
    let store = ContentStore::new(&ws.project_root());

    let id = EntityId::new();
    let path = store
        .write_content(EntityType::Note, &id, "# Hello\n\nThis is a note.")
        .unwrap();
    assert!(path.exists());

    let content = store.read_content(EntityType::Note, &id).unwrap();
    assert_eq!(content.unwrap(), "# Hello\n\nThis is a note.");

    store.delete_content(EntityType::Note, &id).unwrap();
    assert!(store.read_content(EntityType::Note, &id).unwrap().is_none());
}

#[test]
fn content_list() {
    let tmp = tempdir().unwrap();
    let ws = Workspace::init(tmp.path()).unwrap();
    let store = ContentStore::new(&ws.project_root());

    store.write_content(EntityType::Note, &EntityId::new(), "Note 1").unwrap();
    store.write_content(EntityType::Note, &EntityId::new(), "Note 2").unwrap();
    store.write_content(EntityType::Meeting, &EntityId::new(), "Meeting").unwrap();

    let notes = store.list_content(EntityType::Note).unwrap();
    assert_eq!(notes.len(), 2);

    let meetings = store.list_content(EntityType::Meeting).unwrap();
    assert_eq!(meetings.len(), 1);
}

#[test]
fn content_read_nonexistent() {
    let tmp = tempdir().unwrap();
    let ws = Workspace::init(tmp.path()).unwrap();
    let store = ContentStore::new(&ws.project_root());

    let result = store.read_content(EntityType::Note, &EntityId::new()).unwrap();
    assert!(result.is_none());
}

// ===========================================================================
// Entity CRUD
// ===========================================================================

fn make_entity_row(entity_type: &str, title: &str) -> EntityRow {
    EntityRow {
        id: EntityId::new().to_string(),
        entity_type: entity_type.to_string(),
        title: title.to_string(),
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
        status: Some("active".to_string()),
        task_state: None,
        extraction_status: None,
        source_transcript_id: None,
        source_span_start: None,
        source_span_end: None,
        confidence: None,
        content_path: None,
        metadata: None,
    }
}

#[test]
fn entity_insert_and_get() {
    let store = EntityStore::in_memory().unwrap();
    let row = make_entity_row("Program", "Test Program");
    store.insert(&row).unwrap();

    let got = store.get(&row.id).unwrap().unwrap();
    assert_eq!(got.title, "Test Program");
    assert_eq!(got.entity_type, "Program");
}

#[test]
fn entity_update() {
    let store = EntityStore::in_memory().unwrap();
    let mut row = make_entity_row("Program", "Original");
    store.insert(&row).unwrap();

    row.title = "Updated".to_string();
    store.update(&row).unwrap();

    let got = store.get(&row.id).unwrap().unwrap();
    assert_eq!(got.title, "Updated");
}

#[test]
fn entity_delete() {
    let store = EntityStore::in_memory().unwrap();
    let row = make_entity_row("Task", "Do thing");
    store.insert(&row).unwrap();
    store.delete(&row.id).unwrap();
    assert!(store.get(&row.id).unwrap().is_none());
}

#[test]
fn entity_list_by_type() {
    let store = EntityStore::in_memory().unwrap();
    store.insert(&make_entity_row("Task", "Task 1")).unwrap();
    store.insert(&make_entity_row("Task", "Task 2")).unwrap();
    store.insert(&make_entity_row("Program", "Prog")).unwrap();

    let tasks = store.list_by_type("Task").unwrap();
    assert_eq!(tasks.len(), 2);

    let progs = store.list_by_type("Program").unwrap();
    assert_eq!(progs.len(), 1);
}

#[test]
fn entity_list_by_status() {
    let store = EntityStore::in_memory().unwrap();
    let mut row1 = make_entity_row("Program", "Active");
    row1.status = Some("active".to_string());
    store.insert(&row1).unwrap();

    let mut row2 = make_entity_row("Program", "Inactive");
    row2.status = Some("inactive".to_string());
    store.insert(&row2).unwrap();

    let active = store.list_by_status("active").unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].title, "Active");
}

#[test]
fn entity_list_by_state() {
    let store = EntityStore::in_memory().unwrap();
    let mut row = make_entity_row("Task", "Blocked task");
    row.task_state = Some("blocked".to_string());
    store.insert(&row).unwrap();

    let blocked = store.list_by_state("blocked").unwrap();
    assert_eq!(blocked.len(), 1);
}

// ===========================================================================
// Extraction lifecycle
// ===========================================================================

fn make_extraction_row(title: &str, confidence: f64) -> ExtractionRow {
    ExtractionRow {
        id: EntityId::new().to_string(),
        entity_type: "Decision".to_string(),
        title: title.to_string(),
        speech_act: Some("decide".to_string()),
        extraction_status: "draft".to_string(),
        source_transcript_id: Some(EntityId::new().to_string()),
        source_span_start: Some(10),
        source_span_end: Some(50),
        confidence: Some(confidence),
        created_at: Utc::now().to_rfc3339(),
        metadata: None,
    }
}

#[test]
fn extraction_insert_and_list_pending() {
    let store = ExtractionStore::in_memory().unwrap();
    store.insert_draft(&make_extraction_row("Decision A", 0.9)).unwrap();
    store.insert_draft(&make_extraction_row("Decision B", 0.7)).unwrap();

    let pending = store.list_pending().unwrap();
    assert_eq!(pending.len(), 2);
    // Should be ordered by confidence DESC
    assert!(pending[0].confidence.unwrap() >= pending[1].confidence.unwrap());
}

#[test]
fn extraction_promote() {
    let store = ExtractionStore::in_memory().unwrap();
    let row = make_extraction_row("Go with A", 0.95);
    let id = row.id.clone();
    store.insert_draft(&row).unwrap();

    let promoted = store.promote(&id).unwrap();
    assert_eq!(promoted.extraction_status, "promoted");

    // No longer in pending
    let pending = store.list_pending().unwrap();
    assert!(pending.is_empty());
}

#[test]
fn extraction_discard() {
    let store = ExtractionStore::in_memory().unwrap();
    let row = make_extraction_row("Bad decision", 0.3);
    let id = row.id.clone();
    store.insert_draft(&row).unwrap();

    store.discard(&id).unwrap();
    assert!(store.get(&id).unwrap().is_none());
}

#[test]
fn extraction_list_by_confidence() {
    let store = ExtractionStore::in_memory().unwrap();
    store.insert_draft(&make_extraction_row("High", 0.9)).unwrap();
    store.insert_draft(&make_extraction_row("Low", 0.3)).unwrap();

    let high = store.list_by_confidence(0.8).unwrap();
    assert_eq!(high.len(), 1);
    assert_eq!(high[0].title, "High");
}

// ===========================================================================
// JSONL tags
// ===========================================================================

#[test]
fn tags_add_get_remove() {
    let tmp = tempdir().unwrap();
    let store = TagStore::new(tmp.path());

    let id = EntityId::new().to_string();
    store.add_tag(&id, "urgent").unwrap();
    store.add_tag(&id, "review").unwrap();

    let tags = store.get_tags(&id).unwrap();
    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&"urgent".to_string()));

    store.remove_tag(&id, "urgent").unwrap();
    let tags = store.get_tags(&id).unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0], "review");
}

#[test]
fn tags_no_duplicates() {
    let tmp = tempdir().unwrap();
    let store = TagStore::new(tmp.path());

    let id = EntityId::new().to_string();
    store.add_tag(&id, "urgent").unwrap();
    store.add_tag(&id, "urgent").unwrap();

    let tags = store.get_tags(&id).unwrap();
    assert_eq!(tags.len(), 1);
}

#[test]
fn tags_get_entities_by_tag() {
    let tmp = tempdir().unwrap();
    let store = TagStore::new(tmp.path());

    let id1 = EntityId::new().to_string();
    let id2 = EntityId::new().to_string();
    store.add_tag(&id1, "important").unwrap();
    store.add_tag(&id2, "important").unwrap();

    let ids = store.get_entities_by_tag("important").unwrap();
    assert_eq!(ids.len(), 2);
}

// ===========================================================================
// JSONL events
// ===========================================================================

#[test]
fn events_log_and_read() {
    let tmp = tempdir().unwrap();
    let store = EventStore::new(tmp.path());

    let id = EntityId::new().to_string();
    store
        .log(&Event {
            timestamp: Utc::now(),
            event_type: EventType::Created,
            entity_id: id.clone(),
            details: None,
        })
        .unwrap();
    store
        .log(&Event {
            timestamp: Utc::now(),
            event_type: EventType::Updated,
            entity_id: id.clone(),
            details: Some(serde_json::json!({"field": "title"})),
        })
        .unwrap();

    let events = store.read_all().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_type, EventType::Created);
    assert_eq!(events[1].event_type, EventType::Updated);
}

// ===========================================================================
// FTS5 search
// ===========================================================================

#[test]
fn search_index_and_query() {
    let idx = SearchIndex::in_memory().unwrap();

    idx.index_entity("id1", "Program", "Technical Education", "Building a learning culture through workshops and mentorship").unwrap();
    idx.index_entity("id2", "Program", "PMO Establishment", "Setting up project management office and governance").unwrap();
    idx.index_entity("id3", "Note", "Architecture Thoughts", "Exploring microservice patterns and event-driven design").unwrap();

    let results = idx.search("learning culture").unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].entity_id, "id1");
}

#[test]
fn search_remove_entity() {
    let idx = SearchIndex::in_memory().unwrap();
    idx.index_entity("id1", "Note", "Test", "searchable content here").unwrap();

    let results = idx.search("searchable").unwrap();
    assert_eq!(results.len(), 1);

    idx.remove_entity("id1").unwrap();
    let results = idx.search("searchable").unwrap();
    assert!(results.is_empty());
}

#[test]
fn search_empty_query() {
    let idx = SearchIndex::in_memory().unwrap();
    idx.index_entity("id1", "Note", "Test", "content").unwrap();
    let results = idx.search("").unwrap();
    assert!(results.is_empty());
}

#[test]
fn search_rebuild() {
    let idx = SearchIndex::in_memory().unwrap();
    let entity_store = EntityStore::in_memory().unwrap();
    let tmp = tempdir().unwrap();
    let content_store = ContentStore::new(tmp.path());

    // Insert some entities
    entity_store.insert(&make_entity_row("Program", "Rebuild Test")).unwrap();
    entity_store.insert(&make_entity_row("Task", "Another Task")).unwrap();

    let count = idx.rebuild(&entity_store, &content_store).unwrap();
    assert_eq!(count, 2);
}

// ===========================================================================
// Sync coordination
// ===========================================================================

#[test]
fn sync_save_entity_across_backends() {
    let tmp = tempdir().unwrap();
    let ws = Workspace::init(tmp.path()).unwrap();

    let content_store = ContentStore::new(&ws.project_root());
    let entity_store = EntityStore::open(&ws.data_path().join("entities.db")).unwrap();
    let extraction_store = ExtractionStore::open(&ws.data_path().join("extractions.db")).unwrap();
    let event_store = EventStore::new(&ws.data_path());
    let search_index = SearchIndex::open(&ws.index_path().join("search.db")).unwrap();
    let graph_store = GraphStore::open(&ws.graph_path().join("relations.db")).unwrap();

    let sync = StoreSync {
        content: &content_store,
        entities: &entity_store,
        extractions: &extraction_store,
        events: &event_store,
        search: &search_index,
        graph: &graph_store,
    };

    let row = make_entity_row("Note", "My Note");
    let id_str = row.id.clone();
    sync.save_entity(&row, Some("# My Note\n\nSome content."), EntityType::Note).unwrap();

    // Verify in entities.db
    assert!(entity_store.get(&id_str).unwrap().is_some());

    // Verify content file
    let id = uuid::Uuid::parse_str(&id_str).unwrap();
    let content = content_store.read_content(EntityType::Note, &EntityId::from(id)).unwrap();
    assert!(content.is_some());

    // Verify in search index
    let results = search_index.search("content").unwrap();
    assert!(!results.is_empty());

    // Verify in graph
    let eid = EntityId::from(uuid::Uuid::parse_str(&id_str).unwrap());
    assert!(graph_store.has_node(&eid).unwrap());

    // Verify event logged
    let events = event_store.read_all().unwrap();
    assert!(!events.is_empty());
}

// ===========================================================================
// Temporal materialization
// ===========================================================================

#[test]
fn sync_temporal_materialization() {
    let tmp = tempdir().unwrap();
    let ws = Workspace::init(tmp.path()).unwrap();

    let content_store = ContentStore::new(&ws.project_root());
    let entity_store = EntityStore::open(&ws.data_path().join("entities.db")).unwrap();
    let extraction_store = ExtractionStore::open(&ws.data_path().join("extractions.db")).unwrap();
    let event_store = EventStore::new(&ws.data_path());
    let search_index = SearchIndex::open(&ws.index_path().join("search.db")).unwrap();
    let graph_store = GraphStore::open(&ws.graph_path().join("relations.db")).unwrap();

    let sync = StoreSync {
        content: &content_store,
        entities: &entity_store,
        extractions: &extraction_store,
        events: &event_store,
        search: &search_index,
        graph: &graph_store,
    };

    // Save an entity first so the graph node exists
    let row = make_entity_row("Task", "Deadline Task");
    let id_str = row.id.clone();
    sync.save_entity(&row, None, EntityType::Task).unwrap();

    // Materialize temporal edges
    sync.materialize_temporal_edges(&id_str, EntityType::Task, false, true, false).unwrap();

    // Verify HAS_DEADLINE edge exists
    let eid = EntityId::from(uuid::Uuid::parse_str(&id_str).unwrap());
    assert!(graph_store.has_edge(&eid, &eid, RelationType::HasDeadline).unwrap());

    // HAS_CADENCE should not exist
    assert!(!graph_store.has_edge(&eid, &eid, RelationType::HasCadence).unwrap());
}
