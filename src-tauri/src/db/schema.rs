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
    (
        7,
        r#"
        CREATE TABLE IF NOT EXISTS ip_addresses (
            id TEXT PRIMARY KEY,
            ip_address TEXT NOT NULL,
            env TEXT NOT NULL DEFAULT 'prod',
            is_vip INTEGER NOT NULL DEFAULT 0,
            real_ips TEXT,
            description TEXT,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS uk_ip_addresses_ip_env
        ON ip_addresses(ip_address, env) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_ip_addresses_ip
        ON ip_addresses(ip_address) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_ip_addresses_env
        ON ip_addresses(env) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_ip_addresses_is_vip
        ON ip_addresses(is_vip) WHERE is_deleted = 0;

        DROP INDEX IF EXISTS uk_hosts_ip;

        CREATE TABLE IF NOT EXISTS host_ip_bindings (
            id TEXT PRIMARY KEY,
            host_id TEXT NOT NULL,
            ip_id TEXT NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS uk_host_ip_bindings_host_ip
        ON host_ip_bindings(host_id, ip_id) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_host_ip_bindings_host
        ON host_ip_bindings(host_id) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_host_ip_bindings_ip
        ON host_ip_bindings(ip_id) WHERE is_deleted = 0;

        INSERT INTO ip_addresses (id, ip_address, env, is_vip, real_ips, description, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), h.ip_address, h.env, 0, NULL, NULL, 0, NULL, h.created_at, h.updated_at
        FROM hosts h
        WHERE h.is_deleted = 0
          AND TRIM(COALESCE(h.ip_address, '')) <> ''
          AND NOT EXISTS (
            SELECT 1 FROM ip_addresses ia
            WHERE ia.ip_address = h.ip_address
              AND ia.env = h.env
              AND ia.is_deleted = 0
          );

        INSERT INTO host_ip_bindings (id, host_id, ip_id, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), h.id, ia.id, 0, NULL, h.created_at, h.updated_at
        FROM hosts h
        JOIN ip_addresses ia
          ON ia.ip_address = h.ip_address
         AND ia.env = h.env
         AND ia.is_deleted = 0
        LEFT JOIN host_ip_bindings hb
          ON hb.host_id = h.id
         AND hb.ip_id = ia.id
         AND hb.is_deleted = 0
        WHERE h.is_deleted = 0
          AND TRIM(COALESCE(h.ip_address, '')) <> ''
          AND hb.id IS NULL;
    "#,
    ),
    (
        8,
        r#"
        ALTER TABLE ip_addresses ADD COLUMN tags TEXT;
    "#,
    ),
    (
        9,
        r#"
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
    "#,
    ),
    (
        10,
        r#"
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
    "#,
    ),
    (
        11,
        r#"
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
    "#,
    ),
    (
        12,
        "-- taxonomy v2 migration handled by post-migration hook",
    ),
    (
        13,
        r#"
        DROP INDEX IF EXISTS uk_hosts_ip;
        ALTER TABLE hosts DROP COLUMN ip_address;
    "#,
    ),
    (
        14,
        r#"
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
    "#,
    ),
    (
        15,
        r#"
        INSERT OR IGNORE INTO taxonomy_terms (id, field_key, normalized_value, display_name, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), 'owner', lower(trim(owner)), trim(owner), 0, NULL, COALESCE(updated_at, datetime('now')), COALESCE(updated_at, datetime('now'))
        FROM business_applications
        WHERE is_deleted = 0
          AND TRIM(COALESCE(owner, '')) <> '';

        INSERT OR IGNORE INTO taxonomy_bindings (id, term_id, resource_type, resource_id, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), tt.id, 'business_application', ba.id, 0, NULL, COALESCE(ba.updated_at, datetime('now')), COALESCE(ba.updated_at, datetime('now'))
        FROM business_applications ba
        JOIN taxonomy_terms tt
          ON tt.field_key = 'owner'
         AND tt.normalized_value = lower(trim(ba.owner))
         AND tt.is_deleted = 0
        WHERE ba.is_deleted = 0
          AND TRIM(COALESCE(ba.owner, '')) <> '';

        DELETE FROM taxonomy_term_stats;
        INSERT INTO taxonomy_term_stats (term_id, resource_type, usage_count, last_used_at, updated_at)
        SELECT tb.term_id, tb.resource_type, COUNT(*) AS usage_count, MAX(tb.updated_at) AS last_used_at, datetime('now')
        FROM taxonomy_bindings tb
        JOIN taxonomy_terms tt ON tt.id = tb.term_id
        WHERE tb.is_deleted = 0
          AND tt.is_deleted = 0
        GROUP BY tb.term_id, tb.resource_type;

        ALTER TABLE business_applications DROP COLUMN owner;
    "#,
    ),
    (
        16,
        r#"
        DELETE FROM host_ip_bindings
         WHERE host_id IN (SELECT id FROM hosts WHERE is_deleted = 1)
            OR ip_id IN (SELECT id FROM ip_addresses WHERE is_deleted = 1)
            OR is_deleted = 1;

        DELETE FROM deployments WHERE is_deleted = 1;
        DELETE FROM call_relations WHERE is_deleted = 1;
        DELETE FROM applications WHERE is_deleted = 1;
        DELETE FROM business_applications WHERE is_deleted = 1;
        DELETE FROM middlewares WHERE is_deleted = 1;
        DELETE FROM nginx_configs WHERE is_deleted = 1;
        DELETE FROM hosts WHERE is_deleted = 1;
        DELETE FROM ip_addresses WHERE is_deleted = 1;
        DELETE FROM taxonomy_bindings WHERE is_deleted = 1;
        DELETE FROM taxonomy_terms WHERE is_deleted = 1;
    "#,
    ),
];

const TAXONOMY_TERMS_REQUIRED_COLUMNS: [&str; 8] = [
    "id",
    "field_key",
    "normalized_value",
    "display_name",
    "is_deleted",
    "deleted_at",
    "created_at",
    "updated_at",
];

const TAXONOMY_BINDINGS_REQUIRED_COLUMNS: [&str; 8] = [
    "id",
    "term_id",
    "resource_type",
    "resource_id",
    "is_deleted",
    "deleted_at",
    "created_at",
    "updated_at",
];

const TAXONOMY_STATS_REQUIRED_COLUMNS: [&str; 5] = [
    "term_id",
    "resource_type",
    "usage_count",
    "last_used_at",
    "updated_at",
];

const TAXONOMY_TERMS_LEGACY_COLUMNS: [&str; 2] = ["resource_type", "value"];
const TAXONOMY_BINDINGS_LEGACY_COLUMNS: [&str; 1] = ["field_key"];
const TAXONOMY_STATS_LEGACY_COLUMNS: [&str; 1] = ["field_key"];

const TAXONOMY_REQUIRED_INDEXES: [&str; 5] = [
    "uk_taxonomy_terms_field_normalized",
    "uk_taxonomy_bindings_term_resource",
    "idx_taxonomy_bindings_resource",
    "idx_taxonomy_bindings_term",
    "idx_taxonomy_term_stats_recent",
];

fn table_columns(
    conn: &rusqlite::Connection,
    table_name: &str,
) -> Result<std::collections::HashSet<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table_name))?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    let mut columns = std::collections::HashSet::new();
    for row in rows {
        columns.insert(row?);
    }
    Ok(columns)
}

fn has_required_columns(columns: &std::collections::HashSet<String>, required: &[&str]) -> bool {
    required.iter().all(|column| columns.contains(*column))
}

fn has_any_columns(columns: &std::collections::HashSet<String>, candidates: &[&str]) -> bool {
    candidates.iter().any(|column| columns.contains(*column))
}

fn index_exists(conn: &rusqlite::Connection, index_name: &str) -> Result<bool, rusqlite::Error> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?1",
        rusqlite::params![index_name],
        |row| row.get(0),
    )?;
    Ok(exists > 0)
}

fn has_required_indexes(conn: &rusqlite::Connection) -> Result<bool, rusqlite::Error> {
    for index_name in TAXONOMY_REQUIRED_INDEXES {
        if !index_exists(conn, index_name)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn ensure_taxonomy_indexes(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE UNIQUE INDEX IF NOT EXISTS uk_taxonomy_terms_field_normalized
         ON taxonomy_terms(field_key, normalized_value) WHERE is_deleted = 0;

         CREATE UNIQUE INDEX IF NOT EXISTS uk_taxonomy_bindings_term_resource
         ON taxonomy_bindings(term_id, resource_type, resource_id) WHERE is_deleted = 0;

         CREATE INDEX IF NOT EXISTS idx_taxonomy_bindings_resource
         ON taxonomy_bindings(resource_type, resource_id) WHERE is_deleted = 0;

         CREATE INDEX IF NOT EXISTS idx_taxonomy_bindings_term
         ON taxonomy_bindings(term_id) WHERE is_deleted = 0;

         CREATE INDEX IF NOT EXISTS idx_taxonomy_term_stats_recent
         ON taxonomy_term_stats(resource_type, last_used_at DESC);",
    )?;
    Ok(())
}

fn recreate_empty_taxonomy_v2_schema(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "DROP TABLE IF EXISTS taxonomy_term_stats;
         DROP TABLE IF EXISTS taxonomy_bindings;
         DROP TABLE IF EXISTS taxonomy_terms;

         CREATE TABLE taxonomy_terms (
             id TEXT PRIMARY KEY,
             field_key TEXT NOT NULL,
             normalized_value TEXT NOT NULL,
             display_name TEXT NOT NULL,
             is_deleted INTEGER NOT NULL DEFAULT 0,
             deleted_at TEXT,
             created_at TEXT NOT NULL,
             updated_at TEXT NOT NULL
         );

         CREATE TABLE taxonomy_bindings (
             id TEXT PRIMARY KEY,
             term_id TEXT NOT NULL,
             resource_type TEXT NOT NULL,
             resource_id TEXT NOT NULL,
             is_deleted INTEGER NOT NULL DEFAULT 0,
             deleted_at TEXT,
             created_at TEXT NOT NULL,
             updated_at TEXT NOT NULL
         );

         CREATE TABLE taxonomy_term_stats (
             term_id TEXT NOT NULL,
             resource_type TEXT NOT NULL,
             usage_count INTEGER NOT NULL DEFAULT 0,
             last_used_at TEXT,
             updated_at TEXT NOT NULL,
             PRIMARY KEY (term_id, resource_type)
         );",
    )?;
    ensure_taxonomy_indexes(conn)
}

pub fn ensure_taxonomy_schema_consistency(
    conn: &rusqlite::Connection,
) -> Result<(), rusqlite::Error> {
    let term_columns = table_columns(conn, "taxonomy_terms")?;
    let binding_columns = table_columns(conn, "taxonomy_bindings")?;
    let stats_columns = table_columns(conn, "taxonomy_term_stats")?;

    let terms_schema_ok = has_required_columns(&term_columns, &TAXONOMY_TERMS_REQUIRED_COLUMNS);
    let bindings_schema_ok =
        has_required_columns(&binding_columns, &TAXONOMY_BINDINGS_REQUIRED_COLUMNS);
    let stats_schema_ok = has_required_columns(&stats_columns, &TAXONOMY_STATS_REQUIRED_COLUMNS);
    let terms_has_legacy_columns = has_any_columns(&term_columns, &TAXONOMY_TERMS_LEGACY_COLUMNS);
    let bindings_has_legacy_columns =
        has_any_columns(&binding_columns, &TAXONOMY_BINDINGS_LEGACY_COLUMNS);
    let stats_has_legacy_columns = has_any_columns(&stats_columns, &TAXONOMY_STATS_LEGACY_COLUMNS);
    let indexes_ok = has_required_indexes(conn)?;

    if terms_schema_ok
        && bindings_schema_ok
        && stats_schema_ok
        && !terms_has_legacy_columns
        && !bindings_has_legacy_columns
        && !stats_has_legacy_columns
    {
        if indexes_ok {
            return Ok(());
        }
        return ensure_taxonomy_indexes(conn);
    }

    let can_migrate_from_existing =
        has_required_columns(&term_columns, &TAXONOMY_TERMS_REQUIRED_COLUMNS)
            && has_required_columns(&binding_columns, &TAXONOMY_BINDINGS_REQUIRED_COLUMNS);

    conn.execute_batch("BEGIN IMMEDIATE TRANSACTION;")?;

    let repair_result = if can_migrate_from_existing {
        migrate_taxonomy_v2(conn)
    } else {
        recreate_empty_taxonomy_v2_schema(conn)
    };

    match repair_result {
        Ok(_) => {
            ensure_taxonomy_indexes(conn)?;
            conn.execute_batch("COMMIT;")?;
            Ok(())
        }
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

/// Taxonomy v2 migration: merge terms by (field_key, normalized_value), rebuild tables.
///
/// Old schema had terms keyed by (resource_type, field_key, normalized_value).
/// New schema keys terms by (field_key, normalized_value) only, removing resource_type
/// from terms and field_key from bindings/stats.
pub fn migrate_taxonomy_v2(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    use std::collections::HashMap;

    // --- Phase 1: Read old data into memory ---

    // Old terms: id, resource_type, field_key, display_name, normalized_value, is_deleted, deleted_at, created_at, updated_at
    struct OldTerm {
        id: String,
        field_key: String,
        display_name: String,
        normalized_value: String,
        is_deleted: i32,
        deleted_at: Option<String>,
        created_at: String,
        updated_at: String,
    }

    let mut old_terms: Vec<OldTerm> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, field_key, display_name, normalized_value, is_deleted, deleted_at, created_at, updated_at
             FROM taxonomy_terms",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(OldTerm {
                id: row.get(0)?,
                field_key: row.get(1)?,
                display_name: row.get(2)?,
                normalized_value: row.get(3)?,
                is_deleted: row.get(4)?,
                deleted_at: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        for row in rows {
            old_terms.push(row?);
        }
    }

    // Old bindings: id, term_id, resource_type, resource_id, is_deleted, deleted_at, created_at, updated_at
    struct OldBinding {
        id: String,
        term_id: String,
        resource_type: String,
        resource_id: String,
        is_deleted: i32,
        deleted_at: Option<String>,
        created_at: String,
        updated_at: String,
    }

    let mut old_bindings: Vec<OldBinding> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, term_id, resource_type, resource_id, is_deleted, deleted_at, created_at, updated_at
             FROM taxonomy_bindings",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(OldBinding {
                id: row.get(0)?,
                term_id: row.get(1)?,
                resource_type: row.get(2)?,
                resource_id: row.get(3)?,
                is_deleted: row.get(4)?,
                deleted_at: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        for row in rows {
            old_bindings.push(row?);
        }
    }

    // --- Phase 2: Merge terms by (field_key, normalized_value) ---

    struct MergedTerm {
        id: String,
        field_key: String,
        normalized_value: String,
        display_name: String,
        is_deleted: i32,
        deleted_at: Option<String>,
        created_at: String,
        updated_at: String,
    }

    // Key: (field_key, normalized_value) -> MergedTerm
    let mut merged_terms: HashMap<(String, String), MergedTerm> = HashMap::new();
    // Map: old_term_id -> new_term_id
    let mut id_map: HashMap<String, String> = HashMap::new();

    for term in &old_terms {
        let key = (term.field_key.clone(), term.normalized_value.clone());
        match merged_terms.get_mut(&key) {
            Some(existing) => {
                // Keep earliest created_at, latest display_name (by updated_at)
                if term.created_at < existing.created_at {
                    existing.created_at = term.created_at.clone();
                }
                if term.updated_at > existing.updated_at {
                    existing.display_name = term.display_name.clone();
                    existing.updated_at = term.updated_at.clone();
                }
                // If any active copy exists, mark as active
                if term.is_deleted == 0 {
                    existing.is_deleted = 0;
                    existing.deleted_at = None;
                }
                id_map.insert(term.id.clone(), existing.id.clone());
            }
            None => {
                let new_id = uuid::Uuid::new_v4().to_string();
                id_map.insert(term.id.clone(), new_id.clone());
                merged_terms.insert(
                    key,
                    MergedTerm {
                        id: new_id,
                        field_key: term.field_key.clone(),
                        normalized_value: term.normalized_value.clone(),
                        display_name: term.display_name.clone(),
                        is_deleted: term.is_deleted,
                        deleted_at: term.deleted_at.clone(),
                        created_at: term.created_at.clone(),
                        updated_at: term.updated_at.clone(),
                    },
                );
            }
        }
    }

    // --- Phase 3: Drop old tables and create new ones ---

    conn.execute_batch(
        "DROP TABLE IF EXISTS taxonomy_term_stats;
         DROP TABLE IF EXISTS taxonomy_bindings;
         DROP TABLE IF EXISTS taxonomy_terms;

         CREATE TABLE taxonomy_terms (
             id TEXT PRIMARY KEY,
             field_key TEXT NOT NULL,
             normalized_value TEXT NOT NULL,
             display_name TEXT NOT NULL,
             is_deleted INTEGER NOT NULL DEFAULT 0,
             deleted_at TEXT,
             created_at TEXT NOT NULL,
             updated_at TEXT NOT NULL
         );

         CREATE UNIQUE INDEX uk_taxonomy_terms_field_normalized
             ON taxonomy_terms(field_key, normalized_value) WHERE is_deleted = 0;

         CREATE TABLE taxonomy_bindings (
             id TEXT PRIMARY KEY,
             term_id TEXT NOT NULL,
             resource_type TEXT NOT NULL,
             resource_id TEXT NOT NULL,
             is_deleted INTEGER NOT NULL DEFAULT 0,
             deleted_at TEXT,
             created_at TEXT NOT NULL,
             updated_at TEXT NOT NULL
         );

         CREATE UNIQUE INDEX uk_taxonomy_bindings_term_resource
             ON taxonomy_bindings(term_id, resource_type, resource_id) WHERE is_deleted = 0;
         CREATE INDEX idx_taxonomy_bindings_resource
             ON taxonomy_bindings(resource_type, resource_id) WHERE is_deleted = 0;
         CREATE INDEX idx_taxonomy_bindings_term
             ON taxonomy_bindings(term_id) WHERE is_deleted = 0;

         CREATE TABLE taxonomy_term_stats (
             term_id TEXT NOT NULL,
             resource_type TEXT NOT NULL,
             usage_count INTEGER NOT NULL DEFAULT 0,
             last_used_at TEXT,
             updated_at TEXT NOT NULL,
             PRIMARY KEY (term_id, resource_type)
         );

         CREATE INDEX idx_taxonomy_term_stats_recent
             ON taxonomy_term_stats(resource_type, last_used_at DESC);",
    )?;

    // --- Phase 4: Insert merged terms ---

    {
        let mut insert_stmt = conn.prepare(
            "INSERT INTO taxonomy_terms (id, field_key, normalized_value, display_name, is_deleted, deleted_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        for term in merged_terms.values() {
            insert_stmt.execute(rusqlite::params![
                term.id,
                term.field_key,
                term.normalized_value,
                term.display_name,
                term.is_deleted,
                term.deleted_at,
                term.created_at,
                term.updated_at,
            ])?;
        }
    }

    // --- Phase 5: Recreate bindings with mapped term_ids ---

    {
        let mut insert_stmt = conn.prepare(
            "INSERT OR IGNORE INTO taxonomy_bindings (id, term_id, resource_type, resource_id, is_deleted, deleted_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        for binding in &old_bindings {
            if let Some(new_term_id) = id_map.get(&binding.term_id) {
                insert_stmt.execute(rusqlite::params![
                    binding.id,
                    new_term_id,
                    binding.resource_type,
                    binding.resource_id,
                    binding.is_deleted,
                    binding.deleted_at,
                    binding.created_at,
                    binding.updated_at,
                ])?;
            }
        }
    }

    // --- Phase 6: Rebuild stats ---

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO taxonomy_term_stats (term_id, resource_type, usage_count, last_used_at, updated_at)
         SELECT tb.term_id, tb.resource_type, COUNT(*) AS usage_count, MAX(tb.updated_at) AS last_used_at, ?1 AS updated_at
         FROM taxonomy_bindings tb
         JOIN taxonomy_terms tt ON tt.id = tb.term_id
         WHERE tb.is_deleted = 0
           AND tt.is_deleted = 0
         GROUP BY tb.term_id, tb.resource_type",
        rusqlite::params![now],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ensure_taxonomy_schema_consistency, migrate_taxonomy_v2, MIGRATIONS};
    use rusqlite::Connection;

    fn apply_all_migrations(conn: &Connection) {
        for (version, sql) in MIGRATIONS.iter() {
            conn.execute_batch(sql)
                .unwrap_or_else(|e| panic!("Migration v{} failed: {}", version, e));
            if *version == 12 {
                migrate_taxonomy_v2(conn)
                    .unwrap_or_else(|e| panic!("Post-migration hook v{} failed: {}", version, e));
            }
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
    fn hosts_table_should_not_have_legacy_ip_address_column_after_migrations() {
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
            !columns.iter().any(|col| col == "ip_address"),
            "hosts table should not contain legacy ip_address column"
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

    #[test]
    fn ip_addresses_and_bindings_tables_should_exist_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let ip_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='ip_addresses'",
                [],
                |row| row.get(0),
            )
            .expect("query ip_addresses");
        let binding_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='host_ip_bindings'",
                [],
                |row| row.get(0),
            )
            .expect("query host_ip_bindings");

        assert_eq!(ip_exists, 1, "ip_addresses table should exist");
        assert_eq!(binding_exists, 1, "host_ip_bindings table should exist");
    }

    #[test]
    fn migration_should_move_existing_host_ip_to_ip_resources() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        for (version, sql) in MIGRATIONS.iter() {
            if *version == 1 {
                conn.execute_batch(sql)
                    .unwrap_or_else(|e| panic!("Migration v{} failed: {}", version, e));
                conn.execute(
                    "INSERT INTO hosts (id, hostname, ip_address, status, is_deleted, created_at, updated_at)
                     VALUES ('h1', 'server-1', '10.8.0.1', 'running', 0, datetime('now'), datetime('now'))",
                    [],
                )
                .expect("seed host");
                continue;
            }

            conn.execute_batch(sql)
                .unwrap_or_else(|e| panic!("Migration v{} failed: {}", version, e));
            if *version == 12 {
                migrate_taxonomy_v2(&conn)
                    .unwrap_or_else(|e| panic!("Post-migration hook v{} failed: {}", version, e));
            }
        }

        let ip_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM ip_addresses WHERE ip_address='10.8.0.1' AND env='prod' AND is_deleted=0",
                [],
                |row| row.get(0),
            )
            .expect("count migrated ip");
        let binding_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM host_ip_bindings hb
                 JOIN ip_addresses ia ON ia.id = hb.ip_id
                 WHERE hb.host_id='h1' AND ia.ip_address='10.8.0.1' AND hb.is_deleted=0",
                [],
                |row| row.get(0),
            )
            .expect("count migrated binding");

        assert_eq!(ip_count, 1, "host ip should be migrated into ip_addresses");
        assert_eq!(
            binding_count, 1,
            "host should be bound with migrated ip address"
        );
    }

    #[test]
    fn ip_addresses_table_should_have_tags_column_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let mut stmt = conn
            .prepare("PRAGMA table_info(ip_addresses)")
            .expect("prepare table info query");
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .expect("query table info");
        let columns: Vec<String> = rows.filter_map(|r| r.ok()).collect();

        assert!(
            columns.iter().any(|col| col == "tags"),
            "ip_addresses table should contain tags column"
        );
    }

    #[test]
    fn business_applications_table_should_exist_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='business_applications'",
                [],
                |row| row.get(0),
            )
            .expect("query sqlite_master");

        assert_eq!(exists, 1, "business_applications table should exist");

        let mut stmt = conn
            .prepare("PRAGMA table_info(business_applications)")
            .expect("prepare table info query");
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .expect("query table info");
        let columns: Vec<String> = rows.filter_map(|r| r.ok()).collect();
        assert!(
            !columns.iter().any(|col| col == "owner"),
            "business_applications table should not contain legacy owner column"
        );
    }

    #[test]
    fn applications_table_should_have_business_application_id_column_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let mut stmt = conn
            .prepare("PRAGMA table_info(applications)")
            .expect("prepare table info query");
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .expect("query table info");
        let columns: Vec<String> = rows.filter_map(|r| r.ok()).collect();

        assert!(
            columns.iter().any(|col| col == "business_application_id"),
            "applications table should contain business_application_id column"
        );
    }

    #[test]
    fn business_application_owner_should_migrate_to_taxonomy_before_owner_column_drop() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        for (version, sql) in MIGRATIONS.iter() {
            if *version == 15 {
                conn.execute(
                    "INSERT INTO business_applications (id, name, code, owner, description, env, status, is_deleted, deleted_at, created_at, updated_at)
                     VALUES ('ba-mig-1', '支付中心', 'PAY', 'alice', NULL, 'prod', 'active', 0, NULL, datetime('now'), datetime('now'))",
                    [],
                )
                .expect("seed business application owner before migration v15");
            }

            conn.execute_batch(sql)
                .unwrap_or_else(|e| panic!("Migration v{} failed: {}", version, e));
            if *version == 12 {
                migrate_taxonomy_v2(&conn)
                    .unwrap_or_else(|e| panic!("Post-migration hook v{} failed: {}", version, e));
            }
        }

        let owner_binding_count: i64 = conn
            .query_row(
                "SELECT COUNT(*)
                 FROM taxonomy_bindings tb
                 JOIN taxonomy_terms tt ON tt.id = tb.term_id
                 WHERE tb.resource_type = 'business_application'
                   AND tb.resource_id = 'ba-mig-1'
                   AND tb.is_deleted = 0
                   AND tt.is_deleted = 0
                   AND tt.field_key = 'owner'
                   AND tt.display_name = 'alice'",
                [],
                |row| row.get(0),
            )
            .expect("count migrated business owner binding");
        assert_eq!(
            owner_binding_count, 1,
            "business application owner should be migrated into taxonomy"
        );
    }

    #[test]
    fn taxonomy_tables_should_have_correct_v2_schema_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let terms_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='taxonomy_terms'",
                [],
                |row| row.get(0),
            )
            .expect("query taxonomy_terms");
        let bindings_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='taxonomy_bindings'",
                [],
                |row| row.get(0),
            )
            .expect("query taxonomy_bindings");

        assert_eq!(terms_exists, 1, "taxonomy_terms table should exist");
        assert_eq!(bindings_exists, 1, "taxonomy_bindings table should exist");

        // Verify taxonomy_terms has no resource_type column (v2 removed it)
        let mut stmt = conn
            .prepare("PRAGMA table_info(taxonomy_terms)")
            .expect("prepare table info query");
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .expect("query table info");
        let columns: Vec<String> = rows.filter_map(|r| r.ok()).collect();
        assert!(
            !columns.iter().any(|col| col == "resource_type"),
            "taxonomy_terms should not have resource_type column in v2"
        );
        assert!(
            !columns.iter().any(|col| col == "value"),
            "taxonomy_terms should not have value column in v2"
        );
        assert!(
            columns.iter().any(|col| col == "field_key"),
            "taxonomy_terms should have field_key column"
        );

        // Verify taxonomy_bindings has no field_key column (v2 removed it)
        let mut stmt = conn
            .prepare("PRAGMA table_info(taxonomy_bindings)")
            .expect("prepare table info query");
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .expect("query table info");
        let columns: Vec<String> = rows.filter_map(|r| r.ok()).collect();
        assert!(
            !columns.iter().any(|col| col == "field_key"),
            "taxonomy_bindings should not have field_key column in v2"
        );
    }

    #[test]
    fn taxonomy_term_stats_table_should_have_composite_pk_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let stats_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='taxonomy_term_stats'",
                [],
                |row| row.get(0),
            )
            .expect("query taxonomy_term_stats");
        assert_eq!(stats_exists, 1, "taxonomy_term_stats table should exist");

        // Verify no field_key column (v2 removed it)
        let mut stmt = conn
            .prepare("PRAGMA table_info(taxonomy_term_stats)")
            .expect("prepare table info query");
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .expect("query table info");
        let columns: Vec<String> = rows.filter_map(|r| r.ok()).collect();
        assert!(
            !columns.iter().any(|col| col == "field_key"),
            "taxonomy_term_stats should not have field_key column in v2"
        );
        assert!(
            columns.iter().any(|col| col == "term_id"),
            "taxonomy_term_stats should have term_id column"
        );
        assert!(
            columns.iter().any(|col| col == "resource_type"),
            "taxonomy_term_stats should have resource_type column"
        );
    }

    #[test]
    fn call_relations_table_should_exist_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='call_relations'",
                [],
                |row| row.get(0),
            )
            .expect("query sqlite_master");

        assert_eq!(exists, 1, "call_relations table should exist");
    }

    #[test]
    fn dependencies_table_should_be_dropped_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='dependencies'",
                [],
                |row| row.get(0),
            )
            .expect("query sqlite_master");

        assert_eq!(exists, 0, "legacy dependencies table should be dropped");
    }

    #[test]
    fn ensure_taxonomy_schema_consistency_should_repair_missing_index() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        conn.execute("DROP INDEX IF EXISTS idx_taxonomy_term_stats_recent", [])
            .expect("drop index");

        ensure_taxonomy_schema_consistency(&conn).expect("repair taxonomy schema");

        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_taxonomy_term_stats_recent'",
                [],
                |row| row.get(0),
            )
            .expect("query index existence");
        assert_eq!(exists, 1, "missing index should be recreated");
    }

    #[test]
    fn ensure_taxonomy_schema_consistency_should_repair_legacy_stats_schema() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        conn.execute_batch(
            "DROP TABLE IF EXISTS taxonomy_term_stats;
             CREATE TABLE taxonomy_term_stats (
                term_id TEXT PRIMARY KEY,
                resource_type TEXT NOT NULL,
                field_key TEXT NOT NULL,
                usage_count INTEGER NOT NULL DEFAULT 0,
                last_used_at TEXT,
                updated_at TEXT NOT NULL
             );",
        )
        .expect("create legacy stats schema");

        ensure_taxonomy_schema_consistency(&conn).expect("repair taxonomy schema");

        let mut stmt = conn
            .prepare("PRAGMA table_info(taxonomy_term_stats)")
            .expect("prepare table info");
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .expect("query table info");
        let columns: Vec<String> = rows.filter_map(|r| r.ok()).collect();

        assert!(
            !columns.iter().any(|col| col == "field_key"),
            "repaired schema should not contain legacy field_key column"
        );
        assert!(
            columns.iter().any(|col| col == "resource_type"),
            "repaired schema should keep resource_type column"
        );
    }
}
