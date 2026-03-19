-- Processing log for idempotent transcript extraction
CREATE TABLE IF NOT EXISTS processing_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_id TEXT NOT NULL,
    process_name TEXT NOT NULL,
    ontology_ids TEXT,
    processed_at TEXT NOT NULL,
    processed_by TEXT,
    output_entity_ids TEXT,
    notes TEXT,
    UNIQUE(entity_id, process_name, ontology_ids)
);

CREATE INDEX IF NOT EXISTS idx_processing_entity ON processing_log(entity_id);
CREATE INDEX IF NOT EXISTS idx_processing_name ON processing_log(process_name);
