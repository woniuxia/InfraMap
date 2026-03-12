pub const SQL: &str = r#"
        CREATE TABLE IF NOT EXISTS contacts (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            company TEXT,
            phone TEXT,
            email TEXT,
            remark TEXT,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_contacts_name ON contacts(name) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_contacts_company ON contacts(company) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_contacts_phone ON contacts(phone) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_contacts_email ON contacts(email) WHERE is_deleted = 0;

        CREATE TABLE IF NOT EXISTS application_owner_contacts (
            id TEXT PRIMARY KEY,
            application_id TEXT NOT NULL,
            contact_id TEXT NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS uk_application_owner_contacts_app_contact
        ON application_owner_contacts(application_id, contact_id) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_application_owner_contacts_application
        ON application_owner_contacts(application_id) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_application_owner_contacts_contact
        ON application_owner_contacts(contact_id) WHERE is_deleted = 0;

        CREATE TABLE IF NOT EXISTS business_application_owner_contacts (
            id TEXT PRIMARY KEY,
            business_application_id TEXT NOT NULL,
            contact_id TEXT NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS uk_business_application_owner_contacts_app_contact
        ON business_application_owner_contacts(business_application_id, contact_id) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_business_application_owner_contacts_business_application
        ON business_application_owner_contacts(business_application_id) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_business_application_owner_contacts_contact
        ON business_application_owner_contacts(contact_id) WHERE is_deleted = 0;

        -- Backfill contacts and owner relationships from legacy owner taxonomy
        INSERT OR IGNORE INTO contacts (id, name, phone, email, remark, is_deleted, deleted_at, created_at, updated_at)
        SELECT tt.id, tt.display_name, NULL, NULL, NULL, 0, NULL,
               COALESCE(tt.created_at, datetime('now')),
               COALESCE(tt.updated_at, datetime('now'))
          FROM taxonomy_terms tt
         WHERE tt.is_deleted = 0
           AND tt.field_key = 'owner'
           AND TRIM(COALESCE(tt.display_name, '')) <> '';

        INSERT OR IGNORE INTO application_owner_contacts (id, application_id, contact_id, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), tb.resource_id, tb.term_id, 0, NULL,
               COALESCE(tb.created_at, datetime('now')),
               COALESCE(tb.updated_at, datetime('now'))
          FROM taxonomy_bindings tb
          JOIN taxonomy_terms tt
            ON tt.id = tb.term_id
           AND tt.is_deleted = 0
           AND tt.field_key = 'owner'
          JOIN applications a
            ON a.id = tb.resource_id
           AND a.is_deleted = 0
         WHERE tb.resource_type = 'application'
           AND tb.is_deleted = 0;

        INSERT OR IGNORE INTO business_application_owner_contacts (id, business_application_id, contact_id, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), tb.resource_id, tb.term_id, 0, NULL,
               COALESCE(tb.created_at, datetime('now')),
               COALESCE(tb.updated_at, datetime('now'))
          FROM taxonomy_bindings tb
          JOIN taxonomy_terms tt
            ON tt.id = tb.term_id
           AND tt.is_deleted = 0
           AND tt.field_key = 'owner'
          JOIN business_applications ba
            ON ba.id = tb.resource_id
           AND ba.is_deleted = 0
         WHERE tb.resource_type = 'business_application'
           AND tb.is_deleted = 0;

"#;

