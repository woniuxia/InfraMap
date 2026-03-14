/// Migration v024: Rename business_applications → systems, applications → services.
/// Also updates all related tables, indexes, and resource_type references.
pub const SQL: &str = r#"
-- ============================================================
-- Step 1: Create systems table (from business_applications)
-- ============================================================
CREATE TABLE IF NOT EXISTS systems (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    code TEXT,
    description TEXT,
    env TEXT NOT NULL DEFAULT 'prod',
    status TEXT NOT NULL DEFAULT 'active',
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Migrate data from business_applications to systems
INSERT INTO systems (id, name, code, description, env, status, is_deleted, deleted_at, created_at, updated_at)
SELECT id, name, code, description, COALESCE(NULLIF(TRIM(env), ''), 'prod'), status, is_deleted, deleted_at, created_at, updated_at
FROM business_applications;

-- ============================================================
-- Step 2: Auto-create systems for orphan applications
-- ============================================================
-- For applications without a business_application_id, group by (name, env) and create a system.
-- Use a CTE to generate system records, then insert unique ones.
INSERT INTO systems (id, name, code, description, env, status, is_deleted, deleted_at, created_at, updated_at)
SELECT
    ('auto-sys-' || REPLACE(LOWER(HEX(RANDOMBLOB(8))), '00', 'ff')) AS id,
    a.name AS name,
    NULL AS code,
    NULL AS description,
    a.env AS env,
    'active' AS status,
    0 AS is_deleted,
    NULL AS deleted_at,
    a.created_at AS created_at,
    a.updated_at AS updated_at
FROM applications a
WHERE a.is_deleted = 0
  AND (a.business_application_id IS NULL OR TRIM(a.business_application_id) = '')
  AND NOT EXISTS (
    SELECT 1 FROM systems s WHERE s.name = a.name AND s.env = a.env AND s.is_deleted = 0
  )
GROUP BY a.name, a.env;

-- ============================================================
-- Step 3: Create services table (from applications)
-- ============================================================
CREATE TABLE IF NOT EXISTS services (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    address TEXT,
    port INTEGER,
    tech_stack TEXT,
    deploy_mode TEXT,
    env TEXT NOT NULL,
    git_repo TEXT,
    system_id TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'running',
    description TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Migrate data: applications with business_application_id → use it as system_id
INSERT INTO services (id, name, type, address, port, tech_stack, deploy_mode, env, git_repo, system_id, status, description, is_deleted, deleted_at, created_at, updated_at)
SELECT
    a.id, a.name, a.type, a.address, a.port, a.tech_stack, a.deploy_mode, a.env, a.git_repo,
    CASE
        WHEN a.business_application_id IS NOT NULL AND TRIM(a.business_application_id) != ''
            THEN a.business_application_id
        ELSE COALESCE(
            (SELECT s.id FROM systems s WHERE s.name = a.name AND s.env = a.env AND s.is_deleted = 0 LIMIT 1),
            ''
        )
    END AS system_id,
    a.status, a.description, a.is_deleted, a.deleted_at, a.created_at, a.updated_at
FROM applications a;

-- ============================================================
-- Step 4: Create system_owner_contacts (from business_application_owner_contacts)
-- ============================================================
CREATE TABLE IF NOT EXISTS system_owner_contacts (
    id TEXT PRIMARY KEY,
    system_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

INSERT INTO system_owner_contacts (id, system_id, contact_id, is_deleted, deleted_at, created_at, updated_at)
SELECT id, business_application_id, contact_id, is_deleted, deleted_at, created_at, updated_at
FROM business_application_owner_contacts;

-- ============================================================
-- Step 5: Create service_owner_contacts (from application_owner_contacts)
-- ============================================================
CREATE TABLE IF NOT EXISTS service_owner_contacts (
    id TEXT PRIMARY KEY,
    service_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

INSERT INTO service_owner_contacts (id, service_id, contact_id, is_deleted, deleted_at, created_at, updated_at)
SELECT id, application_id, contact_id, is_deleted, deleted_at, created_at, updated_at
FROM application_owner_contacts;

-- ============================================================
-- Step 6: Update resource_type references in related tables
-- ============================================================

-- deployments
UPDATE deployments SET resource_type = 'service' WHERE resource_type = 'application';

-- call_relations
UPDATE call_relations SET owner_type = 'service' WHERE owner_type = 'application';
UPDATE call_relations SET peer_type = 'service' WHERE peer_type = 'application';

-- audit_logs
UPDATE audit_logs SET resource_type = 'service' WHERE resource_type = 'application';
UPDATE audit_logs SET resource_type = 'system' WHERE resource_type = 'business_application';

-- taxonomy_bindings
UPDATE taxonomy_bindings SET resource_type = 'service' WHERE resource_type = 'application';
UPDATE taxonomy_bindings SET resource_type = 'system' WHERE resource_type = 'business_application';

-- import_job_rows
UPDATE import_job_rows SET resource_type = 'service' WHERE resource_type = 'application';

-- ============================================================
-- Step 7: Drop old tables
-- ============================================================
DROP TABLE IF EXISTS application_owner_contacts;
DROP TABLE IF EXISTS business_application_owner_contacts;
DROP TABLE IF EXISTS applications;
DROP TABLE IF EXISTS business_applications;

-- ============================================================
-- Step 8: Create indexes for new tables
-- ============================================================

-- systems indexes
CREATE UNIQUE INDEX IF NOT EXISTS uk_systems_name_env ON systems(name, env) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_systems_status ON systems(status) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_systems_env ON systems(env) WHERE is_deleted = 0;

-- services indexes (no unique constraint on name+env+type per plan)
CREATE INDEX IF NOT EXISTS idx_services_type ON services(type) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_services_env ON services(env) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_services_status ON services(status) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_services_system_id ON services(system_id) WHERE is_deleted = 0;

-- system_owner_contacts indexes
CREATE UNIQUE INDEX IF NOT EXISTS uk_system_owner_contacts_sys_contact ON system_owner_contacts(system_id, contact_id) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_system_owner_contacts_system ON system_owner_contacts(system_id) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_system_owner_contacts_contact ON system_owner_contacts(contact_id) WHERE is_deleted = 0;

-- service_owner_contacts indexes
CREATE UNIQUE INDEX IF NOT EXISTS uk_service_owner_contacts_svc_contact ON service_owner_contacts(service_id, contact_id) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_service_owner_contacts_service ON service_owner_contacts(service_id) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_service_owner_contacts_contact ON service_owner_contacts(contact_id) WHERE is_deleted = 0;
"#;
