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
    use super::{ensure_taxonomy_schema_consistency, migrate_taxonomy_v2};
    use crate::db::migrations::MIGRATIONS;
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
    fn application_owners_table_should_not_exist_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='application_owners'",
                [],
                |row| row.get(0),
            )
            .expect("query sqlite_master");

        assert_eq!(exists, 0, "application_owners table should be removed");
    }

    #[test]
    fn applications_table_should_not_have_legacy_owner_column_after_migrations() {
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
            !columns.iter().any(|col| col == "owner"),
            "applications table should not contain legacy owner column"
        );
    }

    #[test]
    fn applications_owner_index_should_not_exist_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_applications_owner'",
                [],
                |row| row.get(0),
            )
            .expect("query sqlite_master for owner index");

        assert_eq!(exists, 0, "applications owner index should be removed");
    }

    #[test]
    fn nginx_configs_table_should_have_endpoints_column_after_migrations() {
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
            columns.iter().any(|col| col == "endpoints"),
            "nginx_configs table should contain endpoints column"
        );
        assert!(
            !columns.iter().any(|col| col == "address"),
            "nginx_configs table should not contain legacy address column"
        );
        assert!(
            !columns.iter().any(|col| col == "listen_port"),
            "nginx_configs table should not contain legacy listen_port column"
        );
        assert!(
            !columns.iter().any(|col| col == "upstream_servers"),
            "nginx_configs table should not contain legacy upstream_servers column"
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
    fn v002_migration_should_rename_business_applications_to_systems() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        // Apply v23 (initial schema) which creates business_applications and applications tables
        for (version, sql) in MIGRATIONS.iter() {
            if *version == 24 {
                // Before v24, seed data in old tables
                conn.execute(
                    "INSERT INTO business_applications (id, name, code, description, env, status, is_deleted, deleted_at, created_at, updated_at)
                     VALUES ('ba-1', '支付系统', 'PAY', NULL, 'prod', 'active', 0, NULL, datetime('now'), datetime('now'))",
                    [],
                )
                .expect("seed business_application");
                conn.execute(
                    "INSERT INTO applications (id, name, type, env, business_application_id, status, is_deleted, created_at, updated_at)
                     VALUES ('app-1', '订单服务', 'backend', 'prod', 'ba-1', 'running', 0, datetime('now'), datetime('now'))",
                    [],
                )
                .expect("seed application");
            }

            conn.execute_batch(sql)
                .unwrap_or_else(|e| panic!("Migration v{} failed: {}", version, e));
        }

        // Verify systems table has the data
        let sys_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM systems WHERE id = 'ba-1' AND name = '支付系统' AND env = 'prod' AND is_deleted = 0",
                [],
                |row| row.get(0),
            )
            .expect("query systems");
        assert_eq!(sys_count, 1, "business_application should be migrated to systems");

        // Verify services table has the data with system_id
        let svc_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM services WHERE id = 'app-1' AND system_id = 'ba-1' AND is_deleted = 0",
                [],
                |row| row.get(0),
            )
            .expect("query services");
        assert_eq!(svc_count, 1, "application should be migrated to services with system_id");

        // Verify old tables are gone
        let old_ba: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='business_applications'",
                [],
                |row| row.get(0),
            )
            .expect("query old table");
        assert_eq!(old_ba, 0, "business_applications table should be dropped");

        let old_app: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='applications'",
                [],
                |row| row.get(0),
            )
            .expect("query old table");
        assert_eq!(old_app, 0, "applications table should be dropped");
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
    fn systems_table_should_exist_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='systems'",
                [],
                |row| row.get(0),
            )
            .expect("query sqlite_master");

        assert_eq!(exists, 1, "systems table should exist");
    }

    #[test]
    fn services_table_should_have_system_id_column_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let mut stmt = conn
            .prepare("PRAGMA table_info(services)")
            .expect("prepare table info query");
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .expect("query table info");
        let columns: Vec<String> = rows.filter_map(|r| r.ok()).collect();

        assert!(
            columns.iter().any(|col| col == "system_id"),
            "services table should contain system_id column"
        );
    }

    #[test]
    fn services_should_not_have_legacy_unique_index_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let old_index_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='uk_applications_name_env'",
                [],
                |row| row.get(0),
            )
            .expect("query old applications unique index");
        assert_eq!(
            old_index_exists, 0,
            "legacy applications unique index should not exist"
        );

        let old_type_index_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='uk_applications_name_env_type'",
                [],
                |row| row.get(0),
            )
            .expect("query old applications type index");
        assert_eq!(
            old_type_index_exists, 0,
            "legacy applications type index should not exist"
        );
    }

    #[test]
    fn v002_migration_should_update_taxonomy_resource_types() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        for (version, sql) in MIGRATIONS.iter() {
            if *version == 24 {
                // Seed taxonomy bindings with old resource_type values before v24
                let now = chrono::Utc::now().to_rfc3339();
                conn.execute(
                    "INSERT INTO taxonomy_terms (id, field_key, normalized_value, display_name, is_deleted, deleted_at, created_at, updated_at)
                     VALUES ('tt-1', 'owner', 'alice', 'alice', 0, NULL, ?1, ?1)",
                    rusqlite::params![now],
                )
                .expect("seed taxonomy term");
                conn.execute(
                    "INSERT INTO taxonomy_bindings (id, term_id, resource_type, resource_id, is_deleted, deleted_at, created_at, updated_at)
                     VALUES ('tb-1', 'tt-1', 'business_application', 'ba-1', 0, NULL, ?1, ?1)",
                    rusqlite::params![now],
                )
                .expect("seed taxonomy binding for business_application");
                conn.execute(
                    "INSERT INTO taxonomy_bindings (id, term_id, resource_type, resource_id, is_deleted, deleted_at, created_at, updated_at)
                     VALUES ('tb-2', 'tt-1', 'application', 'app-1', 0, NULL, ?1, ?1)",
                    rusqlite::params![now],
                )
                .expect("seed taxonomy binding for application");
            }

            conn.execute_batch(sql)
                .unwrap_or_else(|e| panic!("Migration v{} failed: {}", version, e));
        }

        // Verify resource_type was updated from 'business_application' to 'system'
        let system_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM taxonomy_bindings WHERE resource_type = 'system' AND resource_id = 'ba-1' AND is_deleted = 0",
                [],
                |row| row.get(0),
            )
            .expect("count system bindings");
        assert_eq!(system_count, 1, "business_application resource_type should be migrated to system");

        // Verify resource_type was updated from 'application' to 'service'
        let service_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM taxonomy_bindings WHERE resource_type = 'service' AND resource_id = 'app-1' AND is_deleted = 0",
                [],
                |row| row.get(0),
            )
            .expect("count service bindings");
        assert_eq!(service_count, 1, "application resource_type should be migrated to service");
    }

    #[test]
    fn v002_migration_should_auto_create_system_for_orphan_applications() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        for (version, sql) in MIGRATIONS.iter() {
            if *version == 24 {
                // Seed an application without business_application_id before v24
                conn.execute(
                    "INSERT INTO applications (id, name, type, env, business_application_id, status, is_deleted, created_at, updated_at)
                     VALUES ('app-orphan-1', 'orphan-app', 'backend', 'prod', NULL, 'running', 0, datetime('now'), datetime('now'))",
                    [],
                )
                .expect("seed orphan application");
            }

            conn.execute_batch(sql)
                .unwrap_or_else(|e| panic!("Migration v{} failed: {}", version, e));
        }

        // Verify a system was auto-created for the orphan
        let auto_sys_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM systems WHERE name = 'orphan-app' AND env = 'prod' AND is_deleted = 0",
                [],
                |row| row.get(0),
            )
            .expect("count auto-created systems");
        assert_eq!(auto_sys_count, 1, "an auto system should be created for orphan application");

        // Verify the service was linked to the auto-created system
        let linked_service: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM services s
                 JOIN systems sys ON sys.id = s.system_id
                 WHERE s.id = 'app-orphan-1' AND sys.name = 'orphan-app' AND s.is_deleted = 0",
                [],
                |row| row.get(0),
            )
            .expect("count linked services");
        assert_eq!(linked_service, 1, "orphan application should be linked to auto-created system");
    }

    #[test]
    fn taxonomy_tables_should_support_host_os_and_cpu_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        // Verify taxonomy tables exist and can store host os/cpu data
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO hosts (id, hostname, os_type, cpu_model, env, status, is_deleted, created_at, updated_at)
             VALUES ('host-tax-1', 'test-host', 'openEuler', 'Hygon C86 7285', 'prod', 'running', 0, ?1, ?1)",
            rusqlite::params![now],
        )
        .expect("insert host");

        conn.execute(
            "INSERT INTO taxonomy_terms (id, field_key, normalized_value, display_name, is_deleted, deleted_at, created_at, updated_at)
             VALUES ('tt-os-1', 'os_type', 'openeuler', 'openEuler', 0, NULL, ?1, ?1)",
            rusqlite::params![now],
        )
        .expect("insert os taxonomy term");

        conn.execute(
            "INSERT INTO taxonomy_bindings (id, term_id, resource_type, resource_id, is_deleted, deleted_at, created_at, updated_at)
             VALUES ('tb-os-1', 'tt-os-1', 'host', 'host-tax-1', 0, NULL, ?1, ?1)",
            rusqlite::params![now],
        )
        .expect("insert os taxonomy binding");

        let os_binding_count: i64 = conn
            .query_row(
                "SELECT COUNT(*)
                 FROM taxonomy_bindings tb
                 JOIN taxonomy_terms tt ON tt.id = tb.term_id
                 WHERE tb.resource_type = 'host'
                   AND tb.resource_id = 'host-tax-1'
                   AND tb.is_deleted = 0
                   AND tt.is_deleted = 0
                   AND tt.field_key = 'os_type'
                   AND tt.display_name = 'openEuler'",
                [],
                |row| row.get(0),
            )
            .expect("count host os binding");
        assert_eq!(
            os_binding_count, 1,
            "host os_type should be stored in taxonomy"
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
    fn import_job_tables_should_exist_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let jobs_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='import_jobs'",
                [],
                |row| row.get(0),
            )
            .expect("query import_jobs");
        let rows_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='import_job_rows'",
                [],
                |row| row.get(0),
            )
            .expect("query import_job_rows");
        let issues_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='import_job_issues'",
                [],
                |row| row.get(0),
            )
            .expect("query import_job_issues");

        assert_eq!(jobs_exists, 1, "import_jobs table should exist");
        assert_eq!(rows_exists, 1, "import_job_rows table should exist");
        assert_eq!(issues_exists, 1, "import_job_issues table should exist");
    }

    #[test]
    fn system_job_tables_should_exist_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let jobs_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='system_jobs'",
                [],
                |row| row.get(0),
            )
            .expect("query system_jobs");

        assert_eq!(jobs_exists, 1, "system_jobs table should exist");
    }

    #[test]
    fn system_jobs_table_should_exist_after_migrations() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_all_migrations(&conn);

        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='system_jobs'",
                [],
                |row| row.get(0),
            )
            .expect("query system_jobs");

        assert_eq!(exists, 1, "system_jobs table should exist");
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
