use rusqlite::Connection;

pub fn insert_audit_log(
    conn: &Connection,
    action: &str,
    resource_type: &str,
    resource_id: &str,
    resource_name: Option<&str>,
    changes: Option<&str>,
) -> Result<(), String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO audit_logs (id, action, resource_type, resource_id, resource_name, changes, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, action, resource_type, resource_id, resource_name, changes, now],
    )
    .map_err(|e| format!("Failed to insert audit log: {}", e))?;
    Ok(())
}
