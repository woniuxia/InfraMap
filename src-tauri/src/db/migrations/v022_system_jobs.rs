pub const SQL: &str = r#"
    CREATE TABLE IF NOT EXISTS system_jobs (
        id TEXT PRIMARY KEY,
        job_type TEXT NOT NULL,
        title TEXT NOT NULL,
        status TEXT NOT NULL,
        summary TEXT,
        progress_percent REAL NOT NULL DEFAULT 100,
        retryable INTEGER NOT NULL DEFAULT 0,
        cancellable INTEGER NOT NULL DEFAULT 0,
        payload_json TEXT NOT NULL DEFAULT '{}',
        result_json TEXT,
        error_message TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        finished_at TEXT
    );

    CREATE INDEX IF NOT EXISTS idx_system_jobs_created_at
    ON system_jobs(created_at DESC);
    CREATE INDEX IF NOT EXISTS idx_system_jobs_status
    ON system_jobs(status);
    CREATE INDEX IF NOT EXISTS idx_system_jobs_type
    ON system_jobs(job_type);
"#;
