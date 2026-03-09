use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::audit::insert_audit_log;
use crate::db::crud::{build_where_clause, count_query};
use crate::db::transaction::with_transaction;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::call_relation::CallRelation;
use crate::models::common::{PagedResult, QueryParams};
use crate::validation::{validate_enum, validate_required};

fn row_to_call_relation(row: &rusqlite::Row) -> rusqlite::Result<CallRelation> {
    Ok(CallRelation {
        id: row.get(0)?,
        pair_key: row.get(1)?,
        owner_id: row.get(2)?,
        owner_type: row.get(3)?,
        peer_id: row.get(4)?,
        peer_type: row.get(5)?,
        direction: row.get(6)?,
        relation_type: row.get(7)?,
        description: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

const SELECT_COLUMNS: &str =
    "id, pair_key, owner_id, owner_type, peer_id, peer_type, direction, relation_type, \
     description, created_at, updated_at";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceCallRelationItem {
    pub peer_id: String,
    pub peer_type: String,
    pub direction: String,
    pub relation_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceResourceCallRelationsParams {
    pub resource_id: String,
    pub resource_type: String,
    pub items: Vec<ReplaceCallRelationItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceResourceCallRelationsResult {
    pub created_count: u64,
    pub deleted_count: u64,
    pub deduplicated_count: u64,
}

fn inverse_direction(direction: &str) -> &'static str {
    if direction == "upstream" {
        "downstream"
    } else {
        "upstream"
    }
}

fn normalize_relation_type(relation_type: &str) -> String {
    relation_type.trim().to_string()
}

fn normalize_description(description: &Option<String>) -> Option<String> {
    description
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn build_pair_key(
    owner_id: &str,
    owner_type: &str,
    peer_id: &str,
    peer_type: &str,
    relation_type: &str,
) -> String {
    let left = format!("{}:{}", owner_type.trim(), owner_id.trim());
    let right = format!("{}:{}", peer_type.trim(), peer_id.trim());
    if left <= right {
        format!("{}|{}|{}", left, right, relation_type.trim())
    } else {
        format!("{}|{}|{}", right, left, relation_type.trim())
    }
}

fn validate_replace_item(command: &str, item: &ReplaceCallRelationItem) -> AppResult<()> {
    validate_required(&item.peer_id, "peer_id").map_err(|e| AppError::validation(command, e))?;
    validate_enum(
        &item.peer_type,
        &["application", "middleware", "nginx"],
        "peer_type",
    )
    .map_err(|e| AppError::validation(command, e))?;
    validate_enum(&item.direction, &["upstream", "downstream"], "direction")
        .map_err(|e| AppError::validation(command, e))?;
    validate_enum(
        &item.relation_type,
        &[
            "http_call",
            "tcp",
            "mq_produce",
            "mq_consume",
            "grpc_call",
            "db_query",
            "cache_access",
        ],
        "relation_type",
    )
    .map_err(|e| AppError::validation(command, e))?;
    Ok(())
}

fn normalize_items(
    command: &str,
    items: &[ReplaceCallRelationItem],
) -> AppResult<(Vec<ReplaceCallRelationItem>, u64)> {
    let mut deduplicated = Vec::new();
    let mut unique_keys: HashSet<String> = HashSet::new();
    let mut deduplicated_count: u64 = 0;

    for item in items {
        validate_replace_item(command, item)?;

        let normalized_item = ReplaceCallRelationItem {
            peer_id: item.peer_id.trim().to_string(),
            peer_type: item.peer_type.trim().to_string(),
            direction: item.direction.trim().to_string(),
            relation_type: normalize_relation_type(&item.relation_type),
            description: normalize_description(&item.description),
        };

        let unique_key = format!(
            "{}|{}|{}|{}",
            normalized_item.peer_id,
            normalized_item.peer_type,
            normalized_item.direction,
            normalized_item.relation_type
        );

        if unique_keys.insert(unique_key) {
            deduplicated.push(normalized_item);
        } else {
            deduplicated_count += 1;
        }
    }

    Ok((deduplicated, deduplicated_count))
}

fn query_owner_pair_keys(
    command: &str,
    conn: &rusqlite::Connection,
    resource_id: &str,
    resource_type: &str,
) -> AppResult<Vec<String>> {
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT pair_key
             FROM call_relations
             WHERE owner_id = ?1 AND owner_type = ?2 AND is_deleted = 0",
        )
        .map_err(|e| AppError::from_db_error(command, "查询历史关系键", e))?;
    let rows = stmt
        .query_map(rusqlite::params![resource_id, resource_type], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|e| AppError::from_db_error(command, "读取历史关系键", e))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::from_db_error(command, "读取历史关系键", e))
}

fn resource_exists(
    conn: &rusqlite::Connection,
    resource_type: &str,
    resource_id: &str,
) -> Result<bool, rusqlite::Error> {
    let count: i64 = match resource_type {
        "application" => conn.query_row(
            "SELECT COUNT(*) FROM applications WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![resource_id],
            |row| row.get(0),
        )?,
        "middleware" => conn.query_row(
            "SELECT COUNT(*) FROM middlewares WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![resource_id],
            |row| row.get(0),
        )?,
        "nginx" => conn.query_row(
            "SELECT COUNT(*) FROM nginx_configs WHERE id = ?1 AND is_deleted = 0",
            rusqlite::params![resource_id],
            |row| row.get(0),
        )?,
        _ => 0,
    };
    Ok(count > 0)
}

fn ensure_resource_exists(
    command: &str,
    conn: &rusqlite::Connection,
    resource_type: &str,
    resource_id: &str,
    role: &str,
) -> AppResult<()> {
    if resource_exists(conn, resource_type, resource_id)
        .map_err(|e| AppError::from_db_error(command, "校验资源存在性", e))?
    {
        return Ok(());
    }

    Err(AppError::not_found(
        command,
        format!("{}资源不存在或已删除。", role),
        Some(format!(
            "resource_type={}, resource_id={}",
            resource_type, resource_id
        )),
    ))
}

fn delete_pairs_by_keys(
    command: &str,
    conn: &rusqlite::Connection,
    pair_keys: &[String],
    _now: &str,
) -> AppResult<u64> {
    if pair_keys.is_empty() {
        return Ok(0);
    }

    let placeholders = vec!["?"; pair_keys.len()].join(", ");
    let sql = format!(
        "DELETE FROM call_relations
         WHERE is_deleted = 0 AND pair_key IN ({})",
        placeholders
    );

    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    for pair_key in pair_keys {
        params.push(Box::new(pair_key.clone()));
    }

    let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|v| v.as_ref()).collect();
    let affected = conn
        .execute(&sql, refs.as_slice())
        .map_err(|e| AppError::from_db_error(command, "清理历史调用关系", e))?;
    Ok(affected as u64)
}

fn insert_call_relation_row(
    command: &str,
    conn: &rusqlite::Connection,
    pair_key: &str,
    owner_id: &str,
    owner_type: &str,
    peer_id: &str,
    peer_type: &str,
    direction: &str,
    relation_type: &str,
    description: &Option<String>,
    now: &str,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO call_relations (
            id, pair_key, owner_id, owner_type, peer_id, peer_type, direction, relation_type,
            description, is_deleted, deleted_at, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, NULL, ?10, ?10)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            pair_key,
            owner_id,
            owner_type,
            peer_id,
            peer_type,
            direction,
            relation_type,
            description,
            now
        ],
    )
    .map_err(|e| AppError::from_db_error(command, "写入调用关系", e))?;
    Ok(())
}

fn replace_resource_call_relations_inner(
    command: &str,
    conn: &rusqlite::Connection,
    params: ReplaceResourceCallRelationsParams,
) -> AppResult<ReplaceResourceCallRelationsResult> {
    validate_required(&params.resource_id, "resource_id")
        .map_err(|e| AppError::validation(command, e))?;
    validate_enum(
        &params.resource_type,
        &["application", "middleware", "nginx"],
        "resource_type",
    )
    .map_err(|e| AppError::validation(command, e))?;

    let (normalized_items, deduplicated_count) = normalize_items(command, &params.items)?;
    ensure_resource_exists(
        command,
        conn,
        &params.resource_type,
        &params.resource_id,
        "owner",
    )?;
    for item in &normalized_items {
        ensure_resource_exists(command, conn, &item.peer_type, &item.peer_id, "peer")?;
    }
    let now = chrono::Utc::now().to_rfc3339();

    with_transaction(conn, command, |conn| {
        let pair_keys =
            query_owner_pair_keys(command, conn, &params.resource_id, &params.resource_type)?;
        let deleted_count = delete_pairs_by_keys(command, conn, &pair_keys, &now)?;

        let mut created_count: u64 = 0;
        for item in &normalized_items {
            let pair_key = build_pair_key(
                &params.resource_id,
                &params.resource_type,
                &item.peer_id,
                &item.peer_type,
                &item.relation_type,
            );
            insert_call_relation_row(
                command,
                conn,
                &pair_key,
                &params.resource_id,
                &params.resource_type,
                &item.peer_id,
                &item.peer_type,
                &item.direction,
                &item.relation_type,
                &item.description,
                &now,
            )?;
            insert_call_relation_row(
                command,
                conn,
                &pair_key,
                &item.peer_id,
                &item.peer_type,
                &params.resource_id,
                &params.resource_type,
                inverse_direction(&item.direction),
                &item.relation_type,
                &item.description,
                &now,
            )?;
            created_count += 2;
        }

        insert_audit_log(
            conn,
            "update",
            "call_relation",
            &params.resource_id,
            None,
            None,
        )
        .map_err(|e| AppError::from_db_error(command, "写入审计日志", e))?;

        Ok(ReplaceResourceCallRelationsResult {
            created_count,
            deleted_count,
            deduplicated_count,
        })
    })
}

#[tauri::command]
pub fn list_call_relations(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<CallRelation>> {
    let command = "list_call_relations";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;

    let search_columns: &[&str] = &[];
    let filter_columns = &[
        "owner_id",
        "owner_type",
        "peer_id",
        "peer_type",
        "direction",
        "relation_type",
        "pair_key",
    ];
    let (where_clause, sql_params) = build_where_clause(&params, search_columns, filter_columns);

    let total = count_query(
        command,
        "查询调用关系数量",
        &conn,
        "call_relations",
        &where_clause,
        &sql_params,
    )?;

    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let sql = format!(
        "SELECT {} FROM call_relations {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        SELECT_COLUMNS, where_clause
    );

    let mut all_params = sql_params;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));

    let refs: Vec<&dyn rusqlite::types::ToSql> = all_params.iter().map(|v| v.as_ref()).collect();
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::from_db_error(command, "查询调用关系列表", e))?;
    let rows = stmt
        .query_map(refs.as_slice(), row_to_call_relation)
        .map_err(|e| AppError::from_db_error(command, "读取调用关系列表", e))?;

    let data: Vec<CallRelation> = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::from_db_error(command, "读取调用关系列表", e))?;

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn replace_resource_call_relations(
    pool: State<DbPool>,
    params: ReplaceResourceCallRelationsParams,
) -> AppResult<ReplaceResourceCallRelationsResult> {
    let command = "replace_resource_call_relations";
    let conn = pool
        .get()
        .map_err(|e| AppError::db_unavailable(command, format!("Pool error: {}", e)))?;
    replace_resource_call_relations_inner(command, &conn, params)
}

#[cfg(test)]
mod tests {
    use super::{
        replace_resource_call_relations_inner, ReplaceCallRelationItem,
        ReplaceResourceCallRelationsParams,
    };
    use crate::error::AppErrorCode;
    use crate::test_helpers::{insert_test_application, setup_test_db};

    fn base_params() -> ReplaceResourceCallRelationsParams {
        ReplaceResourceCallRelationsParams {
            resource_id: "app-a".into(),
            resource_type: "application".into(),
            items: vec![ReplaceCallRelationItem {
                peer_id: "app-b".into(),
                peer_type: "application".into(),
                direction: "upstream".into(),
                relation_type: "http_call".into(),
                description: Some("A calls B".into()),
            }],
        }
    }

    #[test]
    fn replace_resource_call_relations_should_create_bidirectional_rows() {
        let conn = setup_test_db();
        insert_test_application(&conn, "app-a", "App-A", "prod");
        insert_test_application(&conn, "app-b", "App-B", "prod");

        let result = replace_resource_call_relations_inner("test", &conn, base_params())
            .expect("replace call relations should succeed");
        assert_eq!(result.created_count, 2);
        assert_eq!(result.deleted_count, 0);
        assert_eq!(result.deduplicated_count, 0);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM call_relations
                 WHERE is_deleted = 0
                   AND pair_key IN (
                     SELECT pair_key FROM call_relations
                     WHERE owner_id='app-a' AND peer_id='app-b' AND relation_type='http_call' AND direction='upstream' AND is_deleted=0
                   )",
                [],
                |row| row.get(0),
            )
            .expect("count pair rows");
        assert_eq!(count, 2);
    }

    #[test]
    fn replace_resource_call_relations_should_replace_existing_pairs_for_owner() {
        let conn = setup_test_db();
        insert_test_application(&conn, "app-a", "App-A", "prod");
        insert_test_application(&conn, "app-b", "App-B", "prod");
        insert_test_application(&conn, "app-c", "App-C", "prod");

        replace_resource_call_relations_inner("test", &conn, base_params())
            .expect("seed first relations");

        let result = replace_resource_call_relations_inner(
            "test",
            &conn,
            ReplaceResourceCallRelationsParams {
                resource_id: "app-a".into(),
                resource_type: "application".into(),
                items: vec![ReplaceCallRelationItem {
                    peer_id: "app-c".into(),
                    peer_type: "application".into(),
                    direction: "downstream".into(),
                    relation_type: "tcp".into(),
                    description: None,
                }],
            },
        )
        .expect("replace second relations");

        assert_eq!(result.created_count, 2);
        assert!(result.deleted_count >= 2);

        let old_active: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM call_relations
                 WHERE owner_id='app-a' AND peer_id='app-b' AND is_deleted=0",
                [],
                |row| row.get(0),
            )
            .expect("count old active rows");
        assert_eq!(old_active, 0);

        let new_active: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM call_relations
                 WHERE owner_id='app-a' AND peer_id='app-c' AND direction='downstream' AND relation_type='tcp' AND is_deleted=0",
                [],
                |row| row.get(0),
            )
            .expect("count new active rows");
        assert_eq!(new_active, 1);
    }

    #[test]
    fn replace_resource_call_relations_should_allow_multiple_relation_types_between_same_pair() {
        let conn = setup_test_db();
        insert_test_application(&conn, "app-a", "App-A", "prod");
        insert_test_application(&conn, "app-b", "App-B", "prod");

        let result = replace_resource_call_relations_inner(
            "test",
            &conn,
            ReplaceResourceCallRelationsParams {
                resource_id: "app-a".into(),
                resource_type: "application".into(),
                items: vec![
                    ReplaceCallRelationItem {
                        peer_id: "app-b".into(),
                        peer_type: "application".into(),
                        direction: "upstream".into(),
                        relation_type: "http_call".into(),
                        description: None,
                    },
                    ReplaceCallRelationItem {
                        peer_id: "app-b".into(),
                        peer_type: "application".into(),
                        direction: "upstream".into(),
                        relation_type: "grpc_call".into(),
                        description: None,
                    },
                ],
            },
        )
        .expect("replace should succeed");

        assert_eq!(result.created_count, 4);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM call_relations
                 WHERE owner_id='app-a' AND peer_id='app-b' AND direction='upstream' AND is_deleted=0",
                [],
                |row| row.get(0),
            )
            .expect("count relation rows");
        assert_eq!(count, 2);
    }

    #[test]
    fn replace_resource_call_relations_should_deduplicate_duplicate_rows() {
        let conn = setup_test_db();
        insert_test_application(&conn, "app-a", "App-A", "prod");
        insert_test_application(&conn, "app-b", "App-B", "prod");

        let result = replace_resource_call_relations_inner(
            "test",
            &conn,
            ReplaceResourceCallRelationsParams {
                resource_id: "app-a".into(),
                resource_type: "application".into(),
                items: vec![
                    ReplaceCallRelationItem {
                        peer_id: "app-b".into(),
                        peer_type: "application".into(),
                        direction: "upstream".into(),
                        relation_type: "http_call".into(),
                        description: None,
                    },
                    ReplaceCallRelationItem {
                        peer_id: "app-b".into(),
                        peer_type: "application".into(),
                        direction: "upstream".into(),
                        relation_type: "http_call".into(),
                        description: Some("dup".into()),
                    },
                ],
            },
        )
        .expect("replace should succeed");

        assert_eq!(result.created_count, 2);
        assert_eq!(result.deduplicated_count, 1);
    }

    #[test]
    fn replace_resource_call_relations_should_reject_missing_owner_resource() {
        let conn = setup_test_db();
        insert_test_application(&conn, "app-b", "App-B", "prod");

        let err = replace_resource_call_relations_inner("test", &conn, base_params())
            .expect_err("missing owner resource should fail");
        assert_eq!(err.code, AppErrorCode::NotFound);
    }

    #[test]
    fn replace_resource_call_relations_should_reject_missing_peer_resource() {
        let conn = setup_test_db();
        insert_test_application(&conn, "app-a", "App-A", "prod");

        let err = replace_resource_call_relations_inner("test", &conn, base_params())
            .expect_err("missing peer resource should fail");
        assert_eq!(err.code, AppErrorCode::NotFound);
    }
}
