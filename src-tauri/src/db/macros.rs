/// Generates a `#[tauri::command]` save function with standard upsert pattern:
/// validate → (prepare) → get_conn → resolve_upsert → transaction(INSERT or UPDATE + audit).
///
/// The caller provides identifiers for `data`, `persisted_id`, `now`, and `command`
/// via the closure-like syntax `|data, pid, now, cmd| { ... }`. These identifiers
/// are used as variable names in the generated function, allowing the caller's
/// SQL param expressions to reference them directly.
///
/// # Optional `prepare` block
///
/// Use `prepare: { ... }` to run code after validation but before the transaction.
/// Useful for pre-processing (e.g., serializing JSON fields). The block can reference
/// the `data` and `command` identifiers provided by the caller.
macro_rules! impl_save_command {
    (
        fn_name: $fn_name:ident,
        model: $model:ty,
        table: $table:expr,
        resource_type: $resource_type:expr,
        validator: $validator:path,
        create_label: $create_label:expr,
        update_label: $update_label:expr,
        |$data:ident, $pid:ident, $now:ident, $cmd:ident| {
            $(prepare: { $($prepare:tt)* },)?
            insert: ($insert_sql:expr, [$($ip:expr),+ $(,)?]),
            update: ($update_sql:expr, [$($up:expr),+ $(,)?]),
            display_name: $dn:expr $(,)?
        } $(,)?
    ) => {
        #[tauri::command]
        pub fn $fn_name(
            pool: tauri::State<crate::db::DbPool>,
            $data: $model,
        ) -> crate::error::AppResult<String> {
            let $cmd = stringify!($fn_name);
            $validator(&$data).map_err(|e| crate::error::AppError::validation($cmd, e))?;
            $($($prepare)*)?
            let __save_conn = crate::db::get_conn(pool.inner(), $cmd)?;
            let $now = chrono::Utc::now().to_rfc3339();
            let ($pid, __save_is_new) =
                crate::db::crud::resolve_upsert_state($cmd, &__save_conn, $table, &$data.id)?;

            crate::db::transaction::with_transaction(&__save_conn, $cmd, |__save_tx| {
                if __save_is_new {
                    __save_tx
                        .execute($insert_sql, rusqlite::params![$($ip),+])
                        .map_err(|e| {
                            crate::error::AppError::from_db_error($cmd, $create_label, e)
                        })?;
                    crate::db::crud::write_audit_log_entry(
                        $cmd,
                        __save_tx,
                        "create",
                        $resource_type,
                        &$pid,
                        Some(&$dn),
                    )?;
                    Ok($pid.clone())
                } else {
                    __save_tx
                        .execute($update_sql, rusqlite::params![$($up),+])
                        .map_err(|e| {
                            crate::error::AppError::from_db_error($cmd, $update_label, e)
                        })?;
                    crate::db::crud::write_audit_log_entry(
                        $cmd,
                        __save_tx,
                        "update",
                        $resource_type,
                        &$data.id,
                        Some(&$dn),
                    )?;
                    Ok($data.id)
                }
            })
        }
    };
}

/// Generates a `#[tauri::command]` soft-delete function with standard pattern:
/// get_conn → (optional name lookup) → transaction(delete_with_audit).
///
/// Use `name_column` to look up a display name before deletion for the audit log.
/// Omit it for resources without a meaningful name (e.g., deployments).
macro_rules! impl_delete_command {
    // With name lookup for audit log
    (
        fn_name: $fn_name:ident,
        table: $table:expr,
        resource_type: $resource_type:expr,
        delete_label: $delete_label:expr,
        name_column: $name_col:expr $(,)?
    ) => {
        #[tauri::command]
        pub fn $fn_name(
            pool: tauri::State<crate::db::DbPool>,
            id: String,
        ) -> crate::error::AppResult<()> {
            let command = stringify!($fn_name);
            let conn = crate::db::get_conn(pool.inner(), command)?;
            let name: Option<String> = conn
                .query_row(
                    &format!(
                        "SELECT {} FROM {} WHERE id = ?1 AND is_deleted = 0",
                        $name_col, $table
                    ),
                    rusqlite::params![id],
                    |row| row.get(0),
                )
                .ok();
            crate::db::transaction::with_transaction(&conn, command, |conn| {
                crate::db::crud::delete_with_audit(
                    command,
                    $delete_label,
                    conn,
                    $table,
                    $resource_type,
                    &id,
                    name.as_deref(),
                )
            })
        }
    };
    // Without name lookup (passes None to audit log)
    (
        fn_name: $fn_name:ident,
        table: $table:expr,
        resource_type: $resource_type:expr,
        delete_label: $delete_label:expr $(,)?
    ) => {
        #[tauri::command]
        pub fn $fn_name(
            pool: tauri::State<crate::db::DbPool>,
            id: String,
        ) -> crate::error::AppResult<()> {
            let command = stringify!($fn_name);
            let conn = crate::db::get_conn(pool.inner(), command)?;
            crate::db::transaction::with_transaction(&conn, command, |conn| {
                crate::db::crud::delete_with_audit(
                    command,
                    $delete_label,
                    conn,
                    $table,
                    $resource_type,
                    &id,
                    None,
                )
            })
        }
    };
}
