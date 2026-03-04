use std::collections::HashMap;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::audit::insert_audit_log;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::common::{PagedResult, QueryParams};

const ISSUE_ERROR: &str = "error";
const ISSUE_CONFLICT: &str = "conflict";
const ISSUE_WARNING: &str = "warning";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDraftRow {
    pub resource_type: String,
    pub name: Option<String>,
    pub env: Option<String>,
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    pub status: Option<String>,
    pub address: Option<String>,
    pub port: Option<i64>,
    pub category: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportValidationIssue {
    pub row_no: u32,
    pub field_key: Option<String>,
    pub issue_type: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportNormalizedRow {
    pub row_no: u32,
    pub resource_type: String,
    pub name: String,
    pub env: String,
    pub item_type: Option<String>,
    pub status: Option<String>,
    pub address: Option<String>,
    pub port: Option<i64>,
    pub description: Option<String>,
    pub conflict_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreviewResult {
    pub rows: Vec<ImportNormalizedRow>,
    pub issues: Vec<ImportValidationIssue>,
    pub error_count: u32,
    pub warning_count: u32,
    pub conflict_count: u32,
    pub valid_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewImportRowsInput {
    pub rows: Vec<ImportDraftRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteImportRowsInput {
    pub rows: Vec<ImportDraftRow>,
    pub strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportExecutionResult {
    pub job_id: String,
    pub status: String,
    pub total_rows: u32,
    pub created_count: u32,
    pub updated_count: u32,
    pub skipped_count: u32,
    pub failed_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportJobSummary {
    pub id: String,
    pub status: String,
    pub strategy: String,
    pub total_rows: u32,
    pub created_count: u32,
    pub updated_count: u32,
    pub skipped_count: u32,
    pub failed_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportJobRowResult {
    pub row_no: u32,
    pub resource_type: String,
    pub name: String,
    pub env: String,
    pub status: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportJobDetail {
    pub summary: ImportJobSummary,
    pub rows: Vec<ImportJobRowResult>,
    pub issues: Vec<ImportValidationIssue>,
}

#[derive(Debug, Clone, Copy)]
enum ExecutionStrategy {
    Skip,
    Overwrite,
    Block,
}

impl ExecutionStrategy {
    fn parse(raw: Option<&str>) -> Result<Self, String> {
        match raw.unwrap_or("skip").trim().to_lowercase().as_str() {
            "skip" => Ok(Self::Skip),
            "overwrite" => Ok(Self::Overwrite),
            "block" => Ok(Self::Block),
            _ => Err("strategy must be one of: skip, overwrite, block".to_string()),
        }
    }
}

fn push_issue(
    issues: &mut Vec<ImportValidationIssue>,
    row_no: u32,
    field_key: Option<&str>,
    issue_type: &str,
    code: &str,
    message: impl Into<String>,
) {
    issues.push(ImportValidationIssue {
        row_no,
        field_key: field_key.map(str::to_string),
        issue_type: issue_type.to_string(),
        code: code.to_string(),
        message: message.into(),
    });
}

fn normalize_env(raw: Option<&str>) -> String {
    match raw.unwrap_or("prod").trim() {
        "prod" => "prod".to_string(),
        "dev" => "dev".to_string(),
        "test" => "test".to_string(),
        _ => "prod".to_string(),
    }
}

fn normalize_status(raw: Option<&str>) -> String {
    match raw.unwrap_or("running").trim() {
        "running" => "running".to_string(),
        "stopped" => "stopped".to_string(),
        "maintenance" => "maintenance".to_string(),
        _ => "running".to_string(),
    }
}

fn normalize_app_type(raw: Option<&str>) -> String {
    match raw.unwrap_or("backend").trim() {
        "frontend" => "frontend".to_string(),
        "backend" => "backend".to_string(),
        "gateway" => "gateway".to_string(),
        "batch_job" => "batch_job".to_string(),
        "microservice" => "microservice".to_string(),
        "other" => "other".to_string(),
        _ => "backend".to_string(),
    }
}

fn normalize_text(raw: Option<&str>) -> Option<String> {
    raw.and_then(|item| {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn validate_and_normalize(
    conn: &Connection,
    row_no: u32,
    row: &ImportDraftRow,
) -> Result<(ImportNormalizedRow, Vec<ImportValidationIssue>), String> {
    let mut issues: Vec<ImportValidationIssue> = Vec::new();
    let resource_type = row.resource_type.trim().to_lowercase();
    let name = normalize_text(row.name.as_deref()).unwrap_or_default();
    let env = normalize_env(row.env.as_deref());
    let app_type = normalize_app_type(row.item_type.as_deref());
    let status = normalize_status(row.status.as_deref());

    if resource_type != "application" {
        push_issue(
            &mut issues,
            row_no,
            Some("resource_type"),
            ISSUE_ERROR,
            "unsupported_resource_type",
            "当前版本仅支持 application 批量录入",
        );
    }

    if name.is_empty() {
        push_issue(
            &mut issues,
            row_no,
            Some("name"),
            ISSUE_ERROR,
            "name_required",
            "name 不能为空",
        );
    }

    if row
        .env
        .as_deref()
        .map(|item| !matches!(item.trim(), "prod" | "dev" | "test"))
        .unwrap_or(false)
    {
        push_issue(
            &mut issues,
            row_no,
            Some("env"),
            ISSUE_ERROR,
            "invalid_env",
            "env 仅支持 prod/dev/test",
        );
    }

    if let Some(port) = row.port {
        if !(1..=65535).contains(&port) {
            push_issue(
                &mut issues,
                row_no,
                Some("port"),
                ISSUE_ERROR,
                "invalid_port",
                "port 必须在 1..65535 之间",
            );
        }
    }

    if row
        .address
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
    {
        push_issue(
            &mut issues,
            row_no,
            Some("address"),
            ISSUE_WARNING,
            "empty_address",
            "address 为空，将以空值写入",
        );
    }

    let mut conflict_id: Option<String> = None;
    let has_error = issues.iter().any(|item| item.issue_type == ISSUE_ERROR);
    if !has_error && !name.is_empty() {
        conflict_id = conn
            .query_row(
                "SELECT id FROM applications
                 WHERE name = ?1 AND env = ?2 AND type = ?3 AND is_deleted = 0
                 LIMIT 1",
                rusqlite::params![name, env, app_type],
                |raw| raw.get(0),
            )
            .ok();
        if conflict_id.is_some() {
            push_issue(
                &mut issues,
                row_no,
                Some("name"),
                ISSUE_CONFLICT,
                "application_conflict",
                "应用名称+环境+类型组合已存在",
            );
        }
    }

    Ok((
        ImportNormalizedRow {
            row_no,
            resource_type,
            name,
            env,
            item_type: Some(app_type),
            status: Some(status),
            address: normalize_text(row.address.as_deref()),
            port: row.port,
            description: normalize_text(row.description.as_deref()),
            conflict_id,
        },
        issues,
    ))
}

fn preview_import_rows_inner(
    _command: &str,
    conn: &Connection,
    input: PreviewImportRowsInput,
) -> AppResult<ImportPreviewResult> {
    let mut rows: Vec<ImportNormalizedRow> = Vec::new();
    let mut issues: Vec<ImportValidationIssue> = Vec::new();
    let mut error_count = 0_u32;
    let mut warning_count = 0_u32;
    let mut conflict_count = 0_u32;
    let mut valid_count = 0_u32;

    for (index, draft) in input.rows.iter().enumerate() {
        let row_no = (index + 1) as u32;
        let (normalized, row_issues) = validate_and_normalize(conn, row_no, draft)
            .map_err(|err| AppError::internal("preview_import_rows", err))?;
        let row_has_error = row_issues.iter().any(|item| item.issue_type == ISSUE_ERROR);
        let row_has_conflict = row_issues
            .iter()
            .any(|item| item.issue_type == ISSUE_CONFLICT);
        if !row_has_error && !row_has_conflict {
            valid_count += 1;
        }
        for issue in &row_issues {
            match issue.issue_type.as_str() {
                ISSUE_ERROR => error_count += 1,
                ISSUE_WARNING => warning_count += 1,
                ISSUE_CONFLICT => conflict_count += 1,
                _ => {}
            }
        }
        rows.push(normalized);
        issues.extend(row_issues);
    }

    Ok(ImportPreviewResult {
        rows,
        issues,
        error_count,
        warning_count,
        conflict_count,
        valid_count,
    })
}

fn persist_application(
    conn: &Connection,
    row: &ImportNormalizedRow,
    overwrite: bool,
) -> Result<bool, String> {
    let now = chrono::Utc::now().to_rfc3339();
    if overwrite {
        if let Some(existing_id) = row.conflict_id.as_ref() {
            conn.execute(
                "UPDATE applications
                 SET name = ?1, type = ?2, address = ?3, port = ?4, env = ?5, status = ?6, description = ?7, updated_at = ?8
                 WHERE id = ?9 AND is_deleted = 0",
                rusqlite::params![
                    row.name,
                    row.item_type.clone().unwrap_or_else(|| "backend".to_string()),
                    row.address,
                    row.port,
                    row.env,
                    row.status.clone().unwrap_or_else(|| "running".to_string()),
                    row.description,
                    now,
                    existing_id
                ],
            )
            .map_err(|err| err.to_string())?;
            insert_audit_log(
                conn,
                "update",
                "application",
                existing_id,
                Some(&row.name),
                Some("import overwrite"),
            )?;
            return Ok(true);
        }
    }

    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO applications (id, name, type, address, port, tech_stack, deploy_mode, env, git_repo, owner, business_application_id, status, description, is_deleted, deleted_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, NULL, NULL, ?6, NULL, NULL, NULL, ?7, ?8, 0, NULL, ?9, ?9)",
        rusqlite::params![
            id,
            row.name,
            row.item_type.clone().unwrap_or_else(|| "backend".to_string()),
            row.address,
            row.port,
            row.env,
            row.status.clone().unwrap_or_else(|| "running".to_string()),
            row.description,
            now
        ],
    )
    .map_err(|err| err.to_string())?;
    insert_audit_log(
        conn,
        "create",
        "application",
        &id,
        Some(&row.name),
        Some("import create"),
    )?;
    Ok(false)
}

fn insert_row_record(
    conn: &Connection,
    job_id: &str,
    row: &ImportNormalizedRow,
    row_status: &str,
    error_message: Option<&str>,
    original: &ImportDraftRow,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO import_job_rows (id, job_id, row_no, resource_type, name, env, status, error_message, payload_json, normalized_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            job_id,
            row.row_no as i64,
            row.resource_type,
            row.name,
            row.env,
            row_status,
            error_message,
            serde_json::to_string(original).map_err(|err| err.to_string())?,
            serde_json::to_string(row).map_err(|err| err.to_string())?,
            now
        ],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn insert_issues(
    conn: &Connection,
    job_id: &str,
    issues: &[ImportValidationIssue],
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    for issue in issues {
        conn.execute(
            "INSERT INTO import_job_issues (id, job_id, row_no, field_key, issue_type, code, message, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                uuid::Uuid::new_v4().to_string(),
                job_id,
                issue.row_no as i64,
                issue.field_key,
                issue.issue_type,
                issue.code,
                issue.message,
                now
            ],
        )
        .map_err(|err| err.to_string())?;
    }
    Ok(())
}

fn execute_import_rows_inner(
    command: &str,
    conn: &Connection,
    input: ExecuteImportRowsInput,
) -> AppResult<ImportExecutionResult> {
    let strategy = ExecutionStrategy::parse(input.strategy.as_deref())
        .map_err(|err| AppError::validation(command, err))?;
    let preview = preview_import_rows_inner(
        command,
        conn,
        PreviewImportRowsInput {
            rows: input.rows.clone(),
        },
    )?;

    let job_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO import_jobs (id, status, strategy, total_rows, created_count, updated_count, skipped_count, failed_count, created_at, updated_at)
         VALUES (?1, 'running', ?2, ?3, 0, 0, 0, 0, ?4, ?4)",
        rusqlite::params![
            job_id,
            match strategy {
                ExecutionStrategy::Skip => "skip",
                ExecutionStrategy::Overwrite => "overwrite",
                ExecutionStrategy::Block => "block",
            },
            preview.rows.len() as i64,
            now
        ],
    )
    .map_err(|err| AppError::from_db_error(command, "写入导入任务", err))?;

    let mut issue_map: HashMap<u32, Vec<ImportValidationIssue>> = HashMap::new();
    for issue in preview.issues.clone() {
        issue_map.entry(issue.row_no).or_default().push(issue);
    }

    let mut created_count = 0_u32;
    let mut updated_count = 0_u32;
    let mut skipped_count = 0_u32;
    let mut failed_count = 0_u32;

    for (index, row) in preview.rows.iter().enumerate() {
        let row_issues = issue_map.get(&row.row_no).cloned().unwrap_or_default();
        let has_error = row_issues.iter().any(|item| item.issue_type == ISSUE_ERROR);
        let has_conflict = row_issues
            .iter()
            .any(|item| item.issue_type == ISSUE_CONFLICT);
        let row_status: &str;
        let mut row_error: Option<String> = None;

        if has_error {
            row_status = "failed";
            row_error = Some("预检失败，存在错误项".to_string());
            failed_count += 1;
        } else if has_conflict {
            match strategy {
                ExecutionStrategy::Skip => {
                    row_status = "skipped";
                    skipped_count += 1;
                }
                ExecutionStrategy::Block => {
                    row_status = "failed";
                    row_error = Some("存在冲突且策略为 block".to_string());
                    failed_count += 1;
                }
                ExecutionStrategy::Overwrite => match persist_application(conn, row, true) {
                    Ok(true) => {
                        row_status = "updated";
                        updated_count += 1;
                    }
                    Ok(false) => {
                        row_status = "created";
                        created_count += 1;
                    }
                    Err(err) => {
                        row_status = "failed";
                        row_error = Some(err);
                        failed_count += 1;
                    }
                },
            }
        } else {
            match persist_application(conn, row, false) {
                Ok(true) => {
                    row_status = "updated";
                    updated_count += 1;
                }
                Ok(false) => {
                    row_status = "created";
                    created_count += 1;
                }
                Err(err) => {
                    row_status = "failed";
                    row_error = Some(err);
                    failed_count += 1;
                }
            }
        }

        let source_row = input.rows.get(index).cloned().unwrap_or(ImportDraftRow {
            resource_type: row.resource_type.clone(),
            name: Some(row.name.clone()),
            env: Some(row.env.clone()),
            item_type: row.item_type.clone(),
            status: row.status.clone(),
            address: row.address.clone(),
            port: row.port,
            category: None,
            description: row.description.clone(),
        });

        insert_row_record(
            conn,
            &job_id,
            row,
            row_status,
            row_error.as_deref(),
            &source_row,
        )
        .map_err(|err| AppError::from_db_error(command, "写入导入行结果", err))?;
        insert_issues(conn, &job_id, &row_issues)
            .map_err(|err| AppError::from_db_error(command, "写入导入问题详情", err))?;
    }

    let final_status = if failed_count > 0 {
        if created_count + updated_count + skipped_count > 0 {
            "completed_with_errors"
        } else {
            "failed"
        }
    } else {
        "completed"
    };
    conn.execute(
        "UPDATE import_jobs
         SET status = ?1, created_count = ?2, updated_count = ?3, skipped_count = ?4, failed_count = ?5, updated_at = ?6
         WHERE id = ?7",
        rusqlite::params![
            final_status,
            created_count as i64,
            updated_count as i64,
            skipped_count as i64,
            failed_count as i64,
            chrono::Utc::now().to_rfc3339(),
            job_id
        ],
    )
    .map_err(|err| AppError::from_db_error(command, "更新导入任务汇总", err))?;

    Ok(ImportExecutionResult {
        job_id,
        status: final_status.to_string(),
        total_rows: preview.rows.len() as u32,
        created_count,
        updated_count,
        skipped_count,
        failed_count,
    })
}

fn list_import_jobs_inner(
    command: &str,
    conn: &Connection,
    params: QueryParams,
) -> AppResult<PagedResult<ImportJobSummary>> {
    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;
    let total: u64 = conn
        .query_row("SELECT COUNT(*) FROM import_jobs", [], |row| {
            row.get::<_, i64>(0)
        })
        .map(|value| value as u64)
        .map_err(|err| AppError::from_db_error(command, "查询导入任务数量", err))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, status, strategy, total_rows, created_count, updated_count, skipped_count, failed_count, created_at, updated_at
             FROM import_jobs
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(|err| AppError::from_db_error(command, "查询导入任务列表", err))?;
    let rows = stmt
        .query_map(rusqlite::params![page_size as i64, offset as i64], |row| {
            Ok(ImportJobSummary {
                id: row.get(0)?,
                status: row.get(1)?,
                strategy: row.get(2)?,
                total_rows: row.get::<_, i64>(3)? as u32,
                created_count: row.get::<_, i64>(4)? as u32,
                updated_count: row.get::<_, i64>(5)? as u32,
                skipped_count: row.get::<_, i64>(6)? as u32,
                failed_count: row.get::<_, i64>(7)? as u32,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|err| AppError::from_db_error(command, "读取导入任务列表", err))?;
    let data: Vec<ImportJobSummary> = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| AppError::from_db_error(command, "读取导入任务列表", err))?;

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

fn get_import_job_detail_inner(
    command: &str,
    conn: &Connection,
    job_id: String,
) -> AppResult<ImportJobDetail> {
    let summary = conn
        .query_row(
            "SELECT id, status, strategy, total_rows, created_count, updated_count, skipped_count, failed_count, created_at, updated_at
             FROM import_jobs
             WHERE id = ?1",
            rusqlite::params![job_id],
            |row| {
                Ok(ImportJobSummary {
                    id: row.get(0)?,
                    status: row.get(1)?,
                    strategy: row.get(2)?,
                    total_rows: row.get::<_, i64>(3)? as u32,
                    created_count: row.get::<_, i64>(4)? as u32,
                    updated_count: row.get::<_, i64>(5)? as u32,
                    skipped_count: row.get::<_, i64>(6)? as u32,
                    failed_count: row.get::<_, i64>(7)? as u32,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
        )
        .map_err(|err| AppError::not_found(command, "导入任务不存在", Some(err.to_string())))?;

    let mut row_stmt = conn
        .prepare(
            "SELECT row_no, resource_type, name, env, status, error_message
             FROM import_job_rows
             WHERE job_id = ?1
             ORDER BY row_no ASC",
        )
        .map_err(|err| AppError::from_db_error(command, "查询导入行结果", err))?;
    let rows = row_stmt
        .query_map(rusqlite::params![summary.id], |row| {
            Ok(ImportJobRowResult {
                row_no: row.get::<_, i64>(0)? as u32,
                resource_type: row.get(1)?,
                name: row.get(2)?,
                env: row.get(3)?,
                status: row.get(4)?,
                error_message: row.get(5)?,
            })
        })
        .map_err(|err| AppError::from_db_error(command, "读取导入行结果", err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| AppError::from_db_error(command, "读取导入行结果", err))?;

    let mut issue_stmt = conn
        .prepare(
            "SELECT row_no, field_key, issue_type, code, message
             FROM import_job_issues
             WHERE job_id = ?1
             ORDER BY row_no ASC, issue_type ASC",
        )
        .map_err(|err| AppError::from_db_error(command, "查询导入问题详情", err))?;
    let issues = issue_stmt
        .query_map(rusqlite::params![summary.id], |row| {
            Ok(ImportValidationIssue {
                row_no: row.get::<_, i64>(0)? as u32,
                field_key: row.get(1)?,
                issue_type: row.get(2)?,
                code: row.get(3)?,
                message: row.get(4)?,
            })
        })
        .map_err(|err| AppError::from_db_error(command, "读取导入问题详情", err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| AppError::from_db_error(command, "读取导入问题详情", err))?;

    Ok(ImportJobDetail {
        summary,
        rows,
        issues,
    })
}

#[tauri::command]
pub fn preview_import_rows(
    pool: State<DbPool>,
    input: PreviewImportRowsInput,
) -> AppResult<ImportPreviewResult> {
    let command = "preview_import_rows";
    let conn = pool
        .get()
        .map_err(|err| AppError::db_unavailable(command, format!("Pool error: {}", err)))?;
    preview_import_rows_inner(command, &conn, input)
}

#[tauri::command]
pub fn execute_import_rows(
    pool: State<DbPool>,
    input: ExecuteImportRowsInput,
) -> AppResult<ImportExecutionResult> {
    let command = "execute_import_rows";
    let conn = pool
        .get()
        .map_err(|err| AppError::db_unavailable(command, format!("Pool error: {}", err)))?;
    execute_import_rows_inner(command, &conn, input)
}

#[tauri::command]
pub fn list_import_jobs(
    pool: State<DbPool>,
    params: QueryParams,
) -> AppResult<PagedResult<ImportJobSummary>> {
    let command = "list_import_jobs";
    let conn = pool
        .get()
        .map_err(|err| AppError::db_unavailable(command, format!("Pool error: {}", err)))?;
    list_import_jobs_inner(command, &conn, params)
}

#[tauri::command]
pub fn get_import_job_detail(pool: State<DbPool>, job_id: String) -> AppResult<ImportJobDetail> {
    let command = "get_import_job_detail";
    let conn = pool
        .get()
        .map_err(|err| AppError::db_unavailable(command, format!("Pool error: {}", err)))?;
    get_import_job_detail_inner(command, &conn, job_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::QueryParams;
    use crate::test_helpers::{insert_test_application, setup_test_db};

    fn make_app_row(name: &str) -> ImportDraftRow {
        ImportDraftRow {
            resource_type: "application".to_string(),
            name: Some(name.to_string()),
            env: Some("prod".to_string()),
            item_type: Some("backend".to_string()),
            status: Some("running".to_string()),
            address: Some("10.0.0.10".to_string()),
            port: Some(8080),
            category: None,
            description: None,
        }
    }

    fn make_app_row_with_type(name: &str, app_type: &str) -> ImportDraftRow {
        ImportDraftRow {
            resource_type: "application".to_string(),
            name: Some(name.to_string()),
            env: Some("prod".to_string()),
            item_type: Some(app_type.to_string()),
            status: Some("running".to_string()),
            address: Some("10.0.0.10".to_string()),
            port: Some(8080),
            category: None,
            description: None,
        }
    }

    #[test]
    fn preview_import_rows_should_detect_existing_conflict() {
        let conn = setup_test_db();
        insert_test_application(&conn, "app-1", "orders-api", "prod");

        let preview = preview_import_rows_inner(
            "test",
            &conn,
            PreviewImportRowsInput {
                rows: vec![make_app_row("orders-api")],
            },
        )
        .expect("preview should succeed");

        assert_eq!(preview.conflict_count, 1);
        assert_eq!(preview.valid_count, 0);
    }

    #[test]
    fn preview_import_rows_should_not_conflict_when_type_is_different() {
        let conn = setup_test_db();
        insert_test_application(&conn, "app-1", "orders-api", "prod");

        let preview = preview_import_rows_inner(
            "test",
            &conn,
            PreviewImportRowsInput {
                rows: vec![make_app_row_with_type("orders-api", "gateway")],
            },
        )
        .expect("preview should succeed");

        assert_eq!(preview.conflict_count, 0);
        assert_eq!(preview.valid_count, 1);
    }

    #[test]
    fn execute_import_rows_should_skip_conflict_by_default() {
        let conn = setup_test_db();
        insert_test_application(&conn, "app-1", "orders-api", "prod");

        let result = execute_import_rows_inner(
            "test",
            &conn,
            ExecuteImportRowsInput {
                rows: vec![make_app_row("orders-api")],
                strategy: None,
            },
        )
        .expect("execute should succeed");

        assert_eq!(result.skipped_count, 1);
        assert_eq!(result.created_count, 0);
        assert_eq!(result.updated_count, 0);
        assert_eq!(result.failed_count, 0);
    }

    #[test]
    fn execute_import_rows_should_create_new_application() {
        let conn = setup_test_db();

        let result = execute_import_rows_inner(
            "test",
            &conn,
            ExecuteImportRowsInput {
                rows: vec![make_app_row("billing-api")],
                strategy: None,
            },
        )
        .expect("execute should succeed");
        assert_eq!(result.created_count, 1);
        assert_eq!(result.skipped_count, 0);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM applications WHERE name='billing-api' AND env='prod' AND is_deleted=0",
                [],
                |row| row.get(0),
            )
            .expect("query inserted application");
        assert_eq!(count, 1);
    }

    #[test]
    fn list_and_detail_should_return_import_job_records() {
        let conn = setup_test_db();
        let execute = execute_import_rows_inner(
            "test",
            &conn,
            ExecuteImportRowsInput {
                rows: vec![make_app_row("inventory-api")],
                strategy: Some("skip".to_string()),
            },
        )
        .expect("execute should succeed");

        let list = list_import_jobs_inner("test", &conn, QueryParams::default())
            .expect("list should succeed");
        assert_eq!(list.total, 1);
        assert_eq!(list.data.len(), 1);

        let detail = get_import_job_detail_inner("test", &conn, execute.job_id)
            .expect("detail should succeed");
        assert_eq!(detail.rows.len(), 1);
        assert_eq!(detail.summary.total_rows, 1);
    }
}
