use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;
use tauri::State;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::common::{PagedResult, QueryParams};
use crate::models::system_job::{
    SystemJobDetail, SystemJobImportIssue, SystemJobImportRow, SystemJobSummary,
};

pub(crate) fn get_latest_system_job_finished_at(
    conn: &Connection,
    job_type: &str,
) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT finished_at
         FROM system_jobs
         WHERE job_type = ?1
           AND finished_at IS NOT NULL
         ORDER BY created_at DESC
         LIMIT 1",
        params![job_type],
        |row| row.get(0),
    )
    .optional()
    .map_err(|err| format!("query latest system job finished_at failed: {}", err))
}

#[derive(Debug, Clone)]
pub(crate) struct CompletedSystemJob {
    pub job_type: String,
    pub title: String,
    pub status: String,
    pub summary: Option<String>,
    pub progress_percent: f64,
    pub retryable: bool,
    pub cancellable: bool,
    pub payload: Option<Value>,
    pub result: Option<Value>,
    pub error_message: Option<String>,
}

pub(crate) fn insert_completed_system_job(
    conn: &Connection,
    job: CompletedSystemJob,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let payload_json = job
        .payload
        .unwrap_or_else(|| serde_json::json!({}))
        .to_string();
    let result_json = job.result.map(|value| value.to_string());

    conn.execute(
        "INSERT INTO system_jobs (
            id, job_type, title, status, summary, progress_percent, retryable, cancellable,
            payload_json, result_json, error_message, created_at, updated_at, finished_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            id,
            job.job_type,
            job.title,
            job.status,
            job.summary,
            job.progress_percent,
            if job.retryable { 1 } else { 0 },
            if job.cancellable { 1 } else { 0 },
            payload_json,
            result_json,
            job.error_message,
            now,
            now,
            now,
        ],
    )
    .map_err(|err| format!("insert system job failed: {}", err))?;

    Ok(id)
}

pub(crate) fn record_system_job(
    conn: &Connection,
    job_type: &str,
    title: &str,
    status: &str,
    summary: Option<&str>,
    payload: Option<&Value>,
    result: Option<&Value>,
    error_message: Option<&str>,
    retryable: bool,
    cancellable: bool,
) -> Result<String, String> {
    insert_completed_system_job(
        conn,
        CompletedSystemJob {
            job_type: job_type.to_string(),
            title: title.to_string(),
            status: status.to_string(),
            summary: summary.map(str::to_string),
            progress_percent: if matches!(status, "queued" | "running") { 50.0 } else { 100.0 },
            retryable,
            cancellable,
            payload: payload.cloned(),
            result: result.cloned(),
            error_message: error_message.map(str::to_string),
        },
    )
}

pub(crate) fn get_latest_system_job_result_json(
    conn: &Connection,
    job_type: &str,
) -> Result<Option<Value>, String> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT result_json FROM system_jobs
             WHERE job_type = ?1 AND status = 'completed' AND result_json IS NOT NULL
             ORDER BY created_at DESC LIMIT 1",
            params![job_type],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("query latest system job result failed: {}", err))?;

    match raw {
        Some(content) => serde_json::from_str(&content)
            .map(Some)
            .map_err(|err| format!("parse latest system job result failed: {}", err)),
        None => Ok(None),
    }
}

fn deserialize_json(raw: Option<String>) -> Result<Option<Value>, String> {
    match raw {
        Some(content) if !content.trim().is_empty() => serde_json::from_str::<Value>(&content)
            .map(Some)
            .map_err(|err| format!("parse job json failed: {}", err)),
        _ => Ok(None),
    }
}

fn build_import_job_summary(
    id: String,
    status: String,
    strategy: String,
    total_rows: u32,
    created_count: u32,
    updated_count: u32,
    skipped_count: u32,
    failed_count: u32,
    created_at: String,
    updated_at: String,
) -> SystemJobSummary {
    SystemJobSummary {
        id: format!("import:{}", id),
        source: "import".to_string(),
        job_type: "import_rows".to_string(),
        title: "批量录入".to_string(),
        status,
        summary: Some(format!(
            "created={} / updated={} / skipped={} / failed={} / strategy={}",
            created_count, updated_count, skipped_count, failed_count, strategy
        )),
        progress_percent: if total_rows == 0 { 0.0 } else { 100.0 },
        retryable: false,
        cancellable: false,
        created_at,
        updated_at: updated_at.clone(),
        finished_at: Some(updated_at),
    }
}

fn load_import_rows(conn: &Connection, import_job_id: &str) -> Result<Vec<SystemJobImportRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT row_no, resource_type, name, env, status, error_message
             FROM import_job_rows WHERE job_id = ?1 ORDER BY row_no ASC",
        )
        .map_err(|err| format!("prepare import job rows failed: {}", err))?;
    let rows = stmt
        .query_map(params![import_job_id], |row| {
            Ok(SystemJobImportRow {
                row_no: row.get(0)?,
                resource_type: row.get(1)?,
                name: row.get(2)?,
                env: row.get(3)?,
                status: row.get(4)?,
                error_message: row.get(5)?,
            })
        })
        .map_err(|err| format!("query import job rows failed: {}", err))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("read import job rows failed: {}", err))
}

fn load_import_issues(
    conn: &Connection,
    import_job_id: &str,
) -> Result<Vec<SystemJobImportIssue>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT row_no, field_key, issue_type, code, message
             FROM import_job_issues WHERE job_id = ?1 ORDER BY row_no ASC, created_at ASC",
        )
        .map_err(|err| format!("prepare import job issues failed: {}", err))?;
    let rows = stmt
        .query_map(params![import_job_id], |row| {
            Ok(SystemJobImportIssue {
                row_no: row.get(0)?,
                field_key: row.get(1)?,
                issue_type: row.get(2)?,
                code: row.get(3)?,
                message: row.get(4)?,
            })
        })
        .map_err(|err| format!("query import job issues failed: {}", err))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("read import job issues failed: {}", err))
}

fn list_system_job_rows(conn: &Connection) -> Result<Vec<SystemJobSummary>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, job_type, title, status, summary, progress_percent, retryable, cancellable,
                    created_at, updated_at, finished_at
             FROM system_jobs ORDER BY created_at DESC",
        )
        .map_err(|err| format!("prepare system jobs failed: {}", err))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(SystemJobSummary {
                id: format!("system:{}", row.get::<_, String>(0)?),
                source: "system".to_string(),
                job_type: row.get(1)?,
                title: row.get(2)?,
                status: row.get(3)?,
                summary: row.get(4)?,
                progress_percent: row.get(5)?,
                retryable: row.get::<_, i64>(6)? != 0,
                cancellable: row.get::<_, i64>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                finished_at: row.get(10)?,
            })
        })
        .map_err(|err| format!("query system jobs failed: {}", err))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("read system jobs failed: {}", err))
}

fn list_import_job_rows_as_jobs(conn: &Connection) -> Result<Vec<SystemJobSummary>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, status, strategy, total_rows, created_count, updated_count, skipped_count, failed_count, created_at, updated_at
             FROM import_jobs ORDER BY created_at DESC",
        )
        .map_err(|err| format!("prepare import jobs failed: {}", err))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(build_import_job_summary(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get(9)?,
            ))
        })
        .map_err(|err| format!("query import jobs failed: {}", err))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("read import jobs failed: {}", err))
}

fn apply_filters(items: Vec<SystemJobSummary>, params: &QueryParams) -> Vec<SystemJobSummary> {
    let search = params.search.as_ref().map(|value| value.to_lowercase());
    let filters = params.filters.as_ref();

    items
        .into_iter()
        .filter(|item| {
            if let Some(search) = &search {
                let haystack = format!(
                    "{} {} {} {} {}",
                    item.title,
                    item.job_type,
                    item.status,
                    item.summary.clone().unwrap_or_default(),
                    item.source
                )
                .to_lowercase();
                if !haystack.contains(search) {
                    return false;
                }
            }

            if let Some(filters) = filters {
                if let Some(status) = filters.get("status") {
                    let allowed: Vec<&str> = status.split(',').map(str::trim).filter(|value| !value.is_empty()).collect();
                    if !allowed.is_empty() && !allowed.iter().any(|value| *value == item.status) {
                        return false;
                    }
                }
                if let Some(job_type) = filters.get("job_type") {
                    let allowed: Vec<&str> = job_type.split(',').map(str::trim).filter(|value| !value.is_empty()).collect();
                    if !allowed.is_empty() && !allowed.iter().any(|value| *value == item.job_type) {
                        return false;
                    }
                }
                if let Some(source) = filters.get("source") {
                    let allowed: Vec<&str> = source.split(',').map(str::trim).filter(|value| !value.is_empty()).collect();
                    if !allowed.is_empty() && !allowed.iter().any(|value| *value == item.source) {
                        return false;
                    }
                }
            }

            true
        })
        .collect()
}

pub(crate) fn list_system_jobs_inner(
    command: &str,
    conn: &Connection,
    params: QueryParams,
) -> AppResult<PagedResult<SystemJobSummary>> {
    let mut items = list_system_job_rows(conn)
        .map_err(|err| AppError::from_db_error(command, "读取系统任务", err))?;
    items.extend(
        list_import_job_rows_as_jobs(conn)
            .map_err(|err| AppError::from_db_error(command, "读取导入任务", err))?,
    );
    items.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

    let filtered = apply_filters(items, &params);
    let total = filtered.len() as u64;
    let page = params.page();
    let page_size = params.page_size();
    let start = ((page.saturating_sub(1)) * page_size) as usize;
    let end = (start + page_size as usize).min(filtered.len());
    let data = if start >= filtered.len() {
        Vec::new()
    } else {
        filtered[start..end].to_vec()
    };

    Ok(PagedResult {
        data,
        total,
        page,
        page_size,
    })
}

fn get_import_job_detail(conn: &Connection, import_job_id: &str) -> Result<SystemJobDetail, String> {
    let summary = conn
        .query_row(
            "SELECT id, status, strategy, total_rows, created_count, updated_count, skipped_count, failed_count, created_at, updated_at
             FROM import_jobs WHERE id = ?1",
            params![import_job_id],
            |row| {
                Ok(build_import_job_summary(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                ))
            },
        )
        .optional()
        .map_err(|err| format!("query import job summary failed: {}", err))?
        .ok_or_else(|| "import job not found".to_string())?;

    Ok(SystemJobDetail {
        summary,
        payload: None,
        result: None,
        error_message: None,
        import_rows: Some(load_import_rows(conn, import_job_id)?),
        import_issues: Some(load_import_issues(conn, import_job_id)?),
    })
}

pub(crate) fn get_system_job_detail_inner(
    command: &str,
    conn: &Connection,
    job_id: String,
) -> AppResult<SystemJobDetail> {
    if let Some(import_job_id) = job_id.strip_prefix("import:") {
        return get_import_job_detail(conn, import_job_id)
            .map_err(|err| AppError::not_found(command, "任务不存在", Some(err)));
    }

    let raw_id = job_id.strip_prefix("system:").unwrap_or(job_id.as_str());
    let row = conn
        .query_row(
            "SELECT id, job_type, title, status, summary, progress_percent, retryable, cancellable,
                    payload_json, result_json, error_message, created_at, updated_at, finished_at
             FROM system_jobs WHERE id = ?1",
            params![raw_id],
            |row| {
                Ok((
                    SystemJobSummary {
                        id: format!("system:{}", row.get::<_, String>(0)?),
                        source: "system".to_string(),
                        job_type: row.get(1)?,
                        title: row.get(2)?,
                        status: row.get(3)?,
                        summary: row.get(4)?,
                        progress_percent: row.get(5)?,
                        retryable: row.get::<_, i64>(6)? != 0,
                        cancellable: row.get::<_, i64>(7)? != 0,
                        created_at: row.get(11)?,
                        updated_at: row.get(12)?,
                        finished_at: row.get(13)?,
                    },
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, Option<String>>(10)?,
                ))
            },
        )
        .optional()
        .map_err(|err| AppError::from_db_error(command, "查询任务详情", err))?
        .ok_or_else(|| AppError::not_found(command, "任务不存在", Some(raw_id.to_string())))?;

    Ok(SystemJobDetail {
        summary: row.0,
        payload: deserialize_json(row.1).map_err(|err| AppError::internal(command, err))?,
        result: deserialize_json(row.2).map_err(|err| AppError::internal(command, err))?,
        error_message: row.3,
        import_rows: None,
        import_issues: None,
    })
}

#[tauri::command]
pub fn list_system_jobs(pool: State<DbPool>, params: QueryParams) -> AppResult<PagedResult<SystemJobSummary>> {
    let command = "list_system_jobs";
    let conn = pool
        .get()
        .map_err(|err| AppError::db_unavailable(command, format!("Pool error: {}", err)))?;
    list_system_jobs_inner(command, &conn, params)
}

#[tauri::command]
pub fn get_system_job_detail(pool: State<DbPool>, job_id: String) -> AppResult<SystemJobDetail> {
    let command = "get_system_job_detail";
    let conn = pool
        .get()
        .map_err(|err| AppError::db_unavailable(command, format!("Pool error: {}", err)))?;
    get_system_job_detail_inner(command, &conn, job_id)
}

#[tauri::command]
pub fn retry_system_job(_pool: State<DbPool>, _job_id: String) -> AppResult<()> {
    Err(AppError::validation(
        "retry_system_job",
        "当前版本暂未开放任务重试，请重新执行对应操作。",
    ))
}

#[tauri::command]
pub fn cancel_system_job(_pool: State<DbPool>, _job_id: String) -> AppResult<()> {
    Err(AppError::validation(
        "cancel_system_job",
        "当前版本任务为同步执行，不支持取消。",
    ))
}

#[cfg(test)]
mod tests {
    use super::{get_system_job_detail_inner, list_system_jobs_inner, record_system_job};
    use crate::models::common::QueryParams;
    use crate::test_helpers::setup_test_db;
    use rusqlite::params;

    #[test]
    fn list_system_jobs_inner_should_merge_system_and_import_jobs() {
        let conn = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO import_jobs (id, status, strategy, total_rows, created_count, updated_count, skipped_count, failed_count, created_at, updated_at)
             VALUES ('job-import', 'completed', 'skip', 1, 1, 0, 0, 0, ?1, ?1)",
            params![now],
        )
        .expect("insert import job");

        let job_id = record_system_job(
            &conn,
            "snapshot_export_v2",
            "导出快照 V2",
            "completed",
            Some("导出成功"),
            Some(&serde_json::json!({ "filepath": "E:/tmp/a.json" })),
            Some(&serde_json::json!({ "total_rows": 3 })),
            None,
            false,
            false,
        )
        .expect("record system job");

        let list = list_system_jobs_inner("test", &conn, QueryParams::default()).expect("list jobs");
        assert_eq!(list.total, 2);
        assert!(list.data.iter().any(|item| item.id == format!("system:{}", job_id)));
        assert!(list.data.iter().any(|item| item.id == "import:job-import"));
    }

    #[test]
    fn get_system_job_detail_inner_should_return_system_and_import_details() {
        let conn = setup_test_db();
        let job_id = record_system_job(
            &conn,
            "integrity_scan",
            "完整性扫描",
            "completed",
            Some("发现 1 个问题"),
            Some(&serde_json::json!({ "scope": "all" })),
            Some(&serde_json::json!({ "total": 1 })),
            None,
            false,
            false,
        )
        .expect("record system job");

        let detail = get_system_job_detail_inner("test", &conn, format!("system:{}", job_id))
            .expect("system detail");
        assert_eq!(detail.summary.job_type, "integrity_scan");
        assert_eq!(detail.payload.expect("payload")["scope"], "all");

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO import_jobs (id, status, strategy, total_rows, created_count, updated_count, skipped_count, failed_count, created_at, updated_at)
             VALUES ('job-import', 'completed', 'skip', 1, 1, 0, 0, 0, ?1, ?1)",
            params![now],
        )
        .expect("insert import job");
        conn.execute(
            "INSERT INTO import_job_rows (id, job_id, row_no, resource_type, name, env, status, error_message, payload_json, normalized_json, created_at, updated_at)
             VALUES ('row-1', 'job-import', 1, 'application', 'orders-api', 'prod', 'created', NULL, '{}', '{}', ?1, ?1)",
            params![now],
        )
        .expect("insert import row");

        let import_detail = get_system_job_detail_inner("test", &conn, "import:job-import".to_string())
            .expect("import detail");
        assert_eq!(import_detail.summary.source, "import");
        assert_eq!(import_detail.import_rows.expect("rows").len(), 1);
    }
}

