-- Extractions table for draft entities pending review
CREATE TABLE IF NOT EXISTS extractions (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    title TEXT NOT NULL,
    speech_act TEXT,
    extraction_status TEXT NOT NULL DEFAULT 'draft',
    source_transcript_id TEXT,
    source_span_start INTEGER,
    source_span_end INTEGER,
    confidence REAL,
    created_at TEXT NOT NULL,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_extractions_status ON extractions(extraction_status);
CREATE INDEX IF NOT EXISTS idx_extractions_confidence ON extractions(confidence);
CREATE INDEX IF NOT EXISTS idx_extractions_transcript ON extractions(source_transcript_id);
