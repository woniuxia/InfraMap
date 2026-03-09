pub const SQL: &str = r#"
        CREATE TABLE IF NOT EXISTS taxonomy_term_stats (
            term_id TEXT PRIMARY KEY,
            resource_type TEXT NOT NULL,
            field_key TEXT NOT NULL,
            usage_count INTEGER NOT NULL DEFAULT 0,
            last_used_at TEXT,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_taxonomy_term_stats_scope_recent
        ON taxonomy_term_stats(resource_type, field_key, last_used_at DESC);

        CREATE INDEX IF NOT EXISTS idx_taxonomy_term_stats_field_recent
        ON taxonomy_term_stats(field_key, last_used_at DESC);

        CREATE INDEX IF NOT EXISTS idx_taxonomy_term_stats_scope_count
        ON taxonomy_term_stats(resource_type, field_key, usage_count DESC);
    
"#;
