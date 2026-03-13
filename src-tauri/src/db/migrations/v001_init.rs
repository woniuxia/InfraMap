/// Consolidated initial schema (v001-v023 merged into single migration).
/// This creates the final database schema directly, skipping all intermediate steps.
pub const SQL: &str = r#"
-- ============================================================
-- Core resource tables
-- ============================================================

CREATE TABLE IF NOT EXISTS hosts (
    id TEXT PRIMARY KEY,
    hostname TEXT NOT NULL,
    os_type TEXT,
    cpu_info TEXT,
    cpu_model TEXT,
    cpu_cores INTEGER,
    cpu_threads INTEGER,
    cpu_freq TEXT,
    ram_gb INTEGER,
    disk_gb INTEGER,
    env TEXT NOT NULL DEFAULT 'prod',
    status TEXT NOT NULL DEFAULT 'running',
    tags TEXT,
    description TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS applications (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    address TEXT,
    port INTEGER,
    tech_stack TEXT,
    deploy_mode TEXT,
    env TEXT NOT NULL,
    git_repo TEXT,
    status TEXT NOT NULL DEFAULT 'running',
    description TEXT,
    business_application_id TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS middlewares (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    type TEXT NOT NULL,
    address TEXT NOT NULL,
    port INTEGER,
    version TEXT,
    env TEXT NOT NULL,
    description TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS nginx_configs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    env TEXT NOT NULL,
    strategy TEXT,
    status TEXT NOT NULL DEFAULT 'running',
    description TEXT,
    endpoints TEXT NOT NULL DEFAULT '[]',
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS business_applications (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    code TEXT,
    description TEXT,
    env TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- ============================================================
-- IP address management
-- ============================================================

CREATE TABLE IF NOT EXISTS ip_addresses (
    id TEXT PRIMARY KEY,
    ip_address TEXT NOT NULL,
    env TEXT NOT NULL DEFAULT 'prod',
    is_vip INTEGER NOT NULL DEFAULT 0,
    real_ips TEXT,
    description TEXT,
    tags TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS host_ip_bindings (
    id TEXT PRIMARY KEY,
    host_id TEXT NOT NULL,
    ip_id TEXT NOT NULL,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- ============================================================
-- Relationships
-- ============================================================

CREATE TABLE IF NOT EXISTS deployments (
    id TEXT PRIMARY KEY,
    resource_id TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    host_id TEXT NOT NULL,
    port INTEGER,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS call_relations (
    id TEXT PRIMARY KEY,
    pair_key TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    owner_type TEXT NOT NULL,
    peer_id TEXT NOT NULL,
    peer_type TEXT NOT NULL,
    direction TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    description TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- ============================================================
-- Contacts and owner relationships
-- ============================================================

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

CREATE TABLE IF NOT EXISTS application_owner_contacts (
    id TEXT PRIMARY KEY,
    application_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS business_application_owner_contacts (
    id TEXT PRIMARY KEY,
    business_application_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- ============================================================
-- Taxonomy (v2 schema)
-- ============================================================

CREATE TABLE IF NOT EXISTS taxonomy_terms (
    id TEXT PRIMARY KEY,
    field_key TEXT NOT NULL,
    normalized_value TEXT NOT NULL,
    display_name TEXT NOT NULL,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS taxonomy_bindings (
    id TEXT PRIMARY KEY,
    term_id TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS taxonomy_term_stats (
    term_id TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    usage_count INTEGER NOT NULL DEFAULT 0,
    last_used_at TEXT,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (term_id, resource_type)
);

-- ============================================================
-- Audit and system
-- ============================================================

CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    resource_name TEXT,
    changes TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS system_settings (
    id TEXT PRIMARY KEY,
    auto_backup_enabled INTEGER NOT NULL DEFAULT 0,
    backup_interval_hours INTEGER NOT NULL DEFAULT 24,
    max_backups INTEGER NOT NULL DEFAULT 10,
    last_backup_time TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS import_jobs (
    id TEXT PRIMARY KEY,
    status TEXT NOT NULL,
    strategy TEXT NOT NULL,
    total_rows INTEGER NOT NULL DEFAULT 0,
    created_count INTEGER NOT NULL DEFAULT 0,
    updated_count INTEGER NOT NULL DEFAULT 0,
    skipped_count INTEGER NOT NULL DEFAULT 0,
    failed_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS import_job_rows (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    row_no INTEGER NOT NULL,
    resource_type TEXT NOT NULL,
    name TEXT NOT NULL,
    env TEXT NOT NULL,
    status TEXT NOT NULL,
    error_message TEXT,
    payload_json TEXT NOT NULL,
    normalized_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS import_job_issues (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    row_no INTEGER NOT NULL,
    field_key TEXT,
    issue_type TEXT NOT NULL,
    code TEXT NOT NULL,
    message TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS system_jobs (
    id TEXT PRIMARY KEY,
    job_type TEXT NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    summary TEXT,
    progress_percent REAL NOT NULL DEFAULT 100,
    retryable INTEGER NOT NULL DEFAULT 0,
    cancellable INTEGER NOT NULL DEFAULT 0,
    payload_json TEXT NOT NULL DEFAULT '{}',
    result_json TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    finished_at TEXT
);

-- ============================================================
-- Indexes: hosts
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_hosts_hostname ON hosts(hostname) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_hosts_status ON hosts(status) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_hosts_env ON hosts(env) WHERE is_deleted = 0;

-- ============================================================
-- Indexes: applications
-- ============================================================
CREATE UNIQUE INDEX IF NOT EXISTS uk_applications_name_env_type ON applications(name, env, type) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_applications_type ON applications(type) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_applications_env ON applications(env) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_applications_status ON applications(status) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_applications_business_application_id ON applications(business_application_id) WHERE is_deleted = 0;

-- ============================================================
-- Indexes: middlewares
-- ============================================================
CREATE UNIQUE INDEX IF NOT EXISTS uk_middlewares_name_env ON middlewares(name, env) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_middlewares_category ON middlewares(category) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_middlewares_type ON middlewares(type) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_middlewares_env ON middlewares(env) WHERE is_deleted = 0;

-- ============================================================
-- Indexes: nginx_configs
-- ============================================================
CREATE UNIQUE INDEX IF NOT EXISTS uk_nginx_configs_name_env ON nginx_configs(name, env) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_nginx_configs_env ON nginx_configs(env) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_nginx_configs_status ON nginx_configs(status) WHERE is_deleted = 0;

-- ============================================================
-- Indexes: business_applications
-- ============================================================
CREATE UNIQUE INDEX IF NOT EXISTS uk_business_applications_name_env ON business_applications(name, env) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_business_applications_status ON business_applications(status) WHERE is_deleted = 0;

-- ============================================================
-- Indexes: ip_addresses
-- ============================================================
CREATE UNIQUE INDEX IF NOT EXISTS uk_ip_addresses_ip_env ON ip_addresses(ip_address, env) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_ip_addresses_ip ON ip_addresses(ip_address) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_ip_addresses_env ON ip_addresses(env) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_ip_addresses_is_vip ON ip_addresses(is_vip) WHERE is_deleted = 0;

-- ============================================================
-- Indexes: host_ip_bindings
-- ============================================================
CREATE UNIQUE INDEX IF NOT EXISTS uk_host_ip_bindings_host_ip ON host_ip_bindings(host_id, ip_id) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_host_ip_bindings_host ON host_ip_bindings(host_id) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_host_ip_bindings_ip ON host_ip_bindings(ip_id) WHERE is_deleted = 0;

-- ============================================================
-- Indexes: deployments
-- ============================================================
CREATE UNIQUE INDEX IF NOT EXISTS uk_deployments_resource_host ON deployments(resource_id, resource_type, host_id) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_deployments_host_id ON deployments(host_id) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_deployments_resource ON deployments(resource_id, resource_type) WHERE is_deleted = 0;

-- ============================================================
-- Indexes: call_relations
-- ============================================================
CREATE UNIQUE INDEX IF NOT EXISTS uk_call_relations_owner_peer_direction_type ON call_relations(owner_id, owner_type, peer_id, peer_type, direction, relation_type) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_call_relations_owner ON call_relations(owner_id, owner_type) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_call_relations_peer ON call_relations(peer_id, peer_type) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_call_relations_pair ON call_relations(pair_key) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_call_relations_relation_type ON call_relations(relation_type) WHERE is_deleted = 0;

-- ============================================================
-- Indexes: contacts
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_contacts_name ON contacts(name) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_contacts_company ON contacts(company) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_contacts_phone ON contacts(phone) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_contacts_email ON contacts(email) WHERE is_deleted = 0;

-- ============================================================
-- Indexes: application_owner_contacts
-- ============================================================
CREATE UNIQUE INDEX IF NOT EXISTS uk_application_owner_contacts_app_contact ON application_owner_contacts(application_id, contact_id) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_application_owner_contacts_application ON application_owner_contacts(application_id) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_application_owner_contacts_contact ON application_owner_contacts(contact_id) WHERE is_deleted = 0;

-- ============================================================
-- Indexes: business_application_owner_contacts
-- ============================================================
CREATE UNIQUE INDEX IF NOT EXISTS uk_business_application_owner_contacts_app_contact ON business_application_owner_contacts(business_application_id, contact_id) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_business_application_owner_contacts_business_application ON business_application_owner_contacts(business_application_id) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_business_application_owner_contacts_contact ON business_application_owner_contacts(contact_id) WHERE is_deleted = 0;

-- ============================================================
-- Indexes: taxonomy (v2)
-- ============================================================
CREATE UNIQUE INDEX IF NOT EXISTS uk_taxonomy_terms_field_normalized ON taxonomy_terms(field_key, normalized_value) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_taxonomy_terms_display_name ON taxonomy_terms(display_name) WHERE is_deleted = 0;

CREATE UNIQUE INDEX IF NOT EXISTS uk_taxonomy_bindings_term_resource ON taxonomy_bindings(term_id, resource_type, resource_id) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_taxonomy_bindings_resource ON taxonomy_bindings(resource_type, resource_id) WHERE is_deleted = 0;
CREATE INDEX IF NOT EXISTS idx_taxonomy_bindings_term ON taxonomy_bindings(term_id) WHERE is_deleted = 0;

CREATE INDEX IF NOT EXISTS idx_taxonomy_term_stats_recent ON taxonomy_term_stats(resource_type, last_used_at DESC);

-- ============================================================
-- Indexes: audit_logs
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource ON audit_logs(resource_type, resource_id);

-- ============================================================
-- Indexes: import_jobs
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_import_jobs_created_at ON import_jobs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_import_jobs_status ON import_jobs(status);

CREATE UNIQUE INDEX IF NOT EXISTS uk_import_job_rows_job_row ON import_job_rows(job_id, row_no);
CREATE INDEX IF NOT EXISTS idx_import_job_rows_job ON import_job_rows(job_id);

CREATE INDEX IF NOT EXISTS idx_import_job_issues_job ON import_job_issues(job_id);
CREATE INDEX IF NOT EXISTS idx_import_job_issues_job_row ON import_job_issues(job_id, row_no);

-- ============================================================
-- Indexes: system_jobs
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_system_jobs_created_at ON system_jobs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_system_jobs_status ON system_jobs(status);
CREATE INDEX IF NOT EXISTS idx_system_jobs_type ON system_jobs(job_type);
"#;
