pub const SQL: &str = r#"
        ALTER TABLE nginx_configs ADD COLUMN address TEXT NOT NULL DEFAULT '';
        UPDATE nginx_configs SET address = 'unknown' WHERE TRIM(address) = '';
    
"#;
