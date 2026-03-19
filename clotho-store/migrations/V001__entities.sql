-- Initial entities table
CREATE TABLE IF NOT EXISTS entities (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    title TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    status TEXT,
    task_state TEXT,
    extraction_status TEXT,
    source_transcript_id TEXT,
    source_span_start INTEGER,
    source_span_end INTEGER,
    confidence REAL,
    content_path TEXT,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_entities_status ON entities(status);
CREATE INDEX IF NOT EXISTS idx_entities_task_state ON entities(task_state);
CREATE INDEX IF NOT EXISTS idx_entities_extraction_status ON entities(extraction_status);
