/// Migration v026: Add focus_target column to topology_node_positions
/// to support independent position caching for different focus objects.
pub const SQL: &str = r#"
-- Step 1: Create new table with focus_target column
CREATE TABLE topology_node_positions_new (
    node_id      TEXT NOT NULL,
    layout_type  TEXT NOT NULL,
    focus_target TEXT,
    x            REAL NOT NULL,
    y            REAL NOT NULL,
    updated_at   TEXT NOT NULL,
    PRIMARY KEY (node_id, layout_type, focus_target)
);

-- Step 2: Migrate existing data (force layout keeps NULL focus_target)
INSERT INTO topology_node_positions_new (node_id, layout_type, focus_target, x, y, updated_at)
SELECT node_id, layout_type, NULL, x, y, updated_at
FROM topology_node_positions
WHERE layout_type = 'force';

-- Step 3: Drop old table
DROP TABLE topology_node_positions;

-- Step 4: Rename new table
ALTER TABLE topology_node_positions_new RENAME TO topology_node_positions;

-- Step 5: Create index for query performance
CREATE INDEX idx_topology_positions_lookup
ON topology_node_positions(layout_type, focus_target);
"#;
