/// Migration v025: Create topology_node_positions table for persisting node layout positions.
/// This is a pure cache table — no id, no soft-delete, no audit logging.
pub const SQL: &str = r#"
CREATE TABLE IF NOT EXISTS topology_node_positions (
    node_id     TEXT NOT NULL,
    layout_type TEXT NOT NULL,
    x           REAL NOT NULL,
    y           REAL NOT NULL,
    updated_at  TEXT NOT NULL,
    PRIMARY KEY (node_id, layout_type)
);
"#;
