pub const MIGRATIONS: &[(i32, &str)] = &[
    (
        1,
        r#"
        CREATE TABLE IF NOT EXISTS hosts (
            id TEXT PRIMARY KEY,
            hostname TEXT NOT NULL,
            ip_address TEXT NOT NULL,
            os_type TEXT,
            cpu_info TEXT,
            ram_gb INTEGER,
            disk_gb INTEGER,
            status TEXT NOT NULL DEFAULT 'running',
            tags TEXT,
            description TEXT,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS applications (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            type TEXT NOT NULL,
            address TEXT,
            port INTEGER,
            tech_stack TEXT,
            deploy_mode TEXT,
            env TEXT NOT NULL,
            git_repo TEXT,
            owner TEXT,
            status TEXT NOT NULL DEFAULT 'running',
            description TEXT,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS middlewares (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            type TEXT NOT NULL,
            address TEXT NOT NULL,
            port INTEGER,
            version TEXT,
            env TEXT NOT NULL,
            description TEXT,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS nginx_configs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            listen_port INTEGER,
            strategy TEXT,
            upstream_servers TEXT,
            env TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'running',
            description TEXT,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS deployments (
            id TEXT PRIMARY KEY,
            resource_id TEXT NOT NULL,
            resource_type TEXT NOT NULL,
            host_id TEXT NOT NULL,
            port INTEGER,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS dependencies (
            id TEXT PRIMARY KEY,
            source_id TEXT NOT NULL,
            source_type TEXT NOT NULL,
            target_id TEXT NOT NULL,
            target_type TEXT NOT NULL,
            relation_type TEXT NOT NULL,
            description TEXT,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS audit_logs (
            id TEXT PRIMARY KEY,
            action TEXT NOT NULL,
            resource_type TEXT NOT NULL,
            resource_id TEXT NOT NULL,
            resource_name TEXT,
            changes TEXT,
            created_at TEXT NOT NULL
        );

        -- Indexes: hosts
        CREATE UNIQUE INDEX uk_hosts_ip ON hosts(ip_address) WHERE is_deleted = 0;
        CREATE INDEX idx_hosts_hostname ON hosts(hostname) WHERE is_deleted = 0;
        CREATE INDEX idx_hosts_status ON hosts(status) WHERE is_deleted = 0;

        -- Indexes: applications
        CREATE UNIQUE INDEX uk_applications_name_env ON applications(name, env) WHERE is_deleted = 0;
        CREATE INDEX idx_applications_type ON applications(type) WHERE is_deleted = 0;
        CREATE INDEX idx_applications_env ON applications(env) WHERE is_deleted = 0;
        CREATE INDEX idx_applications_status ON applications(status) WHERE is_deleted = 0;
        CREATE INDEX idx_applications_owner ON applications(owner) WHERE is_deleted = 0;

        -- Indexes: middlewares
        CREATE UNIQUE INDEX uk_middlewares_name_env ON middlewares(name, env) WHERE is_deleted = 0;
        CREATE INDEX idx_middlewares_category ON middlewares(category) WHERE is_deleted = 0;
        CREATE INDEX idx_middlewares_type ON middlewares(type) WHERE is_deleted = 0;
        CREATE INDEX idx_middlewares_env ON middlewares(env) WHERE is_deleted = 0;

        -- Indexes: nginx_configs
        CREATE UNIQUE INDEX uk_nginx_configs_name_env ON nginx_configs(name, env) WHERE is_deleted = 0;
        CREATE INDEX idx_nginx_configs_env ON nginx_configs(env) WHERE is_deleted = 0;
        CREATE INDEX idx_nginx_configs_status ON nginx_configs(status) WHERE is_deleted = 0;

        -- Indexes: deployments
        CREATE UNIQUE INDEX uk_deployments_resource_host ON deployments(resource_id, resource_type, host_id) WHERE is_deleted = 0;
        CREATE INDEX idx_deployments_host_id ON deployments(host_id) WHERE is_deleted = 0;
        CREATE INDEX idx_deployments_resource ON deployments(resource_id, resource_type) WHERE is_deleted = 0;

        -- Indexes: dependencies
        CREATE UNIQUE INDEX uk_dependencies_source_target ON dependencies(source_id, target_id, relation_type) WHERE is_deleted = 0;
        CREATE INDEX idx_dependencies_source ON dependencies(source_id, source_type) WHERE is_deleted = 0;
        CREATE INDEX idx_dependencies_target ON dependencies(target_id, target_type) WHERE is_deleted = 0;
        CREATE INDEX idx_dependencies_relation_type ON dependencies(relation_type) WHERE is_deleted = 0;

        -- Indexes: audit_logs
        CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
        CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
    "#,
    ),
    (
        2,
        r#"
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
    "#,
    ),
    (
        3,
        r#"
        ALTER TABLE hosts ADD COLUMN cpu_model TEXT;
        ALTER TABLE hosts ADD COLUMN cpu_cores INTEGER;
        ALTER TABLE hosts ADD COLUMN cpu_threads INTEGER;
        ALTER TABLE hosts ADD COLUMN cpu_freq TEXT;
    "#,
    ),
    (
        4,
        r#"
        ALTER TABLE hosts ADD COLUMN env TEXT NOT NULL DEFAULT 'prod';
        CREATE INDEX IF NOT EXISTS idx_hosts_env ON hosts(env) WHERE is_deleted = 0;
    "#,
    ),
    (
        5,
        r#"
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
    "#,
    ),
    (
        6,
        r#"
        ALTER TABLE nginx_configs ADD COLUMN address TEXT NOT NULL DEFAULT '';
        UPDATE nginx_configs SET address = 'unknown' WHERE TRIM(address) = '';
    "#,
    ),
];

#[cfg(test)]
mod tests {
    use super::MIGRATIONS;
    use rusqlite::Connection;

    fn apply_all_migrations(conn: &Connection) {
        for (version, sql) in MIGRATIONS.iter() {
            conn.execute_batch(sql)
                .unwrap_or_else(|e| panic!("Migration v{} failed: {}", version, e));
        }
    }

    #[test]
    fn hosts_table_should_have_env_column_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let mut stmt = conn
            .prepare("PRAGMA table_info(hosts)")
            .expect("prepare table info query");
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .expect("query table info");
        let columns: Vec<String> = rows.filter_map(|r| r.ok()).collect();

        assert!(
            columns.iter().any(|col| col == "env"),
            "hosts table should contain env column"
        );
    }

    #[test]
    fn application_owners_table_should_exist_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='application_owners'",
                [],
                |row| row.get(0),
            )
            .expect("query sqlite_master");

        assert_eq!(exists, 1, "application_owners table should exist");
    }

    #[test]
    fn nginx_configs_table_should_have_address_column_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let mut stmt = conn
            .prepare("PRAGMA table_info(nginx_configs)")
            .expect("prepare table info query");
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .expect("query table info");
        let columns: Vec<String> = rows.filter_map(|r| r.ok()).collect();

        assert!(
            columns.iter().any(|col| col == "address"),
            "nginx_configs table should contain address column"
        );
    }
}
