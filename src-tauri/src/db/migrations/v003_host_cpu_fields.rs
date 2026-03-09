pub const SQL: &str = r#"
        ALTER TABLE hosts ADD COLUMN cpu_model TEXT;
        ALTER TABLE hosts ADD COLUMN cpu_cores INTEGER;
        ALTER TABLE hosts ADD COLUMN cpu_threads INTEGER;
        ALTER TABLE hosts ADD COLUMN cpu_freq TEXT;
    
"#;
