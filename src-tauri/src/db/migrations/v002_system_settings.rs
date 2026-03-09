pub const SQL: &str = r#"
        CREATE TABLE IF NOT EXISTS system_settings (
            id TEXT PRIMARY KEY,
            auto_backup_enabled INTEGER NOT NULL DEFAULT 0,
            backup_interval_hours INTEGER NOT NULL DEFAULT 24,
            max_backups INTEGER NOT NULL DEFAULT 10,
            last_backup_time TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        INSERT OR IGNORE INTO system_settings (id, auto_backup_enabled, backup_interval_hours, max_backups, created_at, updated_at)
        VALUES ('default', 0, 24, 10, datetime('now'), datetime('now'));
    
"#;
