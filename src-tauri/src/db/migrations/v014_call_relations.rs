pub const SQL: &str = r#"
        CREATE TABLE IF NOT EXISTS call_relations (
            id TEXT PRIMARY KEY,
            pair_key TEXT NOT NULL,
            owner_id TEXT NOT NULL,
            owner_type TEXT NOT NULL,
            peer_id TEXT NOT NULL,
            peer_type TEXT NOT NULL,
            direction TEXT NOT NULL,
            relation_type TEXT NOT NULL,
            description TEXT,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS uk_call_relations_owner_peer_direction_type
        ON call_relations(owner_id, owner_type, peer_id, peer_type, direction, relation_type)
        WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_call_relations_owner
        ON call_relations(owner_id, owner_type) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_call_relations_peer
        ON call_relations(peer_id, peer_type) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_call_relations_pair
        ON call_relations(pair_key) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_call_relations_relation_type
        ON call_relations(relation_type) WHERE is_deleted = 0;

        DROP INDEX IF EXISTS uk_dependencies_source_target;
        DROP INDEX IF EXISTS idx_dependencies_source;
        DROP INDEX IF EXISTS idx_dependencies_target;
        DROP INDEX IF EXISTS idx_dependencies_relation_type;
        DROP TABLE IF EXISTS dependencies;
    
"#;
