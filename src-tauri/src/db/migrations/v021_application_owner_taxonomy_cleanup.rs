pub const SQL: &str = r#"
        INSERT OR IGNORE INTO taxonomy_terms (id, field_key, normalized_value, display_name, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), 'owner', lower(trim(owner)), trim(owner), 0, NULL, COALESCE(updated_at, datetime('now')), COALESCE(updated_at, datetime('now'))
        FROM applications
        WHERE is_deleted = 0
          AND TRIM(COALESCE(owner, '')) <> '';

        INSERT OR IGNORE INTO taxonomy_terms (id, field_key, normalized_value, display_name, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), 'owner', lower(trim(owner_name)), trim(owner_name), 0, NULL, COALESCE(updated_at, datetime('now')), COALESCE(updated_at, datetime('now'))
        FROM application_owners
        WHERE is_deleted = 0
          AND TRIM(COALESCE(owner_name, '')) <> '';

        INSERT OR IGNORE INTO taxonomy_bindings (id, term_id, resource_type, resource_id, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), tt.id, 'application', a.id, 0, NULL, COALESCE(a.updated_at, datetime('now')), COALESCE(a.updated_at, datetime('now'))
        FROM applications a
        JOIN taxonomy_terms tt
          ON tt.field_key = 'owner'
         AND tt.normalized_value = lower(trim(a.owner))
         AND tt.is_deleted = 0
        WHERE a.is_deleted = 0
          AND TRIM(COALESCE(a.owner, '')) <> '';

        INSERT OR IGNORE INTO taxonomy_bindings (id, term_id, resource_type, resource_id, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), tt.id, 'application', ao.application_id, 0, NULL, COALESCE(ao.updated_at, datetime('now')), COALESCE(ao.updated_at, datetime('now'))
        FROM application_owners ao
        JOIN applications a ON a.id = ao.application_id AND a.is_deleted = 0
        JOIN taxonomy_terms tt
          ON tt.field_key = 'owner'
         AND tt.normalized_value = lower(trim(ao.owner_name))
         AND tt.is_deleted = 0
        WHERE ao.is_deleted = 0
          AND TRIM(COALESCE(ao.owner_name, '')) <> '';

        DELETE FROM taxonomy_term_stats;
        INSERT INTO taxonomy_term_stats (term_id, resource_type, usage_count, last_used_at, updated_at)
        SELECT tb.term_id, tb.resource_type, COUNT(*) AS usage_count, MAX(tb.updated_at) AS last_used_at, datetime('now')
        FROM taxonomy_bindings tb
        JOIN taxonomy_terms tt ON tt.id = tb.term_id
        WHERE tb.is_deleted = 0
          AND tt.is_deleted = 0
        GROUP BY tb.term_id, tb.resource_type;

        DROP INDEX IF EXISTS idx_applications_owner;
        DROP TABLE IF EXISTS application_owners;
        ALTER TABLE applications DROP COLUMN owner;
    
"#;
