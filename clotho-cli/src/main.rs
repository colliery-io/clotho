mod commands;

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
    command: Commands,
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

    /// Ingest a file as content (note, meeting, transcript, artifact).
    Ingest(commands::ingest::IngestArgs),

    /// List entities with optional filters.
    List(commands::list::ListArgs),

    /// Search entities by keyword (FTS5).
    Search(commands::search::SearchArgs),

    /// Run a raw Cypher query against the relation graph.
    Query(commands::query::QueryArgs),

    /// Create a new reflection entry.
    Reflect(commands::reflect::ReflectArgs),

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
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init(args) => commands::init::run(args, cli.json),
        Commands::Create(args) => commands::create::run(args, cli.json),
        Commands::Get(args) => commands::get::run(args, cli.json),
        Commands::Update(args) => commands::update::run(args, cli.json),
        Commands::Delete(args) => commands::delete::run(args, cli.json),
        Commands::Ingest(args) => commands::ingest::run(args, cli.json),
        Commands::List(args) => commands::list::run(args, cli.json),
        Commands::Search(args) => commands::search::run(args, cli.json),
        Commands::Query(args) => commands::query::run(args, cli.json),
        Commands::Reflect(args) => commands::reflect::run(args, cli.json),
        Commands::Sync(args) => commands::sync::run(args, cli.json),
        Commands::Relate(args) => commands::relate::run_relate(args, cli.json),
        Commands::Unrelate(args) => commands::relate::run_unrelate(args, cli.json),
        Commands::Relations(args) => commands::relate::run_relations(args, cli.json),
        Commands::OntologyGet(args) => commands::ontology::run_get(args, cli.json),
        Commands::OntologySet(args) => commands::ontology::run_set(args, cli.json),
        Commands::OntologySearch(args) => commands::ontology::run_search(args, cli.json),
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
