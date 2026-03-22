use chrono::{DateTime, Utc};
use executors::actions::{ExecutorAction, ExecutorActionType};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

/// Maximum length for auto-generated workspace names (derived from first user prompt)
const WORKSPACE_NAME_MAX_LEN: usize = 60;

use super::{
    execution_process::ExecutorActionField,
    session::Session,
    workspace_repo::{RepoWithTargetBranch, WorkspaceRepo},
};

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("Workspace not found")]
    WorkspaceNotFound,
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Branch not found: {0}")]
    BranchNotFound(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct ContainerInfo {
    pub workspace_id: Uuid,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct Workspace {
    pub id: Uuid,
    pub task_id: Option<Uuid>,
    pub container_ref: Option<String>,
    pub branch: String,
    pub setup_completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived: bool,
    pub pinned: bool,
    pub name: Option<String>,
    pub worktree_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct WorkspaceWithStatus {
    #[serde(flatten)]
    #[ts(flatten)]
    pub workspace: Workspace,
    pub is_running: bool,
    pub is_errored: bool,
}

impl std::ops::Deref for WorkspaceWithStatus {
    type Target = Workspace;
    fn deref(&self) -> &Self::Target {
        &self.workspace
    }
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateFollowUpAttempt {
    pub prompt: String,
}

/// Context data for resume operations (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptResumeContext {
    pub execution_history: String,
    pub cumulative_diffs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceContext {
    pub workspace: Workspace,
    pub workspace_repos: Vec<RepoWithTargetBranch>,
    pub orchestrator_session_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateWorkspace {
    pub branch: String,
    pub name: Option<String>,
}

impl Workspace {
    pub async fn fetch_all(pool: &SqlitePool) -> Result<Vec<Self>, WorkspaceError> {
        let rows = sqlx::query_as::<_, (Uuid, Option<Uuid>, Option<String>, String, Option<DateTime<Utc>>, DateTime<Utc>, DateTime<Utc>, bool, bool, Option<String>, bool)>(
            r#"SELECT id, task_id, container_ref, branch, setup_completed_at, created_at, updated_at, archived, pinned, name, worktree_deleted
               FROM workspaces
               ORDER BY created_at DESC"#
        )
        .fetch_all(pool)
        .await
        .map_err(WorkspaceError::Database)?;

        Ok(rows.into_iter().map(|r| Workspace {
            id: r.0,
            task_id: r.1,
            container_ref: r.2,
            branch: r.3,
            setup_completed_at: r.4,
            created_at: r.5,
            updated_at: r.6,
            archived: r.7,
            pinned: r.8,
            name: r.9,
            worktree_deleted: r.10,
        }).collect())
    }

    /// Load full workspace context by workspace ID.
    pub async fn load_context(
        pool: &SqlitePool,
        workspace_id: Uuid,
    ) -> Result<WorkspaceContext, WorkspaceError> {
        let workspace = Workspace::find_by_id(pool, workspace_id)
            .await?
            .ok_or(WorkspaceError::WorkspaceNotFound)?;

        let workspace_repos =
            WorkspaceRepo::find_repos_with_target_branch_for_workspace(pool, workspace_id).await?;
        let orchestrator_session_id = Session::find_first_by_workspace_id(pool, workspace_id)
            .await?
            .map(|session| session.id);

        Ok(WorkspaceContext {
            workspace,
            workspace_repos,
            orchestrator_session_id,
        })
    }

    pub async fn update_container_ref(
        pool: &SqlitePool,
        workspace_id: Uuid,
        container_ref: &str,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            "UPDATE workspaces SET container_ref = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(container_ref)
        .bind(now)
        .bind(workspace_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn mark_worktree_deleted(
        pool: &SqlitePool,
        workspace_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE workspaces SET worktree_deleted = 1, updated_at = datetime('now', 'subsec') WHERE id = $1"
        )
        .bind(workspace_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn clear_worktree_deleted(
        pool: &SqlitePool,
        workspace_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE workspaces SET worktree_deleted = 0, updated_at = datetime('now', 'subsec') WHERE id = $1"
        )
        .bind(workspace_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn touch(pool: &SqlitePool, workspace_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE workspaces SET updated_at = datetime('now', 'subsec') WHERE id = $1"
        )
        .bind(workspace_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query_as::<_, (Uuid, Option<Uuid>, Option<String>, String, Option<DateTime<Utc>>, DateTime<Utc>, DateTime<Utc>, bool, bool, Option<String>, bool)>(
            r#"SELECT id, task_id, container_ref, branch, setup_completed_at, created_at, updated_at, archived, pinned, name, worktree_deleted
               FROM workspaces WHERE id = $1"#
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| Workspace {
            id: r.0,
            task_id: r.1,
            container_ref: r.2,
            branch: r.3,
            setup_completed_at: r.4,
            created_at: r.5,
            updated_at: r.6,
            archived: r.7,
            pinned: r.8,
            name: r.9,
            worktree_deleted: r.10,
        }))
    }

    pub async fn find_by_rowid(pool: &SqlitePool, rowid: i64) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query_as::<_, (Uuid, Option<Uuid>, Option<String>, String, Option<DateTime<Utc>>, DateTime<Utc>, DateTime<Utc>, bool, bool, Option<String>, bool)>(
            r#"SELECT id, task_id, container_ref, branch, setup_completed_at, created_at, updated_at, archived, pinned, name, worktree_deleted
               FROM workspaces WHERE rowid = $1"#
        )
        .bind(rowid)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| Workspace {
            id: r.0,
            task_id: r.1,
            container_ref: r.2,
            branch: r.3,
            setup_completed_at: r.4,
            created_at: r.5,
            updated_at: r.6,
            archived: r.7,
            pinned: r.8,
            name: r.9,
            worktree_deleted: r.10,
        }))
    }

    pub async fn container_ref_exists(
        pool: &SqlitePool,
        container_ref: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM workspaces WHERE container_ref = $1"#
        )
        .bind(container_ref)
        .fetch_one(pool)
        .await?;

        Ok(result > 0)
    }

    pub async fn find_expired_for_cleanup(
        pool: &SqlitePool,
    ) -> Result<Vec<Workspace>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (Uuid, Option<Uuid>, Option<String>, String, Option<DateTime<Utc>>, DateTime<Utc>, DateTime<Utc>, bool, bool, Option<String>, bool)>(
            r#"
            SELECT
                w.id, w.task_id, w.container_ref, w.branch, w.setup_completed_at,
                w.created_at, w.updated_at, w.archived, w.pinned, w.name, w.worktree_deleted
            FROM workspaces w
            LEFT JOIN sessions s ON w.id = s.workspace_id
            LEFT JOIN execution_processes ep ON s.id = ep.session_id AND ep.completed_at IS NOT NULL
            WHERE w.container_ref IS NOT NULL
                AND w.worktree_deleted = 0
                AND w.id NOT IN (
                    SELECT DISTINCT s2.workspace_id
                    FROM sessions s2
                    JOIN execution_processes ep2 ON s2.id = ep2.session_id
                    WHERE ep2.completed_at IS NULL
                )
            GROUP BY w.id, w.container_ref, w.updated_at
            HAVING datetime('now', 'localtime',
                CASE
                    WHEN w.archived = 1
                    THEN '-1 hours'
                    ELSE '-72 hours'
                END
            ) > datetime(
                MAX(
                    max(
                        datetime(w.updated_at),
                        datetime(ep.completed_at)
                    )
                )
            )
            ORDER BY MAX(
                CASE
                    WHEN ep.completed_at IS NOT NULL THEN ep.completed_at
                    ELSE w.updated_at
                END
            ) ASC
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| Workspace {
            id: r.0,
            task_id: r.1,
            container_ref: r.2,
            branch: r.3,
            setup_completed_at: r.4,
            created_at: r.5,
            updated_at: r.6,
            archived: r.7,
            pinned: r.8,
            name: r.9,
            worktree_deleted: r.10,
        }).collect())
    }

    pub async fn create(
        pool: &SqlitePool,
        data: &CreateWorkspace,
        id: Uuid,
    ) -> Result<Self, WorkspaceError> {
        sqlx::query(
            r#"INSERT INTO workspaces (id, task_id, container_ref, branch, setup_completed_at, name, created_at, updated_at, archived, pinned, worktree_deleted)
               VALUES ($1, $2, $3, $4, $5, $6, datetime('now', 'subsec'), datetime('now', 'subsec'), 0, 0, 0)"#
        )
        .bind(id)
        .bind(Option::<Uuid>::None)
        .bind(Option::<String>::None)
        .bind(&data.branch)
        .bind(Option::<DateTime<Utc>>::None)
        .bind(&data.name)
        .execute(pool)
        .await
        .map_err(WorkspaceError::Database)?;

        Self::find_by_id(pool, id)
            .await
            .map_err(WorkspaceError::Database)?
            .ok_or(WorkspaceError::WorkspaceNotFound)
    }

    pub async fn update_branch_name(
        pool: &SqlitePool,
        workspace_id: Uuid,
        new_branch_name: &str,
    ) -> Result<(), WorkspaceError> {
        sqlx::query(
            "UPDATE workspaces SET branch = $1, updated_at = datetime('now', 'subsec') WHERE id = $2"
        )
        .bind(new_branch_name)
        .bind(workspace_id)
        .execute(pool)
        .await
        .map_err(WorkspaceError::Database)?;

        Ok(())
    }

    pub async fn resolve_container_ref(
        pool: &SqlitePool,
        container_ref: &str,
    ) -> Result<ContainerInfo, sqlx::Error> {
        let workspace_id = sqlx::query_scalar::<_, Uuid>(
            r#"SELECT id FROM workspaces WHERE container_ref = $1"#
        )
        .bind(container_ref)
        .fetch_optional(pool)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

        Ok(ContainerInfo { workspace_id })
    }

    pub async fn resolve_container_ref_by_prefix(
        pool: &SqlitePool,
        path: &str,
    ) -> Result<ContainerInfo, sqlx::Error> {
        let workspaces = sqlx::query_as::<_, (Uuid, String)>(
            r#"SELECT id, container_ref FROM workspaces WHERE container_ref IS NOT NULL"#
        )
        .fetch_all(pool)
        .await?;

        Self::best_matching_container_ref(
            path,
            workspaces.iter().map(|ws| (ws.0, ws.1.as_str())),
        )
        .map(|workspace_id| ContainerInfo { workspace_id })
        .ok_or(sqlx::Error::RowNotFound)
    }

    fn best_matching_container_ref<'a>(
        path: &str,
        candidates: impl Iterator<Item = (Uuid, &'a str)>,
    ) -> Option<Uuid> {
        let path = std::path::Path::new(path);

        candidates
            .filter(|(_, container_ref)| {
                let container_ref = std::path::Path::new(container_ref);
                path.starts_with(container_ref) || container_ref.starts_with(path)
            })
            .max_by_key(|(_, container_ref)| {
                std::path::Path::new(container_ref).components().count()
            })
            .map(|(workspace_id, _)| workspace_id)
    }

    pub async fn set_archived(
        pool: &SqlitePool,
        workspace_id: Uuid,
        archived: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE workspaces SET archived = $1, updated_at = datetime('now', 'subsec') WHERE id = $2"
        )
        .bind(archived)
        .bind(workspace_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn update(
        pool: &SqlitePool,
        workspace_id: Uuid,
        archived: Option<bool>,
        pinned: Option<bool>,
        name: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let name_value = name.filter(|s| !s.is_empty());
        let name_provided = name.is_some();

        sqlx::query(
            r#"UPDATE workspaces SET
                archived = COALESCE($1, archived),
                pinned = COALESCE($2, pinned),
                name = CASE WHEN $3 THEN $4 ELSE name END,
                updated_at = datetime('now', 'subsec')
            WHERE id = $5"#
        )
        .bind(archived)
        .bind(pinned)
        .bind(name_provided)
        .bind(name_value)
        .bind(workspace_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_first_user_message(
        pool: &SqlitePool,
        workspace_id: Uuid,
    ) -> Result<Option<String>, sqlx::Error> {
        let actions = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT ep.executor_action
               FROM sessions s
               JOIN execution_processes ep ON ep.session_id = s.id
               WHERE s.workspace_id = $1
               ORDER BY s.created_at ASC, ep.created_at ASC"#
        )
        .bind(workspace_id)
        .fetch_all(pool)
        .await?;

        for action_value in actions {
            if let Ok(action_field) = serde_json::from_value::<ExecutorActionField>(action_value) {
                if let ExecutorActionField::ExecutorAction(action) = action_field
                    && let Some(prompt) = Self::extract_first_prompt_from_executor_action(&action)
                {
                    return Ok(Some(prompt));
                }
            }
        }

        Ok(None)
    }

    fn extract_first_prompt_from_executor_action(action: &ExecutorAction) -> Option<String> {
        let mut current = Some(action);
        while let Some(action) = current {
            match action.typ() {
                ExecutorActionType::CodingAgentInitialRequest(request) => {
                    return Some(request.prompt.clone());
                }
                ExecutorActionType::CodingAgentFollowUpRequest(request) => {
                    return Some(request.prompt.clone());
                }
                ExecutorActionType::ReviewRequest(request) => {
                    return Some(request.prompt.clone());
                }
                ExecutorActionType::ScriptRequest(_) => {
                    current = action.next_action();
                }
            }
        }
        None
    }

    pub fn truncate_to_name(prompt: &str, max_len: usize) -> String {
        let trimmed = prompt.trim();
        if trimmed.chars().count() <= max_len {
            trimmed.to_string()
        } else {
            let truncated: String = trimmed.chars().take(max_len).collect();
            if let Some(last_space) = truncated.rfind(' ') {
                format!("{}...", &truncated[..last_space])
            } else {
                format!("{}...", truncated)
            }
        }
    }

    pub async fn find_all_with_status(
        pool: &SqlitePool,
        archived: Option<bool>,
        limit: Option<i64>,
    ) -> Result<Vec<WorkspaceWithStatus>, sqlx::Error> {
        let records = sqlx::query_as::<_, (Uuid, Option<Uuid>, Option<String>, String, Option<DateTime<Utc>>, DateTime<Utc>, DateTime<Utc>, bool, bool, Option<String>, bool, i64, i64)>(
            r#"SELECT
                w.id, w.task_id, w.container_ref, w.branch, w.setup_completed_at,
                w.created_at, w.updated_at, w.archived, w.pinned, w.name, w.worktree_deleted,

                CASE WHEN EXISTS (
                    SELECT 1
                    FROM sessions s
                    JOIN execution_processes ep ON ep.session_id = s.id
                    WHERE s.workspace_id = w.id
                      AND ep.status = 'running'
                      AND ep.run_reason IN ('setupscript','cleanupscript','codingagent')
                    LIMIT 1
                ) THEN 1 ELSE 0 END,

                CASE WHEN (
                    SELECT ep.status
                    FROM sessions s
                    JOIN execution_processes ep ON ep.session_id = s.id
                    WHERE s.workspace_id = w.id
                      AND ep.run_reason IN ('setupscript','cleanupscript','codingagent')
                    ORDER BY ep.created_at DESC
                    LIMIT 1
                ) IN ('failed','killed') THEN 1 ELSE 0 END

            FROM workspaces w
            ORDER BY w.updated_at DESC"#
        )
        .fetch_all(pool)
        .await?;

        let mut workspaces: Vec<WorkspaceWithStatus> = records
            .into_iter()
            .map(|rec| WorkspaceWithStatus {
                workspace: Workspace {
                    id: rec.0,
                    task_id: rec.1,
                    container_ref: rec.2,
                    branch: rec.3,
                    setup_completed_at: rec.4,
                    created_at: rec.5,
                    updated_at: rec.6,
                    archived: rec.7,
                    pinned: rec.8,
                    name: rec.9,
                    worktree_deleted: rec.10,
                },
                is_running: rec.11 != 0,
                is_errored: rec.12 != 0,
            })
            .filter(|ws| archived.is_none_or(|a| ws.workspace.archived == a))
            .collect();

        if let Some(lim) = limit {
            workspaces.truncate(lim as usize);
        }

        for ws in &mut workspaces {
            if ws.workspace.name.is_none()
                && let Some(prompt) = Self::get_first_user_message(pool, ws.workspace.id).await?
            {
                let name = Self::truncate_to_name(&prompt, WORKSPACE_NAME_MAX_LEN);
                Self::update(pool, ws.workspace.id, None, None, Some(&name)).await?;
                ws.workspace.name = Some(name);
            }
        }

        Ok(workspaces)
    }

    pub async fn delete(pool: &SqlitePool, id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM workspaces WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn count_all(pool: &SqlitePool) -> Result<i64, WorkspaceError> {
        sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM workspaces"#)
            .fetch_one(pool)
            .await
            .map_err(WorkspaceError::Database)
    }

    pub async fn find_by_id_with_status(
        pool: &SqlitePool,
        id: Uuid,
    ) -> Result<Option<WorkspaceWithStatus>, sqlx::Error> {
        let rec = sqlx::query_as::<_, (Uuid, Option<Uuid>, Option<String>, String, Option<DateTime<Utc>>, DateTime<Utc>, DateTime<Utc>, bool, bool, Option<String>, bool, i64, i64)>(
            r#"SELECT
                w.id, w.task_id, w.container_ref, w.branch, w.setup_completed_at,
                w.created_at, w.updated_at, w.archived, w.pinned, w.name, w.worktree_deleted,

                CASE WHEN EXISTS (
                    SELECT 1
                    FROM sessions s
                    JOIN execution_processes ep ON ep.session_id = s.id
                    WHERE s.workspace_id = w.id
                      AND ep.status = 'running'
                      AND ep.run_reason IN ('setupscript','cleanupscript','codingagent')
                    LIMIT 1
                ) THEN 1 ELSE 0 END,

                CASE WHEN (
                    SELECT ep.status
                    FROM sessions s
                    JOIN execution_processes ep ON ep.session_id = s.id
                    WHERE s.workspace_id = w.id
                      AND ep.run_reason IN ('setupscript','cleanupscript','codingagent')
                    ORDER BY ep.created_at DESC
                    LIMIT 1
                ) IN ('failed','killed') THEN 1 ELSE 0 END

            FROM workspaces w
            WHERE w.id = $1"#
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        let Some(rec) = rec else {
            return Ok(None);
        };

        let mut ws = WorkspaceWithStatus {
            workspace: Workspace {
                id: rec.0,
                task_id: rec.1,
                container_ref: rec.2,
                branch: rec.3,
                setup_completed_at: rec.4,
                created_at: rec.5,
                updated_at: rec.6,
                archived: rec.7,
                pinned: rec.8,
                name: rec.9,
                worktree_deleted: rec.10,
            },
            is_running: rec.11 != 0,
            is_errored: rec.12 != 0,
        };

        if ws.workspace.name.is_none()
            && let Some(prompt) = Self::get_first_user_message(pool, ws.workspace.id).await?
        {
            let name = Self::truncate_to_name(&prompt, WORKSPACE_NAME_MAX_LEN);
            Self::update(pool, ws.workspace.id, None, None, Some(&name)).await?;
            ws.workspace.name = Some(name);
        }

        Ok(Some(ws))
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::Workspace;

    #[test]
    fn best_matching_container_ref_prefers_deepest_match() {
        let broad_id = Uuid::new_v4();
        let exact_id = Uuid::new_v4();
        let selected = Workspace::best_matching_container_ref(
            "/tmp/ws/repo/packages/app",
            [(broad_id, "/tmp"), (exact_id, "/tmp/ws")].into_iter(),
        );

        assert_eq!(selected, Some(exact_id));
    }

    #[test]
    fn best_matching_container_ref_supports_parent_request_path() {
        let workspace_id = Uuid::new_v4();
        let selected = Workspace::best_matching_container_ref(
            "/tmp/ws/repo",
            [(workspace_id, "/tmp/ws/repo/packages/app")].into_iter(),
        );

        assert_eq!(selected, Some(workspace_id));
    }

    #[test]
    fn best_matching_container_ref_ignores_unrelated_paths() {
        let workspace_id = Uuid::new_v4();
        let selected = Workspace::best_matching_container_ref(
            "/tmp/other/path",
            [(workspace_id, "/tmp/ws")].into_iter(),
        );

        assert_eq!(selected, None);
    }
}
