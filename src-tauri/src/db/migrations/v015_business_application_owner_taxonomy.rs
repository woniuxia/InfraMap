pub const SQL: &str = r#"
        INSERT OR IGNORE INTO taxonomy_terms (id, field_key, normalized_value, display_name, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), 'owner', lower(trim(owner)), trim(owner), 0, NULL, COALESCE(updated_at, datetime('now')), COALESCE(updated_at, datetime('now'))
        FROM business_applications
        WHERE is_deleted = 0
          AND TRIM(COALESCE(owner, '')) <> '';

        INSERT OR IGNORE INTO taxonomy_bindings (id, term_id, resource_type, resource_id, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), tt.id, 'business_application', ba.id, 0, NULL, COALESCE(ba.updated_at, datetime('now')), COALESCE(ba.updated_at, datetime('now'))
        FROM business_applications ba
        JOIN taxonomy_terms tt
          ON tt.field_key = 'owner'
         AND tt.normalized_value = lower(trim(ba.owner))
         AND tt.is_deleted = 0
        WHERE ba.is_deleted = 0
          AND TRIM(COALESCE(ba.owner, '')) <> '';

        DELETE FROM taxonomy_term_stats;
        INSERT INTO taxonomy_term_stats (term_id, resource_type, usage_count, last_used_at, updated_at)
        SELECT tb.term_id, tb.resource_type, COUNT(*) AS usage_count, MAX(tb.updated_at) AS last_used_at, datetime('now')
        FROM taxonomy_bindings tb
        JOIN taxonomy_terms tt ON tt.id = tb.term_id
        WHERE tb.is_deleted = 0
          AND tt.is_deleted = 0
        GROUP BY tb.term_id, tb.resource_type;

        ALTER TABLE business_applications DROP COLUMN owner;
    
"#;
