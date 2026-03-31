use crate::formatting::text_result;
use crate::resolve;
use crate::workspace_resolver;
use clotho_store::data::entities::EntityStore;
use clotho_store::data::ontology::{
    OntologyStore, CATEGORY_IGNORE, CATEGORY_KEYWORD, CATEGORY_PERSON, CATEGORY_SIGNAL_SOCIAL,
    CATEGORY_SIGNAL_TECHNICAL,
};
use clotho_store::workspace::Workspace;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_get_ontology",
    description = "Get the extraction ontology for a program or responsibility. Accepts full UUID or prefix.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetOntologyTool {
    /// Entity ID (full UUID or prefix)
    pub entity_id: String,
}

impl GetOntologyTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let entity_store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let ontology_store = OntologyStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let row = match resolve::resolve_for_read(&entity_store, &self.entity_id) {
            Ok(row) => row,
            Err(result) => return Ok(result),
        };

        let ontology = ontology_store
            .get(&row.id)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let mut output = format!("## Ontology: {} ({})\n\n", row.title, row.entity_type);

        if !ontology.keywords.is_empty() {
            output.push_str(&format!(
                "**Keywords:** {}\n\n",
                ontology.keywords.join(", ")
            ));
        }
        if !ontology.signal_technical.is_empty() {
            output.push_str(&format!(
                "**Technical signals:** {}\n\n",
                ontology.signal_technical.join(", ")
            ));
        }
        if !ontology.signal_social.is_empty() {
            output.push_str(&format!(
                "**Social signals:** {}\n\n",
                ontology.signal_social.join(", ")
            ));
        }
        if !ontology.people.is_empty() {
            output.push_str(&format!(
                "**Involved people:** {}\n\n",
                ontology.people.join(", ")
            ));
        }
        if !ontology.ignore.is_empty() {
            output.push_str(&format!(
                "**Ignore (excluded from extraction):** {}\n\n",
                ontology.ignore.join(", ")
            ));
        }

        if ontology.keywords.is_empty()
            && ontology.signal_technical.is_empty()
            && ontology.signal_social.is_empty()
            && ontology.people.is_empty()
            && ontology.ignore.is_empty()
        {
            output.push_str("No ontology configured yet.");
        }

        Ok(text_result(output))
    }
}

#[mcp_tool(
    name = "clotho_update_ontology",
    description = "Update the extraction ontology for a program or responsibility. Accepts full UUID or prefix.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateOntologyTool {
    /// Entity ID (full UUID or prefix)
    pub entity_id: String,
    /// Keywords to add (comma-separated)
    pub add_keywords: Option<String>,
    /// Keywords to remove (comma-separated)
    pub remove_keywords: Option<String>,
    /// Technical signal types to add (comma-separated)
    pub add_technical_signals: Option<String>,
    /// Social signal types to add (comma-separated)
    pub add_social_signals: Option<String>,
    /// People to add (comma-separated)
    pub add_people: Option<String>,
    /// People to remove (comma-separated)
    pub remove_people: Option<String>,
    /// Topics to ignore/exclude from extraction (comma-separated)
    pub add_ignore: Option<String>,
    /// Topics to stop ignoring (comma-separated)
    pub remove_ignore: Option<String>,
    /// Who is making this change: "user" or "agent"
    pub added_by: Option<String>,
}

impl UpdateOntologyTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let entity_store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let ontology_store = OntologyStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let row = match resolve::resolve_for_write(&entity_store, &self.entity_id) {
            Ok(row) => row,
            Err(result) => return Ok(result),
        };
        let resolved_id = &row.id;

        let added_by = self.added_by.as_deref().unwrap_or("user");

        // Additions
        if let Some(ref kw) = self.add_keywords {
            let vals: Vec<&str> = kw.split(',').collect();
            ontology_store
                .add(resolved_id, CATEGORY_KEYWORD, &vals, Some(added_by))
                .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        }
        if let Some(ref ts) = self.add_technical_signals {
            let vals: Vec<&str> = ts.split(',').collect();
            ontology_store
                .add(
                    resolved_id,
                    CATEGORY_SIGNAL_TECHNICAL,
                    &vals,
                    Some(added_by),
                )
                .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        }
        if let Some(ref ss) = self.add_social_signals {
            let vals: Vec<&str> = ss.split(',').collect();
            ontology_store
                .add(resolved_id, CATEGORY_SIGNAL_SOCIAL, &vals, Some(added_by))
                .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        }
        if let Some(ref pp) = self.add_people {
            let vals: Vec<&str> = pp.split(',').collect();
            ontology_store
                .add(resolved_id, CATEGORY_PERSON, &vals, Some(added_by))
                .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        }

        if let Some(ref ig) = self.add_ignore {
            let vals: Vec<&str> = ig.split(',').collect();
            ontology_store
                .add(resolved_id, CATEGORY_IGNORE, &vals, Some(added_by))
                .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        }

        // Removals
        if let Some(ref ig) = self.remove_ignore {
            let vals: Vec<&str> = ig.split(',').collect();
            ontology_store
                .remove(resolved_id, CATEGORY_IGNORE, &vals)
                .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        }
        if let Some(ref kw) = self.remove_keywords {
            let vals: Vec<&str> = kw.split(',').collect();
            ontology_store
                .remove(resolved_id, CATEGORY_KEYWORD, &vals)
                .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        }
        if let Some(ref pp) = self.remove_people {
            let vals: Vec<&str> = pp.split(',').collect();
            ontology_store
                .remove(resolved_id, CATEGORY_PERSON, &vals)
                .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        }

        Ok(text_result(format!(
            "## Ontology Updated\n\nUpdated ontology for entity `{}`",
            &resolved_id[..8]
        )))
    }
}

#[mcp_tool(
    name = "clotho_search_ontology",
    description = "Search across all ontologies to find which programs/responsibilities care about a topic.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchOntologyTool {
    /// Search term
    pub query: String,
}

impl SearchOntologyTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let ontology_store = OntologyStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let results = ontology_store
            .search(&self.query)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        if results.is_empty() {
            return Ok(text_result(format!(
                "No ontology entries matching '{}'.",
                self.query
            )));
        }

        let mut output = format!(
            "## Ontology Search: '{}'\n\n| Entity | Category | Value |\n|---|---|---|\n",
            self.query
        );
        for entry in &results {
            output.push_str(&format!(
                "| `{}...` | {} | {} |\n",
                &entry.entity_id[..8],
                entry.category,
                entry.value
            ));
        }
        output.push_str(&format!("\n{} matches", results.len()));

        Ok(text_result(output))
    }
}
