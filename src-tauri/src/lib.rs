mod db;
mod commands;
mod models;
mod validation;

use db::{init_db_pool, run_migrations};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to resolve app data directory");
            std::fs::create_dir_all(&app_data_dir)
                .expect("Failed to create app data directory");

            let db_path = app_data_dir.join("inframap.db");
            let db_path_str = db_path.to_str()
                .expect("Invalid database path");

            let pool = init_db_pool(db_path_str)
                .expect("Failed to initialize database connection pool");

            run_migrations(&pool)
                .expect("Database migration failed");

            app.manage(pool);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health::health_check,
            commands::health::get_db_info,
            commands::hosts::list_hosts,
            commands::hosts::get_host,
            commands::hosts::save_host,
            commands::hosts::soft_delete_host,
            commands::applications::list_applications,
            commands::applications::get_application,
            commands::applications::save_application,
            commands::applications::soft_delete_application,
            commands::middlewares::list_middlewares,
            commands::middlewares::get_middleware,
            commands::middlewares::save_middleware,
            commands::middlewares::soft_delete_middleware,
            commands::nginx_configs::list_nginx_configs,
            commands::nginx_configs::get_nginx_config,
            commands::nginx_configs::save_nginx_config,
            commands::nginx_configs::soft_delete_nginx_config,
            commands::deployments::list_deployments,
            commands::deployments::save_deployment,
            commands::deployments::soft_delete_deployment,
            commands::dependencies::list_dependencies,
            commands::dependencies::save_dependency,
            commands::dependencies::soft_delete_dependency,
            commands::dashboard::get_dashboard_stats,
            commands::audit_logs::list_audit_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
