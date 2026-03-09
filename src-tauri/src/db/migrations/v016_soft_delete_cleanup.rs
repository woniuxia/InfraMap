pub const SQL: &str = r#"
        DELETE FROM host_ip_bindings
         WHERE host_id IN (SELECT id FROM hosts WHERE is_deleted = 1)
            OR ip_id IN (SELECT id FROM ip_addresses WHERE is_deleted = 1)
            OR is_deleted = 1;

        DELETE FROM deployments WHERE is_deleted = 1;
        DELETE FROM call_relations WHERE is_deleted = 1;
        DELETE FROM applications WHERE is_deleted = 1;
        DELETE FROM business_applications WHERE is_deleted = 1;
        DELETE FROM middlewares WHERE is_deleted = 1;
        DELETE FROM nginx_configs WHERE is_deleted = 1;
        DELETE FROM hosts WHERE is_deleted = 1;
        DELETE FROM ip_addresses WHERE is_deleted = 1;
        DELETE FROM taxonomy_bindings WHERE is_deleted = 1;
        DELETE FROM taxonomy_terms WHERE is_deleted = 1;
    
"#;
