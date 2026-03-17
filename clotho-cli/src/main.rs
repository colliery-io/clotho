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
    /// Initialize a new .workspace/ directory.
    Init(commands::init::InitArgs),

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
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init(args) => commands::init::run(args, cli.json),
        Commands::Ingest(args) => commands::ingest::run(args, cli.json),
        Commands::List(args) => commands::list::run(args, cli.json),
        Commands::Search(args) => commands::search::run(args, cli.json),
        Commands::Query(args) => commands::query::run(args, cli.json),
        Commands::Reflect(args) => commands::reflect::run(args, cli.json),
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
