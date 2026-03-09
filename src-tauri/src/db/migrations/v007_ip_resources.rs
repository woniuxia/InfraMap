pub const SQL: &str = r#"
        CREATE TABLE IF NOT EXISTS ip_addresses (
            id TEXT PRIMARY KEY,
            ip_address TEXT NOT NULL,
            env TEXT NOT NULL DEFAULT 'prod',
            is_vip INTEGER NOT NULL DEFAULT 0,
            real_ips TEXT,
            description TEXT,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS uk_ip_addresses_ip_env
        ON ip_addresses(ip_address, env) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_ip_addresses_ip
        ON ip_addresses(ip_address) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_ip_addresses_env
        ON ip_addresses(env) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_ip_addresses_is_vip
        ON ip_addresses(is_vip) WHERE is_deleted = 0;

        DROP INDEX IF EXISTS uk_hosts_ip;

        CREATE TABLE IF NOT EXISTS host_ip_bindings (
            id TEXT PRIMARY KEY,
            host_id TEXT NOT NULL,
            ip_id TEXT NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS uk_host_ip_bindings_host_ip
        ON host_ip_bindings(host_id, ip_id) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_host_ip_bindings_host
        ON host_ip_bindings(host_id) WHERE is_deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_host_ip_bindings_ip
        ON host_ip_bindings(ip_id) WHERE is_deleted = 0;

        INSERT INTO ip_addresses (id, ip_address, env, is_vip, real_ips, description, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), h.ip_address, h.env, 0, NULL, NULL, 0, NULL, h.created_at, h.updated_at
        FROM hosts h
        WHERE h.is_deleted = 0
          AND TRIM(COALESCE(h.ip_address, '')) <> ''
          AND NOT EXISTS (
            SELECT 1 FROM ip_addresses ia
            WHERE ia.ip_address = h.ip_address
              AND ia.env = h.env
              AND ia.is_deleted = 0
          );

        INSERT INTO host_ip_bindings (id, host_id, ip_id, is_deleted, deleted_at, created_at, updated_at)
        SELECT lower(hex(randomblob(16))), h.id, ia.id, 0, NULL, h.created_at, h.updated_at
        FROM hosts h
        JOIN ip_addresses ia
          ON ia.ip_address = h.ip_address
         AND ia.env = h.env
         AND ia.is_deleted = 0
        LEFT JOIN host_ip_bindings hb
          ON hb.host_id = h.id
         AND hb.ip_id = ia.id
         AND hb.is_deleted = 0
        WHERE h.is_deleted = 0
          AND TRIM(COALESCE(h.ip_address, '')) <> ''
          AND hb.id IS NULL;
    
"#;
