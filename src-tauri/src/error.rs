use serde::Serialize;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppErrorCode {
    ValidationError,
    NotFound,
    Conflict,
    DependencyConflict,
    DbUnavailable,
    DbQueryFailed,
    IoError,
    BackupError,
    InternalError,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: AppErrorCode,
    pub message: String,
    pub details: Option<String>,
    pub command: String,
    pub retryable: bool,
}

pub type AppResult<T> = Result<T, AppError>;

#[allow(dead_code)]
impl AppError {
    pub fn new(
        code: AppErrorCode,
        message: impl Into<String>,
        details: Option<String>,
        command: &str,
        retryable: bool,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            details,
            command: command.to_string(),
            retryable,
        }
    }

    pub fn validation(command: &str, details: impl Into<String>) -> Self {
        Self::new(
            AppErrorCode::ValidationError,
            "参数校验失败，请检查输入内容。",
            Some(details.into()),
            command,
            false,
        )
    }

    pub fn not_found(command: &str, message: impl Into<String>, details: Option<String>) -> Self {
        Self::new(AppErrorCode::NotFound, message, details, command, false)
    }

    pub fn conflict(command: &str, message: impl Into<String>, details: Option<String>) -> Self {
        Self::new(AppErrorCode::Conflict, message, details, command, false)
    }

    pub fn dependency_conflict(
        command: &str,
        message: impl Into<String>,
        details: Option<String>,
    ) -> Self {
        Self::new(
            AppErrorCode::DependencyConflict,
            message,
            details,
            command,
            false,
        )
    }

    pub fn db_unavailable(command: &str, details: impl Into<String>) -> Self {
        Self::new(
            AppErrorCode::DbUnavailable,
            "数据库暂时不可用，请稍后重试。",
            Some(details.into()),
            command,
            true,
        )
    }

    pub fn db_query_failed(
        command: &str,
        action: impl Into<String>,
        details: impl ToString,
    ) -> Self {
        let action = action.into();
        Self::new(
            AppErrorCode::DbQueryFailed,
            format!("{}失败，请稍后重试。", action),
            Some(details.to_string()),
            command,
            true,
        )
    }

    pub fn io_error(command: &str, message: impl Into<String>, details: Option<String>) -> Self {
        Self::new(AppErrorCode::IoError, message, details, command, true)
    }

    pub fn backup_error(
        command: &str,
        message: impl Into<String>,
        details: Option<String>,
    ) -> Self {
        Self::new(AppErrorCode::BackupError, message, details, command, false)
    }

    pub fn internal(command: &str, details: impl Into<String>) -> Self {
        Self::new(
            AppErrorCode::InternalError,
            "发生内部异常，请稍后重试。",
            Some(details.into()),
            command,
            false,
        )
    }

    pub fn from_db_error(command: &str, action: &str, err: impl ToString) -> Self {
        let raw = err.to_string();

        if raw.contains("UNIQUE constraint failed: hosts.ip_address") {
            return Self::conflict(
                command,
                "保存失败，IP 地址已存在，请使用其他 IP 地址。",
                Some(raw),
            );
        }

        if raw.contains("UNIQUE constraint failed") {
            return Self::conflict(command, "保存失败，存在重复数据，请检查后重试。", Some(raw));
        }

        if raw.contains("FOREIGN KEY constraint failed") {
            return Self::dependency_conflict(
                command,
                "操作失败，存在关联依赖，请先处理引用关系。",
                Some(raw),
            );
        }

        if raw.contains("Record not found") || raw.contains("not found") {
            return Self::not_found(command, "目标记录不存在或已被删除。", Some(raw));
        }

        Self::db_query_failed(command, action, raw)
    }

    pub fn from_io_error(command: &str, action: &str, err: impl ToString) -> Self {
        let raw = err.to_string();
        Self::io_error(
            command,
            format!("{}失败，请检查文件路径和权限。", action),
            Some(raw),
        )
    }

    pub fn from_backup_error(command: &str, action: &str, err: impl ToString) -> Self {
        let raw = err.to_string();
        Self::backup_error(command, format!("{}失败，请重试。", action), Some(raw))
    }

    pub fn from_legacy(command: &str, raw: impl ToString) -> Self {
        let message = raw.to_string();

        if message.contains("Pool error") {
            return Self::db_unavailable(command, message);
        }

        if message.contains("Invalid") || message.contains("required") || message.contains("must") {
            return Self::validation(command, message);
        }

        Self::internal(command, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_unique_constraint_to_conflict() {
        let err = AppError::from_db_error(
            "save_host",
            "保存主机",
            "UNIQUE constraint failed: hosts.ip_address",
        );
        assert_eq!(err.code, AppErrorCode::Conflict);
        assert_eq!(err.message, "保存失败，IP 地址已存在，请使用其他 IP 地址。");
    }

    #[test]
    fn map_pool_error_to_db_unavailable() {
        let err = AppError::db_unavailable("list_hosts", "Pool error: timeout");
        assert_eq!(err.code, AppErrorCode::DbUnavailable);
        assert!(err.retryable);
    }

    #[test]
    fn map_foreign_key_to_dependency_conflict() {
        let err = AppError::from_db_error(
            "save_dependency",
            "保存依赖",
            "FOREIGN KEY constraint failed",
        );
        assert_eq!(err.code, AppErrorCode::DependencyConflict);
    }

    #[test]
    fn map_legacy_validation_to_validation_code() {
        let err = AppError::from_legacy("save_host", "hostname is required");
        assert_eq!(err.code, AppErrorCode::ValidationError);
    }
}
