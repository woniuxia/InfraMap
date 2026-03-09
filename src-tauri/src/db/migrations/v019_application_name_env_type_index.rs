pub const SQL: &str = r#"
        DROP INDEX IF EXISTS uk_applications_name_env;
        CREATE UNIQUE INDEX IF NOT EXISTS uk_applications_name_env_type
        ON applications(name, env, type) WHERE is_deleted = 0;
    
"#;
