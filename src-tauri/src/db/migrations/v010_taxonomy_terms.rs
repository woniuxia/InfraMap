pub const SQL: &str = r#"
        CREATE TABLE IF NOT EXISTS taxonomy_terms (
            id TEXT PRIMARY KEY,
            resource_type TEXT NOT NULL,
            field_key TEXT NOT NULL,
            value TEXT NOT NULL,
            display_name TEXT NOT NULL,
            normalized_value TEXT NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS taxonomy_bindings (
            id TEXT PRIMARY KEY,
            term_id TEXT NOT NULL,
            resource_type TEXT NOT NULL,
            resource_id TEXT NOT NULL,
            field_key TEXT NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS uk_taxonomy_terms_scope_normalized
        ON taxonomy_terms(resource_type, field_key, normalized_value) WHERE is_deleted = 0;

        CREATE INDEX IF NOT EXISTS idx_taxonomy_terms_scope
        ON taxonomy_terms(resource_type, field_key) WHERE is_deleted = 0;

        CREATE INDEX IF NOT EXISTS idx_taxonomy_terms_display_name
        ON taxonomy_terms(display_name) WHERE is_deleted = 0;

        CREATE UNIQUE INDEX IF NOT EXISTS uk_taxonomy_bindings_term_resource_field
        ON taxonomy_bindings(term_id, resource_type, resource_id, field_key) WHERE is_deleted = 0;

        CREATE INDEX IF NOT EXISTS idx_taxonomy_bindings_resource
        ON taxonomy_bindings(resource_type, resource_id, field_key) WHERE is_deleted = 0;

        CREATE INDEX IF NOT EXISTS idx_taxonomy_bindings_term
        ON taxonomy_bindings(term_id) WHERE is_deleted = 0;
    
"#;
