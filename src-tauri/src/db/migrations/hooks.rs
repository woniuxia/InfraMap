use crate::db::schema::{ensure_taxonomy_schema_consistency, migrate_taxonomy_v2};
use crate::error::{log_runtime_warning, AppError, AppResult};

pub fn run_post_migration_hook(
    command: &str,
    conn: &rusqlite::Connection,
    version: i32,
) -> AppResult<()> {
    match version {
        12 => migrate_taxonomy_v2(conn).map_err(|error| {
            AppError::from_db_error(command, &format!("执行迁移后置钩子 v{version}"), error)
        }),
        _ => Ok(()),
    }
}

pub fn run_post_migration_repairs(command: &str, conn: &rusqlite::Connection) {
    if let Err(error) = ensure_taxonomy_schema_consistency(conn) {
        log_runtime_warning(command, "迁移后的 taxonomy 一致性修复失败", error);
    }
    if let Err(error) = ensure_contacts_schema_consistency(conn) {
        log_runtime_warning(command, "迁移后的 contacts 表一致性修复失败", error);
    }
}

/// 确保 contacts 表有 company 列
fn ensure_contacts_schema_consistency(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    // 检查 contacts 表是否存在
    let table_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='contacts'",
        [],
        |row| row.get(0),
    )?;

    if table_exists == 0 {
        return Ok(()); // 表不存在，迁移会创建它
    }

    // 检查 company 列是否存在
    let mut stmt = conn.prepare("PRAGMA table_info(contacts)")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;

    let mut has_company = false;
    for row in rows {
        if row? == "company" {
            has_company = true;
            break;
        }
    }

    if !has_company {
        // 添加 company 列
        conn.execute("ALTER TABLE contacts ADD COLUMN company TEXT", [])?;
    }

    // 确保索引存在
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_contacts_company ON contacts(company) WHERE is_deleted = 0;"
    )?;

    Ok(())
}
