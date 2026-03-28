-- Surfaces: agent-pushed text blobs for the TUI
CREATE TABLE IF NOT EXISTS surfaces (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    surface_type TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_surfaces_status ON surfaces(status);
CREATE INDEX IF NOT EXISTS idx_surfaces_type ON surfaces(surface_type);
