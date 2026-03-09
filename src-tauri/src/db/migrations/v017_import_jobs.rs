pub const SQL: &str = r#"
        CREATE TABLE IF NOT EXISTS import_jobs (
            id TEXT PRIMARY KEY,
            status TEXT NOT NULL,
            strategy TEXT NOT NULL,
            total_rows INTEGER NOT NULL DEFAULT 0,
            created_count INTEGER NOT NULL DEFAULT 0,
            updated_count INTEGER NOT NULL DEFAULT 0,
            skipped_count INTEGER NOT NULL DEFAULT 0,
            failed_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_import_jobs_created_at
        ON import_jobs(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_import_jobs_status
        ON import_jobs(status);

        CREATE TABLE IF NOT EXISTS import_job_rows (
            id TEXT PRIMARY KEY,
            job_id TEXT NOT NULL,
            row_no INTEGER NOT NULL,
            resource_type TEXT NOT NULL,
            name TEXT NOT NULL,
            env TEXT NOT NULL,
            status TEXT NOT NULL,
            error_message TEXT,
            payload_json TEXT NOT NULL,
            normalized_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS uk_import_job_rows_job_row
        ON import_job_rows(job_id, row_no);
        CREATE INDEX IF NOT EXISTS idx_import_job_rows_job
        ON import_job_rows(job_id);

        CREATE TABLE IF NOT EXISTS import_job_issues (
            id TEXT PRIMARY KEY,
            job_id TEXT NOT NULL,
            row_no INTEGER NOT NULL,
            field_key TEXT,
            issue_type TEXT NOT NULL,
            code TEXT NOT NULL,
            message TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_import_job_issues_job
        ON import_job_issues(job_id);
        CREATE INDEX IF NOT EXISTS idx_import_job_issues_job_row
        ON import_job_issues(job_id, row_no);
    
"#;
