use std::fs;

use tempfile::tempdir;

use clotho_store::data::entities::EntityStore;
use clotho_store::workspace::Workspace;

use clotho_mcp_server::tools::ClothoTools;

// ===========================================================================
// Tool listing
// ===========================================================================

#[test]
fn list_tools_returns_all_fifteen() {
    let tools = ClothoTools::tools();
    assert_eq!(tools.len(), 15);

    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"clotho_search"));
    assert!(names.contains(&"clotho_query"));
    assert!(names.contains(&"clotho_read_entity"));
    assert!(names.contains(&"clotho_list_entities"));
    assert!(names.contains(&"clotho_init"));
    assert!(names.contains(&"clotho_ingest"));
    assert!(names.contains(&"clotho_create_note"));
    assert!(names.contains(&"clotho_create_reflection"));
}

#[test]
fn all_tools_have_descriptions() {
    let tools = ClothoTools::tools();
    for tool in &tools {
        assert!(
            tool.description.is_some(),
            "Tool {} has no description",
            tool.name
        );
    }
}

// ===========================================================================
// Tool execution (direct call_tool invocation)
// ===========================================================================

#[tokio::test]
async fn test_init_tool() {
    use clotho_mcp_server::tools::InitTool;

    let tmp = tempdir().unwrap();
    let tool = InitTool {
        path: tmp.path().display().to_string(),
    };
    let result = tool.call_tool().await.unwrap();
    assert!(result.is_error.is_none());
    assert!(tmp.path().join(".clotho").exists());
}

#[tokio::test]
async fn test_ingest_tool() {
    use clotho_mcp_server::tools::IngestTool;

    let tmp = tempdir().unwrap();
    Workspace::init(tmp.path()).unwrap();

    let file_path = tmp.path().join("test-note.md");
    fs::write(&file_path, "# Test Note\n\nSome interesting content about architecture.").unwrap();

    let tool = IngestTool {
        workspace_path: tmp.path().display().to_string(),
        file_path: file_path.display().to_string(),
        entity_type: Some("note".to_string()),
        title: Some("Test Note".to_string()),
    };
    let result = tool.call_tool().await.unwrap();
    assert!(result.is_error.is_none());

    let ws = Workspace::open(tmp.path()).unwrap();
    let store = EntityStore::open(&ws.data_path().join("entities.db")).unwrap();
    let all = store.list_all().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].title, "Test Note");
}

#[tokio::test]
async fn test_create_note_tool() {
    use clotho_mcp_server::tools::CreateNoteTool;

    let tmp = tempdir().unwrap();
    Workspace::init(tmp.path()).unwrap();

    let tool = CreateNoteTool {
        workspace_path: tmp.path().display().to_string(),
        title: "Architecture Thoughts".to_string(),
        content: "# Architecture\n\nThinking about microservices vs monolith.".to_string(),
    };
    let result = tool.call_tool().await.unwrap();
    assert!(result.is_error.is_none());

    let ws = Workspace::open(tmp.path()).unwrap();
    let store = EntityStore::open(&ws.data_path().join("entities.db")).unwrap();
    let notes = store.list_by_type("Note").unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "Architecture Thoughts");
}

#[tokio::test]
async fn test_create_reflection_tool() {
    use clotho_mcp_server::tools::CreateReflectionTool;

    let tmp = tempdir().unwrap();
    Workspace::init(tmp.path()).unwrap();

    let tool = CreateReflectionTool {
        workspace_path: tmp.path().display().to_string(),
        period: "weekly".to_string(),
        title: None,
        program_id: None,
    };
    let result = tool.call_tool().await.unwrap();
    assert!(result.is_error.is_none());

    let ws = Workspace::open(tmp.path()).unwrap();
    let store = EntityStore::open(&ws.data_path().join("entities.db")).unwrap();
    let reflections = store.list_by_type("Reflection").unwrap();
    assert_eq!(reflections.len(), 1);
    assert!(reflections[0].title.contains("weekly"));
}

#[tokio::test]
async fn test_search_tool_finds_content() {
    use clotho_mcp_server::tools::{CreateNoteTool, SearchTool};

    let tmp = tempdir().unwrap();
    Workspace::init(tmp.path()).unwrap();

    let create = CreateNoteTool {
        workspace_path: tmp.path().display().to_string(),
        title: "Deployment Strategy".to_string(),
        content: "Blue-green deployment with rolling updates and canary releases.".to_string(),
    };
    create.call_tool().await.unwrap();

    let search = SearchTool {
        workspace_path: tmp.path().display().to_string(),
        query: "canary".to_string(),
        limit: None,
    };
    let result = search.call_tool().await.unwrap();
    assert!(result.is_error.is_none());

    let text = format!("{:?}", result.content);
    assert!(text.contains("Deployment Strategy"));
}

#[tokio::test]
async fn test_list_entities_tool() {
    use clotho_mcp_server::tools::{CreateNoteTool, ListEntitiesTool};

    let tmp = tempdir().unwrap();
    Workspace::init(tmp.path()).unwrap();

    for title in &["Note A", "Note B"] {
        let tool = CreateNoteTool {
            workspace_path: tmp.path().display().to_string(),
            title: title.to_string(),
            content: "Content".to_string(),
        };
        tool.call_tool().await.unwrap();
    }

    let list = ListEntitiesTool {
        workspace_path: tmp.path().display().to_string(),
        entity_type: Some("Note".to_string()),
        status: None,
        state: None,
    };
    let result = list.call_tool().await.unwrap();
    assert!(result.is_error.is_none());

    let text = format!("{:?}", result.content);
    assert!(text.contains("Note A"));
    assert!(text.contains("Note B"));
}
