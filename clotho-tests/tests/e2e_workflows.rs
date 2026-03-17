//! End-to-end workflow tests exercising the full Clotho system.
//!
//! These simulate real usage: init workspace, create structural entities,
//! ingest content, build the graph, search, query, reflect, sync.

use std::fs;

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
use clotho_store::workspace::Workspace;
use clotho_sync::SyncEngine;

/// Helper: init workspace + all stores, returns everything needed.
struct TestWorkspace {
    _tmp: tempfile::TempDir,
    ws: Workspace,
}

impl TestWorkspace {
    fn new() -> Self {
        let tmp = tempdir().unwrap();
        let ws = Workspace::init(tmp.path()).unwrap();
        Self { _tmp: tmp, ws }
    }

    fn entities(&self) -> EntityStore {
        EntityStore::open(&self.ws.data_path().join("entities.db")).unwrap()
    }

    fn extractions(&self) -> ExtractionStore {
        ExtractionStore::open(&self.ws.data_path().join("extractions.db")).unwrap()
    }

    fn content(&self) -> ContentStore {
        ContentStore::new(&self.ws.path)
    }

    fn graph(&self) -> GraphStore {
        GraphStore::open(&self.ws.graph_path().join("relations.db")).unwrap()
    }

    fn search(&self) -> SearchIndex {
        SearchIndex::open(&self.ws.index_path().join("search.db")).unwrap()
    }

    fn tags(&self) -> TagStore {
        TagStore::new(&self.ws.data_path())
    }

    fn events(&self) -> EventStore {
        EventStore::new(&self.ws.data_path())
    }

    fn sync_engine(&self) -> SyncEngine {
        SyncEngine::init(&self.ws.path).unwrap()
    }

    fn insert_entity(&self, entity_type: &str, title: &str) -> String {
        let id = EntityId::new();
        let now = Utc::now();
        let et = parse_entity_type(entity_type);

        let (status, state, extraction) = defaults(et);

        self.entities().insert(&EntityRow {
            id: id.to_string(),
            entity_type: entity_type.to_string(),
            title: title.to_string(),
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            status, task_state: state, extraction_status: extraction,
            source_transcript_id: None, source_span_start: None,
            source_span_end: None, confidence: None,
            content_path: None, metadata: None,
        }).unwrap();

        self.graph().register_node(&id, et, title).unwrap();
        self.search().index_entity(&id.to_string(), entity_type, title, "").unwrap();

        id.to_string()
    }

    fn insert_entity_with_content(&self, entity_type: &str, title: &str, content: &str) -> String {
        let id = EntityId::new();
        let now = Utc::now();
        let et = parse_entity_type(entity_type);

        let (status, state, extraction) = defaults(et);

        let content_path = self.content().write_content(et, &id, content).unwrap();
        self.entities().insert(&EntityRow {
            id: id.to_string(),
            entity_type: entity_type.to_string(),
            title: title.to_string(),
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            status, task_state: state, extraction_status: extraction,
            source_transcript_id: None, source_span_start: None,
            source_span_end: None, confidence: None,
            content_path: Some(content_path.display().to_string()),
            metadata: None,
        }).unwrap();

        self.graph().register_node(&id, et, title).unwrap();
        self.search().index_entity(&id.to_string(), entity_type, title, content).unwrap();

        id.to_string()
    }

    fn relate(&self, source: &str, rel_type: RelationType, target: &str) {
        let src: EntityId = uuid::Uuid::parse_str(source).unwrap().into();
        let tgt: EntityId = uuid::Uuid::parse_str(target).unwrap().into();
        self.graph().add_edge(&src, &tgt, rel_type).unwrap();
    }
}

fn parse_entity_type(s: &str) -> EntityType {
    match s {
        "Program" => EntityType::Program,
        "Responsibility" => EntityType::Responsibility,
        "Objective" => EntityType::Objective,
        "Workstream" => EntityType::Workstream,
        "Task" => EntityType::Task,
        "Meeting" => EntityType::Meeting,
        "Transcript" => EntityType::Transcript,
        "Note" => EntityType::Note,
        "Reflection" => EntityType::Reflection,
        "Artifact" => EntityType::Artifact,
        "Decision" => EntityType::Decision,
        "Risk" => EntityType::Risk,
        "Blocker" => EntityType::Blocker,
        "Question" => EntityType::Question,
        "Insight" => EntityType::Insight,
        "Person" => EntityType::Person,
        _ => panic!("unknown type: {}", s),
    }
}

fn defaults(et: EntityType) -> (Option<String>, Option<String>, Option<String>) {
    match et {
        EntityType::Program | EntityType::Responsibility | EntityType::Objective | EntityType::Workstream
            => (Some("active".into()), None, None),
        EntityType::Task => (None, Some("todo".into()), None),
        EntityType::Decision | EntityType::Risk | EntityType::Blocker | EntityType::Question | EntityType::Insight
            => (None, None, Some("draft".into())),
        _ => (None, None, None),
    }
}

// ===========================================================================
// Scenario 1: Work management lifecycle
//
// Create a program → objectives → tasks → transition states → tag → query
// ===========================================================================

#[test]
fn scenario_work_management_lifecycle() {
    let tw = TestWorkspace::new();

    // Create structural layer
    let prog_id = tw.insert_entity("Program", "Monolith Breakup");
    let obj_id = tw.insert_entity("Objective", "Reduce deploy time to 5min");
    tw.relate(&obj_id, RelationType::BelongsTo, &prog_id);

    // Create execution layer
    let ws_id = tw.insert_entity("Workstream", "API Redesign");
    tw.relate(&ws_id, RelationType::RelatesTo, &prog_id);

    let task1_id = tw.insert_entity("Task", "Write migration RFC");
    let task2_id = tw.insert_entity("Task", "Implement API v2 endpoints");
    let task3_id = tw.insert_entity("Task", "Deploy canary to staging");
    tw.relate(&task1_id, RelationType::BelongsTo, &prog_id);
    tw.relate(&task2_id, RelationType::BelongsTo, &prog_id);
    tw.relate(&task3_id, RelationType::BelongsTo, &prog_id);

    // Create a person
    let _alice_id = tw.insert_entity("Person", "Alice");

    // Tag entities
    tw.tags().add_tag(&task1_id, "urgent").unwrap();
    tw.tags().add_tag(&task2_id, "api").unwrap();
    tw.tags().add_tag(&task3_id, "api").unwrap();

    // Verify structural relationships
    let prog_eid: EntityId = uuid::Uuid::parse_str(&prog_id).unwrap().into();
    let incoming = tw.graph().get_incoming_by_type(&prog_eid, RelationType::BelongsTo).unwrap();
    assert_eq!(incoming.len(), 4); // obj + 3 tasks

    // Verify tags
    let api_entities = tw.tags().get_entities_by_tag("api").unwrap();
    assert_eq!(api_entities.len(), 2);

    // Transition task states
    let store = tw.entities();
    let mut row = store.get(&task1_id).unwrap().unwrap();
    assert_eq!(row.task_state.as_deref(), Some("todo"));
    row.task_state = Some("doing".to_string());
    row.updated_at = Utc::now().to_rfc3339();
    store.update(&row).unwrap();

    // List doing tasks
    let doing = store.list_by_state("doing").unwrap();
    assert_eq!(doing.len(), 1);
    assert_eq!(doing[0].title, "Write migration RFC");

    // List all tasks
    let all_tasks = store.list_by_type("Task").unwrap();
    assert_eq!(all_tasks.len(), 3);

    // Graph query: what belongs to the program?
    let neighbors = tw.graph().get_incoming_by_type(&prog_eid, RelationType::BelongsTo).unwrap();
    assert_eq!(neighbors.len(), 4);

    // Search
    let results = tw.search().search("migration").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Write migration RFC");
}

// ===========================================================================
// Scenario 2: Content capture & search
//
// Ingest transcripts → create notes → search across content → relate to meetings
// ===========================================================================

#[test]
fn scenario_content_capture_and_search() {
    let tw = TestWorkspace::new();

    // Create a meeting
    let meeting_id = tw.insert_entity_with_content(
        "Meeting",
        "2025-01-15 Architecture Review",
        "# Architecture Review\n\nAttendees: Alice, Bob, Carol\n\nDiscussed the migration from monolith to microservices.\nKey concern: database schema compatibility during transition.",
    );

    // Create a transcript for the meeting
    let transcript_id = tw.insert_entity_with_content(
        "Transcript",
        "2025-01-15 Architecture Review Transcript",
        "Alice: I think we should go with a strangler fig pattern for the migration.\nBob: The concern is that our shared database makes this harder than a greenfield split.\nCarol: What if we start with the user service? It has the least coupling.\nAlice: Good idea. I'll write up an RFC for that approach.\nBob: We need to figure out how to handle the session store during the transition.",
    );
    tw.relate(&transcript_id, RelationType::SpawnedFrom, &meeting_id);

    // Create standalone notes
    let _note1_id = tw.insert_entity_with_content(
        "Note",
        "Microservice patterns research",
        "# Microservice Patterns\n\n## Strangler Fig\nGradually replace monolith components.\n\n## Database per Service\nEach service owns its data.\n\n## Event Sourcing\nUse events as source of truth.",
    );

    let _note2_id = tw.insert_entity_with_content(
        "Note",
        "Deployment strategy notes",
        "# Deployment Strategy\n\nBlue-green deployment with canary releases.\nRollback plan: revert to previous container image.\nMonitoring: Grafana dashboards for latency and error rates.",
    );

    // Create people mentioned
    let alice_id = tw.insert_entity("Person", "Alice");
    let bob_id = tw.insert_entity("Person", "Bob");
    tw.relate(&transcript_id, RelationType::Mentions, &alice_id);
    tw.relate(&transcript_id, RelationType::Mentions, &bob_id);

    // Search across all content
    let strangler = tw.search().search("strangler").unwrap();
    assert!(strangler.len() >= 1); // Should find in transcript and/or notes

    let database = tw.search().search("database").unwrap();
    assert!(database.len() >= 1);

    let canary = tw.search().search("canary").unwrap();
    assert_eq!(canary.len(), 1);
    assert_eq!(canary[0].title, "Deployment strategy notes");

    let grafana = tw.search().search("grafana").unwrap();
    assert_eq!(grafana.len(), 1);

    // Verify transcript → meeting relation
    let t_eid: EntityId = uuid::Uuid::parse_str(&transcript_id).unwrap().into();
    let spawned = tw.graph().get_edges_by_type(&t_eid, RelationType::SpawnedFrom).unwrap();
    assert_eq!(spawned.len(), 1);

    // Verify mentions
    let mentions = tw.graph().get_edges_by_type(&t_eid, RelationType::Mentions).unwrap();
    assert_eq!(mentions.len(), 2);

    // Verify content is readable
    let content = tw.content().read_content(
        EntityType::Transcript,
        &uuid::Uuid::parse_str(&transcript_id).unwrap().into(),
    ).unwrap().unwrap();
    assert!(content.contains("strangler fig"));
}

// ===========================================================================
// Scenario 3: Extraction lifecycle (draft → promote/discard)
//
// Simulate what the AI extraction pipeline would produce
// ===========================================================================

#[test]
fn scenario_extraction_lifecycle() {
    let tw = TestWorkspace::new();

    // Create a transcript first
    let transcript_id = tw.insert_entity_with_content(
        "Transcript",
        "Sprint Planning Transcript",
        "Alice: We're going with option B for the API design.\nBob: The concern is rate limiting at scale.\nCarol: I'll take the auth service refactor.",
    );

    // Simulate AI extractions as draft entities
    let extraction_store = tw.extractions();
    let now = Utc::now().to_rfc3339();

    let decision_id = EntityId::new().to_string();
    extraction_store.insert_draft(&ExtractionRow {
        id: decision_id.clone(),
        entity_type: "Decision".to_string(),
        title: "Going with option B for API design".to_string(),
        speech_act: Some("decide".to_string()),
        extraction_status: "draft".to_string(),
        source_transcript_id: Some(transcript_id.clone()),
        source_span_start: Some(0),
        source_span_end: Some(52),
        confidence: Some(0.92),
        created_at: now.clone(),
        metadata: None,
    }).unwrap();

    let risk_id = EntityId::new().to_string();
    extraction_store.insert_draft(&ExtractionRow {
        id: risk_id.clone(),
        entity_type: "Risk".to_string(),
        title: "Rate limiting at scale".to_string(),
        speech_act: Some("risk".to_string()),
        extraction_status: "draft".to_string(),
        source_transcript_id: Some(transcript_id.clone()),
        source_span_start: Some(53),
        source_span_end: Some(100),
        confidence: Some(0.85),
        created_at: now.clone(),
        metadata: None,
    }).unwrap();

    let task_id = EntityId::new().to_string();
    extraction_store.insert_draft(&ExtractionRow {
        id: task_id.clone(),
        entity_type: "Task".to_string(),
        title: "Auth service refactor".to_string(),
        speech_act: Some("commit".to_string()),
        extraction_status: "draft".to_string(),
        source_transcript_id: Some(transcript_id.clone()),
        source_span_start: Some(101),
        source_span_end: Some(150),
        confidence: Some(0.88),
        created_at: now.clone(),
        metadata: None,
    }).unwrap();

    // Verify all are pending
    let pending = extraction_store.list_pending().unwrap();
    assert_eq!(pending.len(), 3);
    // Ordered by confidence DESC
    assert_eq!(pending[0].title, "Going with option B for API design");

    // Promote the decision
    let promoted = extraction_store.promote(&decision_id).unwrap();
    assert_eq!(promoted.extraction_status, "promoted");

    // Move promoted to entities.db
    let entity_store = tw.entities();
    entity_store.insert(&EntityRow {
        id: promoted.id.clone(),
        entity_type: promoted.entity_type,
        title: promoted.title.clone(),
        created_at: promoted.created_at,
        updated_at: Utc::now().to_rfc3339(),
        status: None,
        task_state: None,
        extraction_status: Some("promoted".to_string()),
        source_transcript_id: promoted.source_transcript_id,
        source_span_start: promoted.source_span_start,
        source_span_end: promoted.source_span_end,
        confidence: promoted.confidence,
        content_path: None,
        metadata: None,
    }).unwrap();

    // Register in graph with EXTRACTED_FROM relation
    let dec_eid: EntityId = uuid::Uuid::parse_str(&decision_id).unwrap().into();
    let trans_eid: EntityId = uuid::Uuid::parse_str(&transcript_id).unwrap().into();
    tw.graph().register_node(&dec_eid, EntityType::Decision, &promoted.title).unwrap();
    tw.graph().add_edge(&dec_eid, &trans_eid, RelationType::ExtractedFrom).unwrap();

    // Discard the low-value extraction
    extraction_store.discard(&risk_id).unwrap();

    // Verify state
    let remaining = extraction_store.list_pending().unwrap();
    assert_eq!(remaining.len(), 1); // Only the task commit remains
    assert_eq!(remaining[0].title, "Auth service refactor");

    // Verify promoted decision is in entities.db
    let decisions = entity_store.list_by_type("Decision").unwrap();
    assert_eq!(decisions.len(), 1);

    // Verify graph relation
    let extracted = tw.graph().get_edges_by_type(&dec_eid, RelationType::ExtractedFrom).unwrap();
    assert_eq!(extracted.len(), 1);
}

// ===========================================================================
// Scenario 4: Reflection & cross-entity connections
//
// Create a weekly reflection that references programs and surfaces insights
// ===========================================================================

#[test]
fn scenario_reflection_workflow() {
    let tw = TestWorkspace::new();

    // Set up context
    let prog1_id = tw.insert_entity("Program", "Technical Education");
    let prog2_id = tw.insert_entity("Program", "PMO Establishment");

    let task1_id = tw.insert_entity("Task", "Prepare workshop materials");
    let task2_id = tw.insert_entity("Task", "Draft PMO charter");
    tw.relate(&task1_id, RelationType::BelongsTo, &prog1_id);
    tw.relate(&task2_id, RelationType::BelongsTo, &prog2_id);

    // Complete one task
    let store = tw.entities();
    let mut row = store.get(&task1_id).unwrap().unwrap();
    row.task_state = Some("doing".to_string());
    store.update(&row).unwrap();

    // Create weekly reflection
    let reflection_id = tw.insert_entity_with_content(
        "Reflection",
        "2025-W03 Weekly Reflection",
        "# Week 3 Reflection\n\n## Progress\n- Workshop materials 60% done\n- PMO charter still in draft\n\n## Key Takeaways\n- Need dedicated time blocks for deep work\n- Cross-team alignment taking longer than expected\n\n## Next Week\n- Finish workshop deck\n- Schedule PMO review meeting",
    );

    // Relate reflection to programs
    tw.relate(&reflection_id, RelationType::RelatesTo, &prog1_id);
    tw.relate(&reflection_id, RelationType::RelatesTo, &prog2_id);

    // Search for reflection content
    let results = tw.search().search("workshop").unwrap();
    assert!(results.len() >= 1);

    let alignment = tw.search().search("alignment").unwrap();
    assert_eq!(alignment.len(), 1);
    assert!(alignment[0].title.contains("Reflection"));

    // Verify reflection relates to both programs
    let ref_eid: EntityId = uuid::Uuid::parse_str(&reflection_id).unwrap().into();
    let relates = tw.graph().get_edges_by_type(&ref_eid, RelationType::RelatesTo).unwrap();
    assert_eq!(relates.len(), 2);

    // Query: what's connected to Technical Education?
    let prog1_eid: EntityId = uuid::Uuid::parse_str(&prog1_id).unwrap().into();
    let neighbors = tw.graph().get_neighbors(&prog1_eid).unwrap();
    assert_eq!(neighbors.len(), 2); // task1 + reflection
}

// ===========================================================================
// Scenario 5: Git sync lifecycle
//
// Init → create entities → sync → modify → sync again → prune
// ===========================================================================

#[test]
fn scenario_git_sync_lifecycle() {
    let tw = TestWorkspace::new();
    let engine = tw.sync_engine();

    // Initial sync — commits workspace structure
    let r1 = engine.sync().unwrap();
    assert!(r1.committed);
    assert!(!r1.pushed); // no remote
    assert_eq!(engine.commit_count().unwrap(), 1);

    // Create some entities
    tw.insert_entity_with_content("Program", "Test Program", "# Test\n\nContent here.");
    tw.insert_entity("Task", "Do something");

    // Sync again
    let r2 = engine.sync().unwrap();
    assert!(r2.committed);
    assert!(r2.files_changed > 0);
    assert_eq!(engine.commit_count().unwrap(), 2);

    // Modify content
    let store = tw.entities();
    let all = store.list_all().unwrap();
    let mut prog = all.iter().find(|r| r.entity_type == "Program").unwrap().clone();
    prog.title = "Updated Program".to_string();
    store.update(&prog).unwrap();

    // Sync modification
    let r3 = engine.sync().unwrap();
    assert!(r3.committed);
    assert_eq!(engine.commit_count().unwrap(), 3);

    // No-change sync
    let r4 = engine.sync().unwrap();
    assert!(!r4.committed);

    // Create more commits for pruning
    for i in 0..5 {
        fs::write(
            tw.ws.path.join(format!("content/notes/note-{}.md", i)),
            format!("# Note {}", i),
        ).unwrap();
        engine.sync().unwrap();
    }
    assert_eq!(engine.commit_count().unwrap(), 8);

    // Prune to keep 3
    let pruned = engine.prune_history(3).unwrap();
    assert_eq!(pruned, 5);
    assert_eq!(engine.commit_count().unwrap(), 3);
}

// ===========================================================================
// Scenario 6: Full graph traversal
//
// Build a realistic graph and query it with Cypher
// ===========================================================================

#[test]
fn scenario_graph_traversal() {
    let tw = TestWorkspace::new();

    // Build a realistic work graph
    let prog = tw.insert_entity("Program", "Monolith Breakup");
    let obj1 = tw.insert_entity("Objective", "Extract user service");
    let obj2 = tw.insert_entity("Objective", "Extract billing service");
    tw.relate(&obj1, RelationType::BelongsTo, &prog);
    tw.relate(&obj2, RelationType::BelongsTo, &prog);

    let task1 = tw.insert_entity("Task", "Design user service API");
    let task2 = tw.insert_entity("Task", "Implement user service");
    let task3 = tw.insert_entity("Task", "Design billing API");
    tw.relate(&task1, RelationType::BelongsTo, &prog);
    tw.relate(&task2, RelationType::BelongsTo, &prog);
    tw.relate(&task3, RelationType::BelongsTo, &prog);

    let blocker = tw.insert_entity("Blocker", "Shared database coupling");
    tw.relate(&task2, RelationType::BlockedBy, &blocker);

    let alice = tw.insert_entity("Person", "Alice");
    let meeting = tw.insert_entity("Meeting", "Architecture Review");
    tw.relate(&meeting, RelationType::Mentions, &alice);

    let decision = tw.insert_entity("Decision", "Use strangler fig pattern");
    tw.relate(&meeting, RelationType::HasDecision, &decision);

    let artifact = tw.insert_entity("Artifact", "User service RFC");
    tw.relate(&artifact, RelationType::Delivers, &obj1);

    // Query: what belongs to the program?
    let prog_eid: EntityId = uuid::Uuid::parse_str(&prog).unwrap().into();
    let belongs = tw.graph().get_incoming_by_type(&prog_eid, RelationType::BelongsTo).unwrap();
    assert_eq!(belongs.len(), 5); // 2 objectives + 3 tasks

    // Query: what's blocking task2?
    let task2_eid: EntityId = uuid::Uuid::parse_str(&task2).unwrap().into();
    let blockers = tw.graph().get_related_by_type(&task2_eid, RelationType::BlockedBy).unwrap();
    assert_eq!(blockers.len(), 1);
    assert_eq!(blockers[0].title, "Shared database coupling");

    // Query: what artifacts deliver against objectives?
    let obj1_eid: EntityId = uuid::Uuid::parse_str(&obj1).unwrap().into();
    let deliverables = tw.graph().get_incoming_by_type(&obj1_eid, RelationType::Delivers).unwrap();
    assert_eq!(deliverables.len(), 1);
    assert_eq!(deliverables[0].title, "User service RFC");

    // Query: what decisions came from the meeting?
    let meeting_eid: EntityId = uuid::Uuid::parse_str(&meeting).unwrap().into();
    let decisions = tw.graph().get_related_by_type(&meeting_eid, RelationType::HasDecision).unwrap();
    assert_eq!(decisions.len(), 1);

    // Raw Cypher: count all entities
    let result = tw.graph().raw_cypher("MATCH (n) RETURN count(n) AS total").unwrap();
    let total: i64 = result[0].get("total").unwrap_or(0);
    assert_eq!(total, 11); // prog + 2 obj + 3 task + blocker + alice + meeting + decision + artifact

    // Stats
    let stats = tw.graph().stats().unwrap();
    assert_eq!(stats.node_count, 11);
    assert!(stats.edge_count >= 9); // at least 9 edges we created
}

// ===========================================================================
// Scenario 7: Event log integrity
//
// Verify all mutations produce events
// ===========================================================================

#[test]
fn scenario_event_log_integrity() {
    let tw = TestWorkspace::new();
    let events = tw.events();

    // Log various events
    let id1 = tw.insert_entity("Program", "P1");
    events.log(&Event {
        timestamp: Utc::now(),
        event_type: EventType::Created,
        entity_id: id1.clone(),
        details: Some(serde_json::json!({"entity_type": "Program"})),
    }).unwrap();

    events.log(&Event {
        timestamp: Utc::now(),
        event_type: EventType::Updated,
        entity_id: id1.clone(),
        details: None,
    }).unwrap();

    let id2 = tw.insert_entity("Task", "T1");
    events.log(&Event {
        timestamp: Utc::now(),
        event_type: EventType::Created,
        entity_id: id2.clone(),
        details: None,
    }).unwrap();

    events.log(&Event {
        timestamp: Utc::now(),
        event_type: EventType::Transitioned,
        entity_id: id2.clone(),
        details: Some(serde_json::json!({"from": "todo", "to": "doing"})),
    }).unwrap();

    // Read all events
    let all = events.read_all().unwrap();
    assert_eq!(all.len(), 4);
    assert_eq!(all[0].event_type, EventType::Created);
    assert_eq!(all[1].event_type, EventType::Updated);
    assert_eq!(all[2].event_type, EventType::Created);
    assert_eq!(all[3].event_type, EventType::Transitioned);

    // Events are chronological
    for i in 1..all.len() {
        assert!(all[i].timestamp >= all[i-1].timestamp);
    }
}
