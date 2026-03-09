pub const SQL: &str = r#"
        DROP INDEX IF EXISTS uk_hosts_ip;
        ALTER TABLE hosts DROP COLUMN ip_address;
    
"#;
