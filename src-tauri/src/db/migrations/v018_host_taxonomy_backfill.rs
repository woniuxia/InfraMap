pub const SQL: &str = r#"
        INSERT OR IGNORE INTO taxonomy_terms (id, field_key, normalized_value, display_name, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), 'os_type', lower(trim(os_type)), trim(os_type), 0, NULL, COALESCE(updated_at, datetime('now')), COALESCE(updated_at, datetime('now'))
        FROM hosts
        WHERE is_deleted = 0
          AND TRIM(COALESCE(os_type, '')) <> '';

        INSERT OR IGNORE INTO taxonomy_terms (id, field_key, normalized_value, display_name, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), 'cpu_model', lower(trim(cpu_model)), trim(cpu_model), 0, NULL, COALESCE(updated_at, datetime('now')), COALESCE(updated_at, datetime('now'))
        FROM hosts
        WHERE is_deleted = 0
          AND TRIM(COALESCE(cpu_model, '')) <> '';

        INSERT OR IGNORE INTO taxonomy_bindings (id, term_id, resource_type, resource_id, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), tt.id, 'host', h.id, 0, NULL, COALESCE(h.updated_at, datetime('now')), COALESCE(h.updated_at, datetime('now'))
        FROM hosts h
        JOIN taxonomy_terms tt
          ON tt.field_key = 'os_type'
         AND tt.normalized_value = lower(trim(h.os_type))
         AND tt.is_deleted = 0
        WHERE h.is_deleted = 0
          AND TRIM(COALESCE(h.os_type, '')) <> '';

        INSERT OR IGNORE INTO taxonomy_bindings (id, term_id, resource_type, resource_id, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), tt.id, 'host', h.id, 0, NULL, COALESCE(h.updated_at, datetime('now')), COALESCE(h.updated_at, datetime('now'))
        FROM hosts h
        JOIN taxonomy_terms tt
          ON tt.field_key = 'cpu_model'
         AND tt.normalized_value = lower(trim(h.cpu_model))
         AND tt.is_deleted = 0
        WHERE h.is_deleted = 0
          AND TRIM(COALESCE(h.cpu_model, '')) <> '';

        DELETE FROM taxonomy_term_stats;
        INSERT INTO taxonomy_term_stats (term_id, resource_type, usage_count, last_used_at, updated_at)
        SELECT tb.term_id, tb.resource_type, COUNT(*) AS usage_count, MAX(tb.updated_at) AS last_used_at, datetime('now')
        FROM taxonomy_bindings tb
        JOIN taxonomy_terms tt ON tt.id = tb.term_id
        WHERE tb.is_deleted = 0
          AND tt.is_deleted = 0
        GROUP BY tb.term_id, tb.resource_type;
    
"#;
