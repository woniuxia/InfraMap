pub const SQL: &str = r#"
        ALTER TABLE hosts ADD COLUMN env TEXT NOT NULL DEFAULT 'prod';
        CREATE INDEX IF NOT EXISTS idx_hosts_env ON hosts(env) WHERE is_deleted = 0;
    
"#;
