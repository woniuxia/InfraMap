use tauri::State;

use crate::db::audit::insert_audit_log;
use crate::db::crud::{build_where_clause, count_query, soft_delete};
use crate::db::DbPool;
use crate::error::{AppError, AppErrorCode, AppResult};
use crate::models::common::{PagedResult, QueryParams};
use crate::models::dependency::Dependency;
use crate::validation::{validate_dependency, validate_enum, validate_required};
use serde::{Deserialize, Serialize};

fn row_to_dependency(row: &rusqlite::Row) -> rusqlite::Result<Dependency> {
    Ok(Dependency {
        id: row.get(0)?,
        source_id: row.get(1)?,
        source_type: row.get(2)?,
        target_id: row.get(3)?,
        target_type: row.get(4)?,
        relation_type: row.get(5)?,
        description: row.get(6)?,
        is_deleted: row.get(7)?,
        deleted_at: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

const SELECT_COLUMNS: &str = "id, source_id, source_type, target_id, target_type, relation_type, \
     description, is_deleted, deleted_at, created_at, updated_at";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDependencyBatchItem {
    pub target_id: String,
    pub target_type: String,
    pub relation_type: String,
    pub direction: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDependenciesBatchParams {
    pub resource_id: String,
    pub resource_type: String,
    pub items: Vec<SaveDependencyBatchItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDependenciesBatchResult {
    pub created_count: u64,
    pub skipped_count: u64,
}

fn expand_batch_item(
    command: &str,
    resource_id: &str,
    resource_type: &str,
    item: &SaveDependencyBatchItem,
) -> AppResult<Vec<Dependency>> {
    validate_required(&item.target_id, "target_id")
        .map_err(|e| AppError::validation(command, e))?;
    validate_enum(
        &item.target_type,
        &["application", "middleware", "nginx"],
        "target_type",
    )
    .map_err(|e| AppError::validation(command, e))?;
    validate_enum(
        &item.direction,
        &["upstream", "downstream", "bidirectional"],
        "direction",
    )
    .map_err(|e| AppError::validation(command, e))?;

    let mut dependencies = Vec::new();
    if item.direction == "downstream" || item.direction == "bidirectional" {
        dependencies.push(Dependency {
            id: "".into(),
            source_id: resource_id.to_string(),
            source_type: resource_type.to_string(),
            target_id: item.target_id.clone(),
            target_type: item.target_type.clone(),
            relation_type: item.relation_type.clone(),
            description: item.description.clone(),
            is_deleted: 0,
            deleted_at: None,
            created_at: "".into(),
            updated_at: "".into(),
        });
    }
    if item.direction == "upstream" || item.direction == "bidirectional" {
        dependencies.push(Dependency {
            id: "".into(),
            source_id: item.target_id.clone(),
            source_type: item.target_type.clone(),
            target_id: resource_id.to_string(),
            target_type: resource_type.to_string(),
            relation_type: item.relation_type.clone(),
            description: item.description.clone(),
            is_deleted: 0,
            deleted_at: None,
            created_at: "".into(),
            updated_at: "".into(),
        });
    }
    Ok(dependencies)
}

fn save_dependency_inner(
    command: &str,
    conn: &rusqlite::Connection,
    data: Dependency,
) -> AppResult<()> {
    validate_dependency(&data).map_err(|e| AppError::validation(command, e))?;

    let now = chrono::Utc::now().to_rfc3339();
    let is_new = data.id.is_empty() || {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM dependencies WHERE id = ?1 AND is_deleted = 0",
                rusqlite::params![data.id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        count == 0
    };

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    let result: AppResult<()> = (|| {
        if is_new {
            let id = if data.id.is_empty() {
                uuid::Uuid::new_v4().to_string()
            } else {
                data.id.clone()
            };
            conn.execute(
                "INSERT INTO dependencies (id, source_id, source_type, target_id, target_type, relation_type,
                                           description, is_deleted, deleted_at, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,0,NULL,?8,?8)",
                rusqlite::params![
                    id,
                    data.source_id,
                    data.source_type,
                    data.target_id,
                    data.target_type,
                    data.relation_type,
                    data.description,
                    now
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "创建依赖关系", e))?;
            insert_audit_log(conn, "create", "dependency", &id, None, None)
                .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
        } else {
            conn.execute(
                "UPDATE dependencies SET source_id=?1, source_type=?2, target_id=?3, target_type=?4,
                                         relation_type=?5, description=?6, updated_at=?7
                 WHERE id=?8 AND is_deleted=0",
                rusqlite::params![
                    data.source_id,
                    data.source_type,
                    data.target_id,
                    data.target_type,
                    data.relation_type,
                    data.description,
                    now,
                    data.id
                ],
            )
            .map_err(|e| AppError::from_db_error(command, "更新依赖关系", e))?;
            insert_audit_log(conn, "update", "dependency", &data.id, None, None)
                .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;
        }
        Ok(())
    })();

    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT;")
                .map_err(|e| AppError::from_db_error(command, "提交事务", e))?;
            Ok(())
        }
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

fn save_dependencies_batch_inner(
    command: &str,
    conn: &rusqlite::Connection,
    params: SaveDependenciesBatchParams,
) -> AppResult<SaveDependenciesBatchResult> {
    validate_required(&params.resource_id, "resource_id")
        .map_err(|e| AppError::validation(command, e))?;
    validate_enum(
        &params.resource_type,
        &["application", "middleware", "nginx"],
        "resource_type",
    )
    .map_err(|e| AppError::validation(command, e))?;
    if params.items.is_empty() {
        return Err(AppError::validation(command, "items is required"));
    }

    let mut created_count: u64 = 0;
    let mut skipped_count: u64 = 0;
    for item in &params.items {
        let expanded =
            expand_batch_item(command, &params.resource_id, &params.resource_type, item)?;
        for dependency in expanded {
            match save_dependency_inner(command, conn, dependency) {
                Ok(()) => created_count += 1,
                Err(error) if error.code == AppErrorCode::Conflict => skipped_count += 1,
                Err(error) => return Err(error),
            }
        }
    }

    Ok(SaveDependenciesBatchResult {
        created_count,
        skipped_count,
    })
}

#[tauri::command]
pub fn list_dependencies(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<Dependency>> {
    let command = "list_dependencies";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let search_columns: &[&str] = &[];
    let filter_columns = &[
        "source_id",
        "source_type",
        "target_id",
        "target_type",
        "relation_type",
    ];
    let (where_clause, sql_params) = build_where_clause(&params, search_columns, filter_columns);

    let total = count_query(&conn, "dependencies", &where_clause, &sql_params)
        .map_err(|e| AppError::from_db_error(command, "查询依赖关系数量", e))?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let sql = format!(
        "SELECT {} FROM dependencies {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );

    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        all_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询依赖关系列表", e))?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_dependency)
        .map_err(|e| AppError::from_db_error(command, "读取依赖关系列表", e))?;

    let data: Vec<Dependency> = rows.filter_map(|r| r.ok()).collect();

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn save_dependency(pool: State<DbPool>, data: Dependency) -> AppResult<()> {
    let command = "save_dependency";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    save_dependency_inner(command, &conn, data)
}

#[tauri::command]
pub fn save_dependencies_batch(
    pool: State<DbPool>,
    params: SaveDependenciesBatchParams,
) -> AppResult<SaveDependenciesBatchResult> {
    let command = "save_dependencies_batch";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    save_dependencies_batch_inner(command, &conn, params)
}

#[tauri::command]
pub fn soft_delete_dependency(pool: State<DbPool>, id: String) -> AppResult<()> {
    let command = "soft_delete_dependency";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| AppError::from_db_error(command, "开启事务", e))?;

    match soft_delete(&conn, "dependencies", &id) {
        Ok(()) => match insert_audit_log(&conn, "delete", "dependency", &id, None, None) {
            Ok(()) => {
                conn.execute_batch("COMMIT;")
                    .map_err(|e| AppError::from_db_error(command, "提交事务", e))?;
                Ok(())
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK;");
                Err(AppError::from_db_error(command, "写入审计日志", e))
            }
        },
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(AppError::from_db_error(command, "删除依赖关系", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        save_dependencies_batch_inner, SaveDependenciesBatchParams, SaveDependencyBatchItem,
    };
    use crate::error::AppErrorCode;
    use crate::test_helpers::{
        insert_test_application, insert_test_dependency, insert_test_middleware, setup_test_db,
    };

    fn make_batch_params(direction: &str) -> SaveDependenciesBatchParams {
        SaveDependenciesBatchParams {
            resource_id: "app-a".into(),
            resource_type: "application".into(),
            items: vec![SaveDependencyBatchItem {
                target_id: "app-b".into(),
                target_type: "application".into(),
                relation_type: "http_call".into(),
                direction: direction.into(),
                description: Some("test relation".into()),
            }],
        }
    }

    #[test]
    fn save_dependencies_batch_should_create_downstream_edges() {
        let conn = setup_test_db();
        insert_test_application(&conn, "app-a", "App-A", "prod");
        insert_test_application(&conn, "app-b", "App-B", "prod");

        let result = save_dependencies_batch_inner("test", &conn, make_batch_params("downstream"))
            .expect("batch save should succeed");
        assert_eq!(result.created_count, 1);
        assert_eq!(result.skipped_count, 0);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM dependencies
                 WHERE source_id='app-a' AND target_id='app-b' AND relation_type='http_call' AND is_deleted=0",
                [],
                |row| row.get(0),
            )
            .expect("query dependency count");
        assert_eq!(count, 1);
    }

    #[test]
    fn save_dependencies_batch_should_create_upstream_edges() {
        let conn = setup_test_db();
        insert_test_application(&conn, "app-a", "App-A", "prod");
        insert_test_application(&conn, "app-b", "App-B", "prod");

        let result = save_dependencies_batch_inner("test", &conn, make_batch_params("upstream"))
            .expect("batch save should succeed");
        assert_eq!(result.created_count, 1);
        assert_eq!(result.skipped_count, 0);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM dependencies
                 WHERE source_id='app-b' AND target_id='app-a' AND relation_type='http_call' AND is_deleted=0",
                [],
                |row| row.get(0),
            )
            .expect("query dependency count");
        assert_eq!(count, 1);
    }

    #[test]
    fn save_dependencies_batch_should_create_bidirectional_edges() {
        let conn = setup_test_db();
        insert_test_application(&conn, "app-a", "App-A", "prod");
        insert_test_middleware(&conn, "mw-1", "redis-main", "cache");

        let result = save_dependencies_batch_inner(
            "test",
            &conn,
            SaveDependenciesBatchParams {
                resource_id: "app-a".into(),
                resource_type: "application".into(),
                items: vec![SaveDependencyBatchItem {
                    target_id: "mw-1".into(),
                    target_type: "middleware".into(),
                    relation_type: "tcp".into(),
                    direction: "bidirectional".into(),
                    description: None,
                }],
            },
        )
        .expect("batch save should succeed");
        assert_eq!(result.created_count, 2);
        assert_eq!(result.skipped_count, 0);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM dependencies
                 WHERE (
                   source_id='app-a' AND source_type='application' AND target_id='mw-1' AND target_type='middleware'
                 ) OR (
                   source_id='mw-1' AND source_type='middleware' AND target_id='app-a' AND target_type='application'
                 )",
                [],
                |row| row.get(0),
            )
            .expect("query dependency count");
        assert_eq!(count, 2);
    }

    #[test]
    fn save_dependencies_batch_should_skip_duplicates() {
        let conn = setup_test_db();
        insert_test_application(&conn, "app-a", "App-A", "prod");
        insert_test_application(&conn, "app-b", "App-B", "prod");
        insert_test_dependency(
            &conn,
            "dep-existing",
            "app-a",
            "application",
            "app-b",
            "application",
            "http_call",
        );

        let result = save_dependencies_batch_inner("test", &conn, make_batch_params("downstream"))
            .expect("batch save should succeed");
        assert_eq!(result.created_count, 0);
        assert_eq!(result.skipped_count, 1);
    }

    #[test]
    fn save_dependencies_batch_should_reject_invalid_direction() {
        let conn = setup_test_db();
        insert_test_application(&conn, "app-a", "App-A", "prod");
        insert_test_application(&conn, "app-b", "App-B", "prod");

        let err = save_dependencies_batch_inner(
            "test",
            &conn,
            SaveDependenciesBatchParams {
                resource_id: "app-a".into(),
                resource_type: "application".into(),
                items: vec![SaveDependencyBatchItem {
                    target_id: "app-b".into(),
                    target_type: "application".into(),
                    relation_type: "http_call".into(),
                    direction: "invalid".into(),
                    description: None,
                }],
            },
        )
        .expect_err("invalid direction should fail");
        assert_eq!(err.code, AppErrorCode::ValidationError);
    }
}
