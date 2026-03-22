use clotho_core::domain::entities::execution::Task;
use clotho_core::domain::entities::structural::Program;
use clotho_core::domain::traits::*;
use clotho_core::domain::types::*;
use clotho_core::graph::GraphStore;

fn setup() -> GraphStore {
    GraphStore::in_memory().expect("failed to create in-memory graph")
}

// ===========================================================================
// GraphStore lifecycle
// ===========================================================================

#[test]
fn graph_store_in_memory_empty() {
    let gs = setup();
    let stats = gs.stats().unwrap();
    assert_eq!(stats.node_count, 0);
    assert_eq!(stats.edge_count, 0);
}

// ===========================================================================
// Node CRUD
// ===========================================================================

#[test]
fn register_and_get_node() {
    let gs = setup();
    let id = EntityId::new();
    gs.register_node(&id, EntityType::Program, "Test Program")
        .unwrap();

    assert!(gs.has_node(&id).unwrap());

    let info = gs.get_node(&id).unwrap().expect("node should exist");
    assert_eq!(info.id, id);
    assert_eq!(info.entity_type, EntityType::Program);
    assert_eq!(info.title, "Test Program");
}

#[test]
fn has_node_false_for_missing() {
    let gs = setup();
    let id = EntityId::new();
    assert!(!gs.has_node(&id).unwrap());
}

#[test]
fn get_node_returns_none_for_missing() {
    let gs = setup();
    let id = EntityId::new();
    assert!(gs.get_node(&id).unwrap().is_none());
}

#[test]
fn remove_node() {
    let gs = setup();
    let id = EntityId::new();
    gs.register_node(&id, EntityType::Task, "Do thing").unwrap();
    assert!(gs.has_node(&id).unwrap());

    gs.remove_node(&id).unwrap();
    assert!(!gs.has_node(&id).unwrap());
}

#[test]
fn upsert_node_updates_title() {
    let gs = setup();
    let id = EntityId::new();
    gs.register_node(&id, EntityType::Program, "Original")
        .unwrap();
    gs.register_node(&id, EntityType::Program, "Updated")
        .unwrap();

    let info = gs.get_node(&id).unwrap().expect("node should exist");
    assert_eq!(info.title, "Updated");

    // Should still be one node, not two
    let stats = gs.stats().unwrap();
    assert_eq!(stats.node_count, 1);
}

// ===========================================================================
// Edge CRUD
// ===========================================================================

#[test]
fn add_and_has_edge() {
    let gs = setup();
    let src = EntityId::new();
    let tgt = EntityId::new();
    gs.register_node(&src, EntityType::Task, "Task A").unwrap();
    gs.register_node(&tgt, EntityType::Program, "Program X")
        .unwrap();

    gs.add_edge(&src, &tgt, RelationType::BelongsTo).unwrap();
    assert!(gs.has_edge(&src, &tgt, RelationType::BelongsTo).unwrap());
}

#[test]
fn has_edge_false_when_missing() {
    let gs = setup();
    let src = EntityId::new();
    let tgt = EntityId::new();
    gs.register_node(&src, EntityType::Task, "Task A").unwrap();
    gs.register_node(&tgt, EntityType::Program, "Program X")
        .unwrap();

    assert!(!gs.has_edge(&src, &tgt, RelationType::BelongsTo).unwrap());
}

#[test]
fn remove_edge() {
    let gs = setup();
    let src = EntityId::new();
    let tgt = EntityId::new();
    gs.register_node(&src, EntityType::Task, "Task A").unwrap();
    gs.register_node(&tgt, EntityType::Program, "Program X")
        .unwrap();

    gs.add_edge(&src, &tgt, RelationType::BelongsTo).unwrap();
    assert!(gs.has_edge(&src, &tgt, RelationType::BelongsTo).unwrap());

    gs.remove_edge(&src, &tgt, RelationType::BelongsTo).unwrap();
    assert!(!gs.has_edge(&src, &tgt, RelationType::BelongsTo).unwrap());
}

#[test]
fn get_edges_from() {
    let gs = setup();
    let src = EntityId::new();
    let tgt1 = EntityId::new();
    let tgt2 = EntityId::new();
    gs.register_node(&src, EntityType::Transcript, "Transcript")
        .unwrap();
    gs.register_node(&tgt1, EntityType::Person, "Alice")
        .unwrap();
    gs.register_node(&tgt2, EntityType::Program, "PMO").unwrap();

    gs.add_edge(&src, &tgt1, RelationType::Mentions).unwrap();
    gs.add_edge(&src, &tgt2, RelationType::Mentions).unwrap();

    let edges = gs.get_edges_from(&src).unwrap();
    assert_eq!(edges.len(), 2);
    assert!(edges
        .iter()
        .all(|e| e.relation_type == RelationType::Mentions));
}

#[test]
fn get_edges_by_type_filters() {
    let gs = setup();
    let src = EntityId::new();
    let tgt1 = EntityId::new();
    let tgt2 = EntityId::new();
    gs.register_node(&src, EntityType::Task, "Task").unwrap();
    gs.register_node(&tgt1, EntityType::Program, "Program")
        .unwrap();
    gs.register_node(&tgt2, EntityType::Blocker, "Blocker")
        .unwrap();

    gs.add_edge(&src, &tgt1, RelationType::BelongsTo).unwrap();
    gs.add_edge(&src, &tgt2, RelationType::BlockedBy).unwrap();

    let belongs = gs.get_edges_by_type(&src, RelationType::BelongsTo).unwrap();
    assert_eq!(belongs.len(), 1);
    assert_eq!(belongs[0].target_id, tgt1);

    let blocked = gs.get_edges_by_type(&src, RelationType::BlockedBy).unwrap();
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0].target_id, tgt2);
}

#[test]
fn get_edges_to() {
    let gs = setup();
    let src1 = EntityId::new();
    let src2 = EntityId::new();
    let tgt = EntityId::new();
    gs.register_node(&src1, EntityType::Task, "Task 1").unwrap();
    gs.register_node(&src2, EntityType::Task, "Task 2").unwrap();
    gs.register_node(&tgt, EntityType::Program, "Program")
        .unwrap();

    gs.add_edge(&src1, &tgt, RelationType::BelongsTo).unwrap();
    gs.add_edge(&src2, &tgt, RelationType::BelongsTo).unwrap();

    let edges = gs.get_edges_to(&tgt).unwrap();
    assert_eq!(edges.len(), 2);
}

// ===========================================================================
// Edge with properties
// ===========================================================================

#[test]
fn add_edge_with_props() {
    let gs = setup();
    let src = EntityId::new();
    let tgt = EntityId::new();
    gs.register_node(&src, EntityType::Task, "Task").unwrap();
    gs.register_node(&tgt, EntityType::Task, "Deadline Node")
        .unwrap();

    let props = vec![("deadline".to_string(), "2025-06-01".to_string())];
    gs.add_edge_with_props(&src, &tgt, RelationType::HasDeadline, props)
        .unwrap();
    assert!(gs.has_edge(&src, &tgt, RelationType::HasDeadline).unwrap());
}

// ===========================================================================
// Relatable trait integration
// ===========================================================================

#[test]
fn relatable_returns_real_edges() {
    let gs = setup();
    let program = Program::new("Test Program");
    let task = Task::new("Test Task");

    gs.register_node(program.id(), EntityType::Program, program.title())
        .unwrap();
    gs.register_node(task.id(), EntityType::Task, task.title())
        .unwrap();
    gs.add_edge(task.id(), program.id(), RelationType::BelongsTo)
        .unwrap();

    let relations = task.relations(&gs);
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].relation_type, RelationType::BelongsTo);
    assert_eq!(relations[0].target_id, *program.id());
}

#[test]
fn relatable_empty_when_no_edges() {
    let gs = setup();
    let program = Program::new("Orphan Program");
    gs.register_node(program.id(), EntityType::Program, program.title())
        .unwrap();

    let relations = program.relations(&gs);
    assert!(relations.is_empty());
}

// ===========================================================================
// Query helpers
// ===========================================================================

#[test]
fn get_neighbors() {
    let gs = setup();
    let a = EntityId::new();
    let b = EntityId::new();
    let c = EntityId::new();
    gs.register_node(&a, EntityType::Program, "A").unwrap();
    gs.register_node(&b, EntityType::Task, "B").unwrap();
    gs.register_node(&c, EntityType::Person, "C").unwrap();

    gs.add_edge(&a, &b, RelationType::RelatesTo).unwrap();
    gs.add_edge(&c, &a, RelationType::Mentions).unwrap();

    let neighbors = gs.get_neighbors(&a).unwrap();
    assert_eq!(neighbors.len(), 2);
}

#[test]
fn get_related_by_type() {
    let gs = setup();
    let meeting = EntityId::new();
    let decision = EntityId::new();
    let risk = EntityId::new();
    gs.register_node(&meeting, EntityType::Meeting, "Standup")
        .unwrap();
    gs.register_node(&decision, EntityType::Decision, "Go with A")
        .unwrap();
    gs.register_node(&risk, EntityType::Risk, "Budget risk")
        .unwrap();

    gs.add_edge(&meeting, &decision, RelationType::HasDecision)
        .unwrap();
    gs.add_edge(&meeting, &risk, RelationType::HasRisk).unwrap();

    let decisions = gs
        .get_related_by_type(&meeting, RelationType::HasDecision)
        .unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].entity_type, EntityType::Decision);
}

#[test]
fn get_incoming_by_type() {
    let gs = setup();
    let program = EntityId::new();
    let task1 = EntityId::new();
    let task2 = EntityId::new();
    gs.register_node(&program, EntityType::Program, "PMO")
        .unwrap();
    gs.register_node(&task1, EntityType::Task, "Task 1")
        .unwrap();
    gs.register_node(&task2, EntityType::Task, "Task 2")
        .unwrap();

    gs.add_edge(&task1, &program, RelationType::BelongsTo)
        .unwrap();
    gs.add_edge(&task2, &program, RelationType::BelongsTo)
        .unwrap();

    let tasks = gs
        .get_incoming_by_type(&program, RelationType::BelongsTo)
        .unwrap();
    assert_eq!(tasks.len(), 2);
}

#[test]
fn get_entities_by_label() {
    let gs = setup();
    gs.register_node(&EntityId::new(), EntityType::Task, "Task 1")
        .unwrap();
    gs.register_node(&EntityId::new(), EntityType::Task, "Task 2")
        .unwrap();
    gs.register_node(&EntityId::new(), EntityType::Program, "Program")
        .unwrap();

    let tasks = gs.get_entities_by_label(EntityType::Task).unwrap();
    assert_eq!(tasks.len(), 2);

    let programs = gs.get_entities_by_label(EntityType::Program).unwrap();
    assert_eq!(programs.len(), 1);
}

// ===========================================================================
// Raw Cypher
// ===========================================================================

#[test]
fn raw_cypher_query() {
    let gs = setup();
    let id = EntityId::new();
    gs.register_node(&id, EntityType::Program, "Test").unwrap();

    let result = gs
        .raw_cypher(&format!(
            "MATCH (n {{id: '{}'}}) RETURN n.title AS title",
            id
        ))
        .unwrap();

    assert_eq!(result.len(), 1);
    let title: String = result[0].get("title").unwrap_or_default();
    assert_eq!(title, "Test");
}

// ===========================================================================
// Node removal cascades edges
// ===========================================================================

#[test]
fn remove_node_cascades_edges() {
    let gs = setup();
    let src = EntityId::new();
    let tgt = EntityId::new();
    gs.register_node(&src, EntityType::Task, "Task").unwrap();
    gs.register_node(&tgt, EntityType::Program, "Program")
        .unwrap();
    gs.add_edge(&src, &tgt, RelationType::BelongsTo).unwrap();

    assert!(gs.has_edge(&src, &tgt, RelationType::BelongsTo).unwrap());

    gs.remove_node(&src).unwrap();
    // Edge should be gone since source was removed (DETACH DELETE)
    assert!(!gs.has_edge(&src, &tgt, RelationType::BelongsTo).unwrap());
}

// ===========================================================================
// Stats
// ===========================================================================

#[test]
fn stats_counts() {
    let gs = setup();
    let a = EntityId::new();
    let b = EntityId::new();
    gs.register_node(&a, EntityType::Program, "A").unwrap();
    gs.register_node(&b, EntityType::Task, "B").unwrap();
    gs.add_edge(&a, &b, RelationType::RelatesTo).unwrap();

    let stats = gs.stats().unwrap();
    assert_eq!(stats.node_count, 2);
    assert_eq!(stats.edge_count, 1);
}
