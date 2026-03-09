pub const SQL: &str = r#"
        ALTER TABLE nginx_configs ADD COLUMN endpoints TEXT NOT NULL DEFAULT '[]';
        ALTER TABLE nginx_configs DROP COLUMN address;
        ALTER TABLE nginx_configs DROP COLUMN listen_port;
        ALTER TABLE nginx_configs DROP COLUMN upstream_servers;
    
"#;
