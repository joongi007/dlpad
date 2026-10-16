-- ToolManager's dedicated activation.sqlite; no library/download schema version.
CREATE TABLE IF NOT EXISTS install_staging (
    name TEXT PRIMARY KEY NOT NULL,
    evidence_json TEXT NOT NULL CHECK(length(CAST(evidence_json AS BLOB)) <= 16384)
);
CREATE TABLE IF NOT EXISTS install_schema_migrations(version INTEGER PRIMARY KEY);
INSERT OR IGNORE INTO install_schema_migrations(version) VALUES(1);
