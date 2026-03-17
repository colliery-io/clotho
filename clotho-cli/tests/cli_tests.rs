use std::fs;

use tempfile::tempdir;

use clotho_core::domain::types::{EntityId, EntityType};
use clotho_core::graph::GraphStore;
use clotho_store::content::ContentStore;
use clotho_store::data::entities::EntityStore;
use clotho_store::index::SearchIndex;
use clotho_store::workspace::Workspace;

/// Helper: initialize a workspace and return the temp dir + workspace.
fn setup_workspace() -> (tempfile::TempDir, Workspace) {
    let tmp = tempdir().unwrap();
    let ws = Workspace::init(tmp.path()).unwrap();
    (tmp, ws)
}

/// Helper: create a sample file in the temp dir.
fn create_sample_file(tmp: &tempfile::TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = tmp.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

// ===========================================================================
// Init
// ===========================================================================

#[test]
fn test_init_creates_workspace() {
    let tmp = tempdir().unwrap();
    let ws = Workspace::init(tmp.path()).unwrap();
    assert!(ws.path.exists());
    assert!(ws.content_path().join("notes").is_dir());
    assert!(ws.config_path().join("config.toml").is_file());
}

#[test]
fn test_init_fails_if_exists() {
    let tmp = tempdir().unwrap();
    Workspace::init(tmp.path()).unwrap();
    assert!(Workspace::init(tmp.path()).is_err());
}

// ===========================================================================
// Ingest
// ===========================================================================

#[test]
fn test_ingest_stores_content_and_entity() {
    let (tmp, ws) = setup_workspace();
    let file = create_sample_file(&tmp, "meeting-notes.md", "# Standup\n\nDiscussed deployment timeline.");

    // Simulate what the ingest command does
    let content = fs::read_to_string(&file).unwrap();
    let id = EntityId::new();
    let now = chrono::Utc::now();

    let content_store = ContentStore::new(&ws.path);
    let content_path = content_store.write_content(EntityType::Note, &id, &content).unwrap();
    assert!(content_path.exists());

    let entity_store = EntityStore::open(&ws.data_path().join("entities.db")).unwrap();
    let row = clotho_store::data::entities::EntityRow {
        id: id.to_string(),
        entity_type: "Note".to_string(),
        title: "meeting-notes".to_string(),
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
        status: Some("active".to_string()),
        task_state: None,
        extraction_status: None,
        source_transcript_id: None,
        source_span_start: None,
        source_span_end: None,
        confidence: None,
        content_path: Some(content_path.display().to_string()),
        metadata: None,
    };
    entity_store.insert(&row).unwrap();

    // Verify entity is stored
    let got = entity_store.get(&id.to_string()).unwrap().unwrap();
    assert_eq!(got.title, "meeting-notes");
    assert_eq!(got.entity_type, "Note");

    // Verify content is readable
    let read = content_store.read_content(EntityType::Note, &id).unwrap().unwrap();
    assert!(read.contains("deployment timeline"));
}

#[test]
fn test_ingest_registers_graph_node() {
    let (_tmp, ws) = setup_workspace();
    let id = EntityId::new();

    let graph = GraphStore::open(&ws.graph_path().join("relations.db")).unwrap();
    graph.register_node(&id, EntityType::Meeting, "Standup").unwrap();
    assert!(graph.has_node(&id).unwrap());
}

#[test]
fn test_ingest_indexes_in_fts5() {
    let (_tmp, ws) = setup_workspace();
    let id = EntityId::new();

    let index = SearchIndex::open(&ws.index_path().join("search.db")).unwrap();
    index.index_entity(&id.to_string(), "Note", "Architecture", "microservice patterns and event sourcing").unwrap();

    let results = index.search("microservice").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Architecture");
}

// ===========================================================================
// List
// ===========================================================================

#[test]
fn test_list_entities() {
    let (_tmp, ws) = setup_workspace();
    let store = EntityStore::open(&ws.data_path().join("entities.db")).unwrap();

    let now = chrono::Utc::now().to_rfc3339();
    store.insert(&clotho_store::data::entities::EntityRow {
        id: EntityId::new().to_string(),
        entity_type: "Task".to_string(),
        title: "Fix bug".to_string(),
        created_at: now.clone(),
        updated_at: now.clone(),
        status: None,
        task_state: Some("todo".to_string()),
        extraction_status: None,
        source_transcript_id: None,
        source_span_start: None,
        source_span_end: None,
        confidence: None,
        content_path: None,
        metadata: None,
    }).unwrap();

    store.insert(&clotho_store::data::entities::EntityRow {
        id: EntityId::new().to_string(),
        entity_type: "Program".to_string(),
        title: "PMO".to_string(),
        created_at: now.clone(),
        updated_at: now.clone(),
        status: Some("active".to_string()),
        task_state: None,
        extraction_status: None,
        source_transcript_id: None,
        source_span_start: None,
        source_span_end: None,
        confidence: None,
        content_path: None,
        metadata: None,
    }).unwrap();

    let all = store.list_all().unwrap();
    assert_eq!(all.len(), 2);

    let tasks = store.list_by_type("Task").unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Fix bug");

    let active = store.list_by_status("active").unwrap();
    assert_eq!(active.len(), 1);

    let todo = store.list_by_state("todo").unwrap();
    assert_eq!(todo.len(), 1);
}

// ===========================================================================
// Search
// ===========================================================================

#[test]
fn test_search_finds_content() {
    let (_tmp, ws) = setup_workspace();
    let index = SearchIndex::open(&ws.index_path().join("search.db")).unwrap();

    index.index_entity("id1", "Note", "Deploy Plan", "Rolling deployment strategy with blue-green").unwrap();
    index.index_entity("id2", "Meeting", "Standup", "Quick sync on sprint progress").unwrap();

    let results = index.search("deployment").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entity_id, "id1");

    let results = index.search("sprint").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entity_id, "id2");
}

#[test]
fn test_search_empty_results() {
    let (_tmp, ws) = setup_workspace();
    let index = SearchIndex::open(&ws.index_path().join("search.db")).unwrap();
    index.index_entity("id1", "Note", "Test", "something").unwrap();

    let results = index.search("nonexistent").unwrap();
    assert!(results.is_empty());
}

// ===========================================================================
// Query
// ===========================================================================

#[test]
fn test_query_cypher() {
    let (_tmp, ws) = setup_workspace();
    let graph = GraphStore::open(&ws.graph_path().join("relations.db")).unwrap();

    let id = EntityId::new();
    graph.register_node(&id, EntityType::Program, "Test Program").unwrap();

    let result = graph.raw_cypher(&format!(
        "MATCH (n {{id: '{}'}}) RETURN n.title AS title",
        id
    )).unwrap();

    assert_eq!(result.len(), 1);
    let title: String = result[0].get("title").unwrap_or_default();
    assert_eq!(title, "Test Program");
}

// ===========================================================================
// Reflect
// ===========================================================================

#[test]
fn test_reflect_creates_entity_and_content() {
    let (_tmp, ws) = setup_workspace();
    let id = EntityId::new();
    let now = chrono::Utc::now();

    // Create content
    let content_store = ContentStore::new(&ws.path);
    let template = "# Weekly Reflection\n\n## Reflections\n\n## Key Takeaways\n";
    let path = content_store.write_content(EntityType::Reflection, &id, template).unwrap();
    assert!(path.exists());

    // Create entity
    let entity_store = EntityStore::open(&ws.data_path().join("entities.db")).unwrap();
    entity_store.insert(&clotho_store::data::entities::EntityRow {
        id: id.to_string(),
        entity_type: "Reflection".to_string(),
        title: "Weekly reflection".to_string(),
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
        status: Some("active".to_string()),
        task_state: None,
        extraction_status: None,
        source_transcript_id: None,
        source_span_start: None,
        source_span_end: None,
        confidence: None,
        content_path: Some(path.display().to_string()),
        metadata: Some(r#"{"period_type":"weekly"}"#.to_string()),
    }).unwrap();

    let got = entity_store.get(&id.to_string()).unwrap().unwrap();
    assert_eq!(got.entity_type, "Reflection");

    let content = content_store.read_content(EntityType::Reflection, &id).unwrap().unwrap();
    assert!(content.contains("Key Takeaways"));
}
