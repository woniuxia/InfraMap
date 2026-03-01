mod backup;
mod commands;
mod db;
mod error;
mod models;
mod validation;

#[cfg(test)]
mod test_helpers;

use db::{init_db_pool, run_migrations};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

            let db_path = app_data_dir.join("inframap.db");
            let db_path_str = db_path.to_str().expect("Invalid database path");

            let pool =
                init_db_pool(db_path_str).expect("Failed to initialize database connection pool");

            run_migrations(&pool).expect("Database migration failed");

            app.manage(pool.clone());
            app.manage(backup::AppDataDir(app_data_dir.clone()));

            backup::start_auto_backup_thread(pool, app_data_dir);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health::health_check,
            commands::health::get_db_info,
            commands::hosts::list_hosts,
            commands::hosts::get_host,
            commands::hosts::save_host,
            commands::hosts::soft_delete_host,
            commands::ip_addresses::list_ip_addresses,
            commands::ip_addresses::get_ip_address,
            commands::ip_addresses::save_ip_address,
            commands::ip_addresses::batch_create_ip_addresses,
            commands::ip_addresses::soft_delete_ip_address,
            commands::host_ip_bindings::list_host_ip_bindings,
            commands::host_ip_bindings::bind_host_ip,
            commands::host_ip_bindings::unbind_host_ip,
            commands::applications::list_applications,
            commands::applications::get_application,
            commands::applications::save_application,
            commands::applications::soft_delete_application,
            commands::business_applications::list_business_applications,
            commands::business_applications::get_business_application,
            commands::business_applications::save_business_application,
            commands::business_applications::soft_delete_business_application,
            commands::business_applications::list_unassigned_application_services,
            commands::business_applications::attach_services_to_business_application,
            commands::business_applications::detach_service_from_business_application,
            commands::business_applications::replace_services_by_business_application,
            commands::business_applications::list_services_by_business_application,
            commands::middlewares::list_middlewares,
            commands::middlewares::get_middleware,
            commands::middlewares::save_middleware,
            commands::middlewares::soft_delete_middleware,
            commands::nginx_configs::list_nginx_configs,
            commands::nginx_configs::get_nginx_config,
            commands::nginx_configs::save_nginx_config,
            commands::nginx_configs::soft_delete_nginx_config,
            commands::deployments::list_deployments,
            commands::deployments::get_resource_deploy_context,
            commands::deployments::save_deployment,
            commands::deployments::soft_delete_deployment,
            commands::dependencies::list_call_relations,
            commands::dependencies::replace_resource_call_relations,
            commands::dashboard::get_dashboard_stats,
            commands::audit_logs::list_audit_logs,
            commands::topology::get_topology_graph,
            commands::topology::find_paths,
            commands::topology::analyze_impact,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::taxonomy::list_taxonomy_terms,
            commands::taxonomy::save_resource_terms_command,
            commands::taxonomy::list_resource_terms,
            commands::backup::create_backup,
            commands::backup::list_backups,
            commands::backup::delete_backup,
            commands::backup::preview_restore,
            commands::backup::restore_backup,
            commands::backup::export_json,
            commands::backup::import_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
