-- Kanban tables for local persistence

-- Organizations table
CREATE TABLE IF NOT EXISTS organizations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
);

-- Projects table (renamed to avoid conflict with existing projects table)
CREATE TABLE IF NOT EXISTS kanban_projects (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT NOT NULL DEFAULT '#3b82f6',
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
);

CREATE INDEX IF NOT EXISTS idx_kanban_projects_org_id ON kanban_projects(organization_id);

-- Project statuses table
CREATE TABLE IF NOT EXISTS project_statuses (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT NOT NULL DEFAULT '220 9% 46%',
    sort_order INTEGER NOT NULL DEFAULT 0,
    hidden INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
);

CREATE INDEX IF NOT EXISTS idx_project_statuses_project_id ON project_statuses(project_id);

-- Tags table
CREATE TABLE IF NOT EXISTS kanban_tags (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT NOT NULL DEFAULT '220 9% 46%',
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
);

CREATE INDEX IF NOT EXISTS idx_kanban_tags_project_id ON kanban_tags(project_id);

-- Issues table
CREATE TABLE IF NOT EXISTS issues (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    issue_number INTEGER NOT NULL,
    simple_id TEXT NOT NULL,
    status_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    priority TEXT CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
    sort_order INTEGER NOT NULL DEFAULT 0,
    start_date TEXT,
    target_date TEXT,
    completed_at TEXT,
    parent_issue_id TEXT,
    parent_issue_sort_order INTEGER,
    extension_metadata TEXT DEFAULT '{}',
    creator_user_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
);

CREATE INDEX IF NOT EXISTS idx_issues_project_id ON issues(project_id);
CREATE INDEX IF NOT EXISTS idx_issues_status_id ON issues(status_id);

-- Issue assignees table
CREATE TABLE IF NOT EXISTS issue_assignees (
    id TEXT PRIMARY KEY,
    issue_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    assigned_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
);

CREATE INDEX IF NOT EXISTS idx_issue_assignees_issue_id ON issue_assignees(issue_id);

-- Issue followers table
CREATE TABLE IF NOT EXISTS issue_followers (
    id TEXT PRIMARY KEY,
    issue_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    project_id TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_issue_followers_issue_id ON issue_followers(issue_id);

-- Issue tags table
CREATE TABLE IF NOT EXISTS issue_tags (
    id TEXT PRIMARY KEY,
    issue_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    project_id TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_issue_tags_issue_id ON issue_tags(issue_id);
CREATE INDEX IF NOT EXISTS idx_issue_tags_tag_id ON issue_tags(tag_id);

-- Issue relationships table
CREATE TABLE IF NOT EXISTS issue_relationships (
    id TEXT PRIMARY KEY,
    issue_id TEXT NOT NULL,
    related_issue_id TEXT NOT NULL,
    relationship_type TEXT NOT NULL,
    project_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
);

CREATE INDEX IF NOT EXISTS idx_issue_relationships_issue_id ON issue_relationships(issue_id);
CREATE INDEX IF NOT EXISTS idx_issue_relationships_related_id ON issue_relationships(related_issue_id);

-- Test data for SQLx prepare (can be removed after prepare is complete)
INSERT INTO organizations (id, name) VALUES ('dummy-org', 'Dummy Organization');
INSERT INTO kanban_projects (id, organization_id, name, color) VALUES ('dummy-project', 'dummy-org', 'Dummy Project', '#3b82f6');
INSERT INTO project_statuses (id, project_id, name, color, hidden) VALUES ('dummy-status', 'dummy-project', 'Dummy', '220 9% 46%', 0);

