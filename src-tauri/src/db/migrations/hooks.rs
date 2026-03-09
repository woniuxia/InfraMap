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
}
