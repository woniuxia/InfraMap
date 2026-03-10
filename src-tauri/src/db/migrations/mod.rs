pub mod hooks;

mod v001_initial_schema;
mod v002_system_settings;
mod v003_host_cpu_fields;
mod v004_host_env;
mod v005_application_owners;
mod v006_nginx_address;
mod v007_ip_resources;
mod v008_ip_tags;
mod v009_business_applications;
mod v010_taxonomy_terms;
mod v011_taxonomy_term_stats;
mod v012_taxonomy_hook_placeholder;
mod v013_drop_host_legacy_ip;
mod v014_call_relations;
mod v015_business_application_owner_taxonomy;
mod v016_soft_delete_cleanup;
mod v017_import_jobs;
mod v018_host_taxonomy_backfill;
mod v019_application_name_env_type_index;
mod v020_nginx_endpoints;
mod v021_application_owner_taxonomy_cleanup;
mod v022_system_jobs;

pub const MIGRATIONS: &[(i32, &str)] = &[
    (1, v001_initial_schema::SQL),
    (2, v002_system_settings::SQL),
    (3, v003_host_cpu_fields::SQL),
    (4, v004_host_env::SQL),
    (5, v005_application_owners::SQL),
    (6, v006_nginx_address::SQL),
    (7, v007_ip_resources::SQL),
    (8, v008_ip_tags::SQL),
    (9, v009_business_applications::SQL),
    (10, v010_taxonomy_terms::SQL),
    (11, v011_taxonomy_term_stats::SQL),
    (12, v012_taxonomy_hook_placeholder::SQL),
    (13, v013_drop_host_legacy_ip::SQL),
    (14, v014_call_relations::SQL),
    (15, v015_business_application_owner_taxonomy::SQL),
    (16, v016_soft_delete_cleanup::SQL),
    (17, v017_import_jobs::SQL),
    (18, v018_host_taxonomy_backfill::SQL),
    (19, v019_application_name_env_type_index::SQL),
    (20, v020_nginx_endpoints::SQL),
    (21, v021_application_owner_taxonomy_cleanup::SQL),
    (22, v022_system_jobs::SQL),
];
