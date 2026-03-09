pub const SQL: &str = r#"
        CREATE TABLE IF NOT EXISTS business_applications (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            code TEXT,
            owner TEXT,
            description TEXT,
            env TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS uk_business_applications_name_env
        ON business_applications(name, env) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_business_applications_status
        ON business_applications(status) WHERE is_deleted = 0;

        ALTER TABLE applications ADD COLUMN business_application_id TEXT;
        CREATE INDEX IF NOT EXISTS idx_applications_business_application_id
        ON applications(business_application_id) WHERE is_deleted = 0;
    
"#;
