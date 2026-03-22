use std::path::Path;

use rusqlite::Connection;

use crate::error::StoreError;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

/// Run all pending migrations on the entities database.
pub fn run_migrations(db_path: &Path) -> Result<(), StoreError> {
    let mut conn = Connection::open(db_path)?;
    embedded::migrations::runner()
        .run(&mut conn)
        .map_err(|e| StoreError::Io(std::io::Error::other(e.to_string())))?;
    Ok(())
}

/// Run migrations on an in-memory connection (for tests).
pub fn run_migrations_in_memory(conn: &mut Connection) -> Result<(), StoreError> {
    embedded::migrations::runner()
        .run(conn)
        .map_err(|e| StoreError::Io(std::io::Error::other(e.to_string())))?;
    Ok(())
}
