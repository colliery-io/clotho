use serial_test::serial;
use tempfile::tempdir;

use clotho_core::domain::types::EntityId;
use clotho_core::graph::GraphStore;
use clotho_store::data::entities::EntityStore;
use clotho_store::workspace::Workspace;

use clotho_mcp_server::tools::{
    ClothoTools, CreateEntityTool, CreateRelationTool, DeleteEntityTool, GetRelationsTool,
    UpdateEntityTool,
};
use clotho_mcp_server::workspace_resolver;

// ===========================================================================
// Tool count
// ===========================================================================

#[test]
fn tools_now_fifteen() {
    let tools = ClothoTools::tools();
    assert_eq!(tools.len(), 28);
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"clotho_create_entity"));
    assert!(names.contains(&"clotho_update_entity"));
    assert!(names.contains(&"clotho_delete_entity"));
    assert!(names.contains(&"clotho_create_relation"));
    assert!(names.contains(&"clotho_delete_relation"));
    assert!(names.contains(&"clotho_get_relations"));
}

// ===========================================================================
// Create entity - structural types
// ===========================================================================

#[serial]
#[tokio::test]
async fn create_program() {
    let tmp = tempdir().unwrap();
    Workspace::init(tmp.path()).unwrap();
    workspace_resolver::set_workspace(tmp.path().display().to_string());

    let tool = CreateEntityTool {
        entity_type: "program".to_string(),
        title: "Technical Education".to_string(),
        status: None,
        state: None,
        email: None,
        url: None,
        parent_id: None,
        content: None,
    };
    let result = tool.call_tool().await.unwrap();
    assert!(result.is_error.is_none());

    let ws = Workspace::open(tmp.path()).unwrap();
    let store = EntityStore::open(&ws.data_path().join("entities.db")).unwrap();
    let programs = store.list_by_type("Program").unwrap();
    assert_eq!(programs.len(), 1);
    assert_eq!(programs[0].title, "Technical Education");
    assert_eq!(programs[0].status.as_deref(), Some("active"));
}

#[serial]
#[tokio::test]
async fn create_responsibility() {
    let tmp = tempdir().unwrap();
    Workspace::init(tmp.path()).unwrap();
    workspace_resolver::set_workspace(tmp.path().display().to_string());

    let tool = CreateEntityTool {
        entity_type: "responsibility".to_string(),
        title: "Team Mentorship".to_string(),
        status: None,
        state: None,
        email: None,
        url: None,
        parent_id: None,
        content: None,
    };
    tool.call_tool().await.unwrap();

    let ws = Workspace::open(tmp.path()).unwrap();
    let store = EntityStore::open(&ws.data_path().join("entities.db")).unwrap();
    let items = store.list_by_type("Responsibility").unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].status.as_deref(), Some("active"));
}

#[serial]
#[tokio::test]
async fn create_objective_with_parent() {
    let tmp = tempdir().unwrap();
    Workspace::init(tmp.path()).unwrap();
    workspace_resolver::set_workspace(tmp.path().display().to_string());

    // Create program first
    let prog = CreateEntityTool {
        entity_type: "program".to_string(),
        title: "PMO".to_string(),
        status: None,
        state: None,
        email: None,
        url: None,
        parent_id: None,
        content: None,
    };
    prog.call_tool().await.unwrap();

    let ws = Workspace::open(tmp.path()).unwrap();
    let store = EntityStore::open(&ws.data_path().join("entities.db")).unwrap();
    let programs = store.list_by_type("Program").unwrap();
    let prog_id = programs[0].id.clone();

    // Create objective under program
    let obj = CreateEntityTool {
        entity_type: "objective".to_string(),
        title: "Reduce deploy time".to_string(),
        status: None,
        state: None,
        email: None,
        url: None,
        parent_id: Some(prog_id.clone()),
        content: None,
    };
    obj.call_tool().await.unwrap();

    // Verify BELONGS_TO edge
    let graph = GraphStore::open(&ws.graph_path().join("relations.db")).unwrap();
    let objectives = store.list_by_type("Objective").unwrap();
    let obj_id: EntityId = uuid::Uuid::parse_str(&objectives[0].id).unwrap().into();
    let prog_eid: EntityId = uuid::Uuid::parse_str(&prog_id).unwrap().into();
    assert!(graph
        .has_edge(
            &obj_id,
            &prog_eid,
            clotho_core::domain::traits::RelationType::BelongsTo
        )
        .unwrap());
}

// ===========================================================================
// Create task with state
// ===========================================================================

#[serial]
#[tokio::test]
async fn create_task_defaults_to_todo() {
    let tmp = tempdir().unwrap();
    Workspace::init(tmp.path()).unwrap();
    workspace_resolver::set_workspace(tmp.path().display().to_string());

    let tool = CreateEntityTool {
        entity_type: "task".to_string(),
        title: "Write RFC".to_string(),
        status: None,
        state: None,
        email: None,
        url: None,
        parent_id: None,
        content: None,
    };
    tool.call_tool().await.unwrap();

    let ws = Workspace::open(tmp.path()).unwrap();
    let store = EntityStore::open(&ws.data_path().join("entities.db")).unwrap();
    let tasks = store.list_by_type("Task").unwrap();
    assert_eq!(tasks[0].task_state.as_deref(), Some("todo"));
}

// ===========================================================================
// Create person with email
// ===========================================================================

#[serial]
#[tokio::test]
async fn create_person_with_email() {
    let tmp = tempdir().unwrap();
    Workspace::init(tmp.path()).unwrap();
    workspace_resolver::set_workspace(tmp.path().display().to_string());

    let tool = CreateEntityTool {
        entity_type: "person".to_string(),
        title: "Alice".to_string(),
        status: None,
        state: None,
        email: Some("alice@example.com".to_string()),
        url: None,
        parent_id: None,
        content: None,
    };
    tool.call_tool().await.unwrap();

    let ws = Workspace::open(tmp.path()).unwrap();
    let store = EntityStore::open(&ws.data_path().join("entities.db")).unwrap();
    let people = store.list_by_type("Person").unwrap();
    assert_eq!(people.len(), 1);
    assert!(people[0]
        .metadata
        .as_ref()
        .unwrap()
        .contains("alice@example.com"));
}

// ===========================================================================
// Update entity
// ===========================================================================

#[serial]
#[tokio::test]
async fn update_entity_title() {
    let tmp = tempdir().unwrap();
    Workspace::init(tmp.path()).unwrap();
    workspace_resolver::set_workspace(tmp.path().display().to_string());

    let create = CreateEntityTool {
        entity_type: "program".to_string(),
        title: "Original".to_string(),
        status: None,
        state: None,
        email: None,
        url: None,
        parent_id: None,
        content: None,
    };
    create.call_tool().await.unwrap();

    let ws = Workspace::open(tmp.path()).unwrap();
    let store = EntityStore::open(&ws.data_path().join("entities.db")).unwrap();
    let id = store.list_all().unwrap()[0].id.clone();

    let update = UpdateEntityTool {
        entity_id: id.clone(),
        title: Some("Updated".to_string()),
        status: None,
        state: None,
        content: None,
        email: None,
        url: None,
    };
    update.call_tool().await.unwrap();

    let row = store.get(&id).unwrap().unwrap();
    assert_eq!(row.title, "Updated");
}

// ===========================================================================
// Delete entity
// ===========================================================================

#[serial]
#[tokio::test]
async fn delete_entity_removes_from_all() {
    let tmp = tempdir().unwrap();
    Workspace::init(tmp.path()).unwrap();
    workspace_resolver::set_workspace(tmp.path().display().to_string());

    let create = CreateEntityTool {
        entity_type: "note".to_string(),
        title: "Doomed Note".to_string(),
        status: None,
        state: None,
        email: None,
        url: None,
        parent_id: None,
        content: Some("Will be deleted".to_string()),
    };
    create.call_tool().await.unwrap();

    let ws = Workspace::open(tmp.path()).unwrap();
    let store = EntityStore::open(&ws.data_path().join("entities.db")).unwrap();
    let id = store.list_all().unwrap()[0].id.clone();

    let delete = DeleteEntityTool {
        entity_id: id.clone(),
    };
    delete.call_tool().await.unwrap();

    assert!(store.get(&id).unwrap().is_none());

    let graph = GraphStore::open(&ws.graph_path().join("relations.db")).unwrap();
    let eid: EntityId = uuid::Uuid::parse_str(&id).unwrap().into();
    assert!(!graph.has_node(&eid).unwrap());
}

// ===========================================================================
// Relations
// ===========================================================================

#[serial]
#[tokio::test]
async fn create_and_get_relations() {
    let tmp = tempdir().unwrap();
    Workspace::init(tmp.path()).unwrap();
    workspace_resolver::set_workspace(tmp.path().display().to_string());

    // Create two entities
    for (t, title) in &[("program", "PMO"), ("task", "Review docs")] {
        let tool = CreateEntityTool {
            entity_type: t.to_string(),
            title: title.to_string(),
            status: None,
            state: None,
            email: None,
            url: None,
            parent_id: None,
            content: None,
        };
        tool.call_tool().await.unwrap();
    }

    let ws = Workspace::open(tmp.path()).unwrap();
    let store = EntityStore::open(&ws.data_path().join("entities.db")).unwrap();
    let all = store.list_all().unwrap();
    let prog_id = all
        .iter()
        .find(|r| r.entity_type == "Program")
        .unwrap()
        .id
        .clone();
    let task_id = all
        .iter()
        .find(|r| r.entity_type == "Task")
        .unwrap()
        .id
        .clone();

    // Create relation
    let relate = CreateRelationTool {
        source_id: task_id.clone(),
        relation_type: "belongs_to".to_string(),
        target_id: prog_id.clone(),
    };
    relate.call_tool().await.unwrap();

    // Get relations
    let rels = GetRelationsTool {
        entity_id: task_id.clone(),
    };
    let result = rels.call_tool().await.unwrap();
    let text = format!("{:?}", result.content);
    assert!(text.contains("BelongsTo"));
}
