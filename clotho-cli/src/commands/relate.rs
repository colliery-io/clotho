use clap::Args;

use clotho_core::domain::traits::RelationType;
use clotho_core::domain::types::EntityId;
use clotho_core::graph::GraphStore;
use clotho_store::workspace::Workspace;

#[derive(Args)]
pub struct RelateArgs {
    /// Source entity ID.
    pub source_id: String,

    /// Relation type (belongs_to, relates_to, delivers, spawned_from, extracted_from,
    /// has_decision, has_risk, blocked_by, mentions, has_cadence, has_deadline, has_schedule).
    pub relation_type: String,

    /// Target entity ID.
    pub target_id: String,
}

#[derive(Args)]
pub struct UnrelateArgs {
    /// Source entity ID.
    pub source_id: String,

    /// Relation type.
    pub relation_type: String,

    /// Target entity ID.
    pub target_id: String,
}

#[derive(Args)]
pub struct RelationsArgs {
    /// Entity ID to show relations for.
    pub id: String,
}

pub fn run_relate(args: RelateArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let graph = GraphStore::open(&ws.graph_path().join("relations.db"))
        .map_err(|e| format!("graph error: {}", e))?;

    let source = parse_id(&args.source_id)?;
    let target = parse_id(&args.target_id)?;
    let rel_type = parse_relation_type(&args.relation_type)?;

    // Verify both nodes exist
    if !graph.has_node(&source).map_err(|e| format!("{}", e))? {
        return Err(format!("Source entity not found in graph: {}", args.source_id).into());
    }
    if !graph.has_node(&target).map_err(|e| format!("{}", e))? {
        return Err(format!("Target entity not found in graph: {}", args.target_id).into());
    }

    graph
        .add_edge(&source, &target, rel_type)
        .map_err(|e| format!("graph error: {}", e))?;

    if json {
        let out = serde_json::json!({
            "status": "ok",
            "source": args.source_id,
            "relation": args.relation_type,
            "target": args.target_id,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!(
            "Created relation: {} -[{}]-> {}",
            &args.source_id[..8],
            args.relation_type.to_uppercase(),
            &args.target_id[..8]
        );
    }

    Ok(())
}

pub fn run_unrelate(args: UnrelateArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let graph = GraphStore::open(&ws.graph_path().join("relations.db"))
        .map_err(|e| format!("graph error: {}", e))?;

    let source = parse_id(&args.source_id)?;
    let target = parse_id(&args.target_id)?;
    let rel_type = parse_relation_type(&args.relation_type)?;

    graph
        .remove_edge(&source, &target, rel_type)
        .map_err(|e| format!("graph error: {}", e))?;

    if json {
        let out = serde_json::json!({
            "status": "ok",
            "removed": true,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!(
            "Removed relation: {} -[{}]-> {}",
            &args.source_id[..8],
            args.relation_type.to_uppercase(),
            &args.target_id[..8]
        );
    }

    Ok(())
}

pub fn run_relations(args: RelationsArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let graph = GraphStore::open(&ws.graph_path().join("relations.db"))
        .map_err(|e| format!("graph error: {}", e))?;

    let id = parse_id(&args.id)?;

    let outgoing = graph
        .get_edges_from(&id)
        .map_err(|e| format!("graph error: {}", e))?;
    let incoming = graph
        .get_edges_to(&id)
        .map_err(|e| format!("graph error: {}", e))?;

    if json {
        let out = serde_json::json!({
            "entity_id": args.id,
            "outgoing": outgoing.iter().map(|e| serde_json::json!({
                "relation": format!("{:?}", e.relation_type),
                "target": e.target_id.to_string(),
            })).collect::<Vec<_>>(),
            "incoming": incoming.iter().map(|e| serde_json::json!({
                "relation": format!("{:?}", e.relation_type),
                "source": e.source_id.to_string(),
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        if outgoing.is_empty() && incoming.is_empty() {
            println!("No relations for entity {}", args.id);
            return Ok(());
        }

        if !outgoing.is_empty() {
            println!("Outgoing relations:");
            for e in &outgoing {
                println!("  -[{:?}]-> {}", e.relation_type, e.target_id);
            }
        }

        if !incoming.is_empty() {
            println!("Incoming relations:");
            for e in &incoming {
                println!("  {} -[{:?}]->", e.source_id, e.relation_type);
            }
        }

        println!(
            "\n{} outgoing, {} incoming",
            outgoing.len(),
            incoming.len()
        );
    }

    Ok(())
}

pub fn parse_relation_type(s: &str) -> Result<RelationType, Box<dyn std::error::Error>> {
    match s.to_lowercase().as_str() {
        "belongs_to" => Ok(RelationType::BelongsTo),
        "relates_to" => Ok(RelationType::RelatesTo),
        "delivers" => Ok(RelationType::Delivers),
        "spawned_from" => Ok(RelationType::SpawnedFrom),
        "extracted_from" => Ok(RelationType::ExtractedFrom),
        "has_decision" => Ok(RelationType::HasDecision),
        "has_risk" => Ok(RelationType::HasRisk),
        "blocked_by" => Ok(RelationType::BlockedBy),
        "mentions" => Ok(RelationType::Mentions),
        "has_cadence" => Ok(RelationType::HasCadence),
        "has_deadline" => Ok(RelationType::HasDeadline),
        "has_schedule" => Ok(RelationType::HasSchedule),
        _ => Err(format!(
            "Unknown relation type '{}'. Valid: belongs_to, relates_to, delivers, spawned_from, extracted_from, has_decision, has_risk, blocked_by, mentions, has_cadence, has_deadline, has_schedule",
            s
        ).into()),
    }
}

fn parse_id(s: &str) -> Result<EntityId, Box<dyn std::error::Error>> {
    uuid::Uuid::parse_str(s)
        .map(EntityId::from)
        .map_err(|e| format!("invalid entity ID '{}': {}", s, e).into())
}
