pub const SQL: &str = r#"
        CREATE TABLE IF NOT EXISTS application_owners (
            id TEXT PRIMARY KEY,
            application_id TEXT NOT NULL,
            owner_name TEXT NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS uk_application_owners_app_owner
        ON application_owners(application_id, owner_name) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_application_owners_application
        ON application_owners(application_id) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_application_owners_owner_name
        ON application_owners(owner_name) WHERE is_deleted = 0;
    
"#;
