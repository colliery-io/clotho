-- Per-program extraction ontology
CREATE TABLE IF NOT EXISTS ontology (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_id TEXT NOT NULL,
    category TEXT NOT NULL,
    value TEXT NOT NULL,
    added_at TEXT NOT NULL,
    added_by TEXT,
    UNIQUE(entity_id, category, value)
);

CREATE INDEX IF NOT EXISTS idx_ontology_entity ON ontology(entity_id);
CREATE INDEX IF NOT EXISTS idx_ontology_category ON ontology(category);
CREATE INDEX IF NOT EXISTS idx_ontology_value ON ontology(value);
