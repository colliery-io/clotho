mod commands;
pub mod resolve;

use clap::Parser;

/// Clotho — Personal work and time management through reflection,
/// transcripts, and sense-making.
#[derive(Parser)]
#[command(name = "clotho", version, about)]
struct Cli {
    /// Output results as JSON instead of human-readable text.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Initialize a new .clotho/ directory.
    Init(commands::init::InitArgs),

    /// Create any entity type (program, task, person, etc.).
    Create(commands::create::CreateArgs),

    /// Read a single entity by ID.
    Get(commands::get::GetArgs),

    /// Update an entity's fields.
    Update(commands::update::UpdateArgs),

    /// Delete an entity from all backends.
    Delete(commands::delete::DeleteArgs),

    /// Capture a file as content (note, meeting, transcript, artifact).
    Capture(commands::capture::CaptureArgs),

    /// List entities with optional filters.
    List(commands::list::ListArgs),

    /// Search entities by keyword (FTS5).
    Search(commands::search::SearchArgs),

    /// Run a raw Cypher query against the relation graph.
    Query(commands::query::QueryArgs),

    /// Create a new reflection entry.
    Reflect(commands::reflect::ReflectArgs),

    /// Show workspace status dashboard.
    Status(commands::status::StatusArgs),

    /// Sync workspace to git (commit + push).
    Sync(commands::sync::SyncArgs),

    /// Create a typed relation between two entities.
    Relate(commands::relate::RelateArgs),

    /// Remove a typed relation between two entities.
    Unrelate(commands::relate::UnrelateArgs),

    /// Show all relations for an entity.
    Relations(commands::relate::RelationsArgs),

    /// Get ontology for a program/responsibility.
    OntologyGet(commands::ontology::OntologyGetArgs),

    /// Update ontology for a program/responsibility.
    OntologySet(commands::ontology::OntologySetArgs),

    /// Search across all ontologies for a term.
    OntologySearch(commands::ontology::OntologySearchArgs),

    /// Check processing history for an entity.
    ProcessedCheck(commands::processed::ProcessedCheckArgs),

    /// Mark an entity as processed.
    ProcessedMark(commands::processed::ProcessedMarkArgs),

    /// Launch the interactive TUI.
    Tui(commands::tui::TuiArgs),
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        // No subcommand — launch TUI with defaults
        None => commands::tui::run(commands::tui::TuiArgs {
            workspace: None,
            claude_args: Vec::new(),
        }),
        Some(Commands::Init(args)) => commands::init::run(args, cli.json),
        Some(Commands::Create(args)) => commands::create::run(args, cli.json),
        Some(Commands::Get(args)) => commands::get::run(args, cli.json),
        Some(Commands::Update(args)) => commands::update::run(args, cli.json),
        Some(Commands::Delete(args)) => commands::delete::run(args, cli.json),
        Some(Commands::Capture(args)) => commands::capture::run(args, cli.json),
        Some(Commands::List(args)) => commands::list::run(args, cli.json),
        Some(Commands::Search(args)) => commands::search::run(args, cli.json),
        Some(Commands::Query(args)) => commands::query::run(args, cli.json),
        Some(Commands::Reflect(args)) => commands::reflect::run(args, cli.json),
        Some(Commands::Status(args)) => commands::status::run(args, cli.json),
        Some(Commands::Sync(args)) => commands::sync::run(args, cli.json),
        Some(Commands::Relate(args)) => commands::relate::run_relate(args, cli.json),
        Some(Commands::Unrelate(args)) => commands::relate::run_unrelate(args, cli.json),
        Some(Commands::Relations(args)) => commands::relate::run_relations(args, cli.json),
        Some(Commands::OntologyGet(args)) => commands::ontology::run_get(args, cli.json),
        Some(Commands::OntologySet(args)) => commands::ontology::run_set(args, cli.json),
        Some(Commands::OntologySearch(args)) => commands::ontology::run_search(args, cli.json),
        Some(Commands::ProcessedCheck(args)) => commands::processed::run_check(args, cli.json),
        Some(Commands::ProcessedMark(args)) => commands::processed::run_mark(args, cli.json),
        Some(Commands::Tui(args)) => commands::tui::run(args),
    };

    if let Err(e) = result {
        if cli.json {
            let err = serde_json::json!({"error": e.to_string()});
            eprintln!("{}", serde_json::to_string_pretty(&err).unwrap());
        } else {
            eprintln!("Error: {}", e);
        }
        std::process::exit(1);
    }
}
