use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use executors::{
    actions::{ExecutorAction, ExecutorActionType},
    profile::ExecutorProfileId,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, SqlitePool, Type};
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

use super::{
    execution_process_repo_state::{CreateExecutionProcessRepoState, ExecutionProcessRepoState},
    repo::Repo,
    session::Session,
    workspace::Workspace,
    workspace_repo::WorkspaceRepo,
};

#[derive(Debug, Error)]
pub enum ExecutionProcessError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("Execution process not found")]
    ExecutionProcessNotFound,
    #[error("Failed to create execution process: {0}")]
    CreateFailed(String),
    #[error("Failed to update execution process: {0}")]
    UpdateFailed(String),
    #[error("Invalid executor action format")]
    InvalidExecutorAction,
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq, TS)]
#[sqlx(type_name = "execution_process_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[ts(use_ts_enum)]
pub enum ExecutionProcessStatus {
    Running,
    Completed,
    Failed,
    Killed,
}

#[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq, TS)]
#[sqlx(type_name = "execution_process_run_reason", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ExecutionProcessRunReason {
    SetupScript,
    CleanupScript,
    ArchiveScript,
    CodingAgent,
    DevServer,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct ExecutionProcess {
    pub id: Uuid,
    pub session_id: Uuid,
    pub run_reason: ExecutionProcessRunReason,
    #[ts(type = "ExecutorAction")]
    pub executor_action: sqlx::types::Json<ExecutorActionField>,
    pub status: ExecutionProcessStatus,
    pub exit_code: Option<i64>,
    pub dropped: bool,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateExecutionProcess {
    pub session_id: Uuid,
    pub executor_action: ExecutorAction,
    pub run_reason: ExecutionProcessRunReason,
}

#[derive(Debug, Deserialize, TS)]
#[allow(dead_code)]
pub struct UpdateExecutionProcess {
    pub status: Option<ExecutionProcessStatus>,
    pub exit_code: Option<i64>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct ExecutionContext {
    pub execution_process: ExecutionProcess,
    pub session: Session,
    pub workspace: Workspace,
    pub repos: Vec<Repo>,
}

#[derive(Debug, Clone, FromRow)]
pub struct LatestProcessInfo {
    pub workspace_id: Uuid,
    pub execution_process_id: Uuid,
    pub session_id: Uuid,
    pub status: ExecutionProcessStatus,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExecutorActionField {
    ExecutorAction(ExecutorAction),
    Other(Value),
}

#[derive(Debug, Clone)]
pub struct MissingBeforeContext {
    pub id: Uuid,
    pub session_id: Uuid,
    pub workspace_id: Uuid,
    pub repo_id: Uuid,
    pub prev_after_head_commit: Option<String>,
    pub target_branch: String,
    pub repo_path: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
struct MissingBeforeContextRow {
    pub id: Uuid,
    pub session_id: Uuid,
    pub workspace_id: Uuid,
    pub repo_id: Uuid,
    pub after_head_commit: Option<String>,
    pub prev_after_head_commit: Option<String>,
    pub target_branch: String,
    pub repo_path: Option<String>,
}

impl ExecutionProcess {
    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, ExecutionProcess>(
            r#"SELECT id, session_id, run_reason, executor_action, status, exit_code,
                      dropped, started_at, completed_at, created_at, updated_at
               FROM execution_processes WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn list_missing_before_context(
        pool: &SqlitePool,
    ) -> Result<Vec<MissingBeforeContext>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MissingBeforeContextRow>(
            r#"SELECT
                ep.id, ep.session_id, s.workspace_id, eprs.repo_id,
                eprs.after_head_commit, prev.after_head_commit as prev_after_head_commit,
                wr.target_branch, r.path as repo_path
            FROM execution_processes ep
            JOIN sessions s ON s.id = ep.session_id
            JOIN execution_process_repo_states eprs ON eprs.execution_process_id = ep.id
            JOIN repos r ON r.id = eprs.repo_id
            JOIN workspaces w ON w.id = s.workspace_id
            JOIN workspace_repos wr ON wr.workspace_id = w.id AND wr.repo_id = eprs.repo_id
            LEFT JOIN execution_process_repo_states prev
              ON prev.execution_process_id = (
                   SELECT id FROM execution_processes
                     WHERE session_id = ep.session_id
                       AND created_at < ep.created_at
                     ORDER BY created_at DESC
                     LIMIT 1
               )
              AND prev.repo_id = eprs.repo_id
            WHERE eprs.before_head_commit IS NULL
              AND eprs.after_head_commit IS NOT NULL"#,
        )
        .fetch_all(pool)
        .await?;

        let result = rows
            .into_iter()
            .map(|r| MissingBeforeContext {
                id: r.id,
                session_id: r.session_id,
                workspace_id: r.workspace_id,
                repo_id: r.repo_id,
                prev_after_head_commit: r.prev_after_head_commit,
                target_branch: r.target_branch,
                repo_path: r.repo_path,
            })
            .collect();
        Ok(result)
    }

    pub async fn find_by_rowid(pool: &SqlitePool, rowid: i64) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, ExecutionProcess>(
            r#"SELECT id, session_id, run_reason, executor_action, status, exit_code,
                      dropped, started_at, completed_at, created_at, updated_at
               FROM execution_processes WHERE rowid = $1"#,
        )
        .bind(rowid)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_session_id(
        pool: &SqlitePool,
        session_id: Uuid,
        show_soft_deleted: bool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        if show_soft_deleted {
            sqlx::query_as::<_, ExecutionProcess>(
                r#"SELECT id, session_id, run_reason, executor_action, status, exit_code,
                          dropped, started_at, completed_at, created_at, updated_at
                   FROM execution_processes
                   WHERE session_id = $1
                   ORDER BY created_at ASC"#,
            )
            .bind(session_id)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as::<_, ExecutionProcess>(
                r#"SELECT id, session_id, run_reason, executor_action, status, exit_code,
                          dropped, started_at, completed_at, created_at, updated_at
                   FROM execution_processes
                   WHERE session_id = $1 AND dropped = 0
                   ORDER BY created_at ASC"#,
            )
            .bind(session_id)
            .fetch_all(pool)
            .await
        }
    }

    pub async fn find_running(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, ExecutionProcess>(
            r#"SELECT id, session_id, run_reason, executor_action, status, exit_code,
                      dropped, started_at, completed_at, created_at, updated_at
               FROM execution_processes WHERE status = 'running' ORDER BY created_at ASC"#,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn has_running_coding_agent_for_session(
        pool: &SqlitePool,
        session_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM execution_processes ep
               WHERE ep.session_id = $1 AND ep.status = 'running' AND ep.run_reason = 'codingagent'"#,
        )
        .bind(session_id)
        .fetch_one(pool)
        .await?;
        Ok(count > 0)
    }

    pub async fn has_running_non_dev_server_processes_for_workspace(
        pool: &SqlitePool,
        workspace_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM execution_processes ep
               JOIN sessions s ON ep.session_id = s.id
               WHERE s.workspace_id = $1 AND ep.status = 'running' AND ep.run_reason != 'devserver'"#,
        )
        .bind(workspace_id)
        .fetch_one(pool)
        .await?;
        Ok(count > 0)
    }

    pub async fn find_running_dev_servers_by_workspace(
        pool: &SqlitePool,
        workspace_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, ExecutionProcess>(
            r#"SELECT ep.id, ep.session_id, ep.run_reason, ep.executor_action, ep.status,
                      ep.exit_code, ep.dropped, ep.started_at, ep.completed_at, ep.created_at, ep.updated_at
               FROM execution_processes ep
               JOIN sessions s ON ep.session_id = s.id
               WHERE s.workspace_id = $1 AND ep.status = 'running' AND ep.run_reason = 'devserver'
               ORDER BY ep.created_at DESC"#,
        )
        .bind(workspace_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_latest_by_session_and_run_reason(
        pool: &SqlitePool,
        session_id: Uuid,
        run_reason: &ExecutionProcessRunReason,
    ) -> Result<Option<Self>, sqlx::Error> {
        let run_reason_str = match run_reason {
            ExecutionProcessRunReason::SetupScript => "setupscript",
            ExecutionProcessRunReason::CleanupScript => "cleanupscript",
            ExecutionProcessRunReason::ArchiveScript => "archivescript",
            ExecutionProcessRunReason::CodingAgent => "codingagent",
            ExecutionProcessRunReason::DevServer => "devserver",
        };

        sqlx::query_as::<_, ExecutionProcess>(
            r#"SELECT id, session_id, run_reason, executor_action, status, exit_code,
                      dropped, started_at, completed_at, created_at, updated_at
               FROM execution_processes
               WHERE session_id = $1 AND run_reason = $2 AND dropped = 0
               ORDER BY created_at DESC LIMIT 1"#,
        )
        .bind(session_id)
        .bind(run_reason_str)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_latest_by_workspace_and_run_reason(
        pool: &SqlitePool,
        workspace_id: Uuid,
        run_reason: &ExecutionProcessRunReason,
    ) -> Result<Option<Self>, sqlx::Error> {
        let run_reason_str = match run_reason {
            ExecutionProcessRunReason::SetupScript => "setupscript",
            ExecutionProcessRunReason::CleanupScript => "cleanupscript",
            ExecutionProcessRunReason::ArchiveScript => "archivescript",
            ExecutionProcessRunReason::CodingAgent => "codingagent",
            ExecutionProcessRunReason::DevServer => "devserver",
        };

        sqlx::query_as::<_, ExecutionProcess>(
            r#"SELECT ep.id, ep.session_id, ep.run_reason, ep.executor_action, ep.status,
                      ep.exit_code, ep.dropped, ep.started_at, ep.completed_at, ep.created_at, ep.updated_at
               FROM execution_processes ep
               JOIN sessions s ON ep.session_id = s.id
               WHERE s.workspace_id = $1 AND ep.run_reason = $2 AND ep.dropped = 0
               ORDER BY ep.created_at DESC LIMIT 1"#,
        )
        .bind(workspace_id)
        .bind(run_reason_str)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &SqlitePool,
        data: &CreateExecutionProcess,
        process_id: Uuid,
        repo_states: &[CreateExecutionProcessRepoState],
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();
        let executor_action_json = sqlx::types::Json(&data.executor_action);

        let run_reason_str = match data.run_reason {
            ExecutionProcessRunReason::SetupScript => "setupscript",
            ExecutionProcessRunReason::CleanupScript => "cleanupscript",
            ExecutionProcessRunReason::ArchiveScript => "archivescript",
            ExecutionProcessRunReason::CodingAgent => "codingagent",
            ExecutionProcessRunReason::DevServer => "devserver",
        };

        sqlx::query(
            r#"INSERT INTO execution_processes (
                    id, session_id, run_reason, executor_action,
                    status, exit_code, dropped, started_at, completed_at, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, 'running', NULL, 0, $5, NULL, $5, $5)"#,
        )
        .bind(process_id)
        .bind(data.session_id)
        .bind(run_reason_str)
        .bind(executor_action_json)
        .bind(now)
        .execute(pool)
        .await?;

        ExecutionProcessRepoState::create_many(pool, process_id, repo_states).await?;

        Self::find_by_id(pool, process_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn was_stopped(pool: &SqlitePool, id: Uuid) -> bool {
        if let Ok(exp_process) = Self::find_by_id(pool, id).await
            && exp_process.is_some_and(|ep| {
                ep.status == ExecutionProcessStatus::Killed
                    || ep.status == ExecutionProcessStatus::Completed
            })
        {
            return true;
        }
        false
    }

    pub async fn update_completion(
        pool: &SqlitePool,
        id: Uuid,
        status: ExecutionProcessStatus,
        exit_code: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        let completed_at = if matches!(status, ExecutionProcessStatus::Running) {
            None
        } else {
            Some(Utc::now())
        };

        let status_str = match status {
            ExecutionProcessStatus::Running => "running",
            ExecutionProcessStatus::Completed => "completed",
            ExecutionProcessStatus::Failed => "failed",
            ExecutionProcessStatus::Killed => "killed",
        };

        sqlx::query(
            r#"UPDATE execution_processes
               SET status = $1, exit_code = $2, completed_at = $3, updated_at = datetime('now', 'subsec')
               WHERE id = $4"#,
        )
        .bind(status_str)
        .bind(exit_code)
        .bind(completed_at)
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub fn executor_action(&self) -> Result<&ExecutorAction, anyhow::Error> {
        match &self.executor_action.0 {
            ExecutorActionField::ExecutorAction(action) => Ok(action),
            ExecutorActionField::Other(_) => Err(anyhow::anyhow!(
                "Executor action is not a valid ExecutorAction JSON object"
            )),
        }
    }

    pub async fn drop_at_and_after(
        pool: &SqlitePool,
        session_id: Uuid,
        boundary_process_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE execution_processes
               SET dropped = 1
             WHERE session_id = $1
               AND created_at >= (SELECT created_at FROM execution_processes WHERE id = $2)
               AND dropped = 0"#,
        )
        .bind(session_id)
        .bind(boundary_process_id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() as i64)
    }

    pub async fn find_prev_after_head_commit(
        pool: &SqlitePool,
        session_id: Uuid,
        boundary_process_id: Uuid,
        repo_id: Uuid,
    ) -> Result<Option<String>, sqlx::Error> {
        let result: Option<String> = sqlx::query_scalar(
            r#"SELECT eprs.after_head_commit
               FROM execution_process_repo_states eprs
               JOIN execution_processes ep ON ep.id = eprs.execution_process_id
              WHERE ep.session_id = $1
                AND eprs.repo_id = $2
                AND ep.created_at < (SELECT created_at FROM execution_processes WHERE id = $3)
              ORDER BY ep.created_at DESC
              LIMIT 1"#,
        )
        .bind(session_id)
        .bind(repo_id)
        .bind(boundary_process_id)
        .fetch_optional(pool)
        .await?;
        Ok(result)
    }

    pub async fn parent_session(&self, pool: &SqlitePool) -> Result<Option<Session>, sqlx::Error> {
        Session::find_by_id(pool, self.session_id).await
    }

    pub async fn parent_workspace_and_session(
        &self,
        pool: &SqlitePool,
    ) -> Result<Option<(Workspace, Session)>, sqlx::Error> {
        let session = match Session::find_by_id(pool, self.session_id).await? {
            Some(s) => s,
            None => return Ok(None),
        };
        let workspace = match Workspace::find_by_id(pool, session.workspace_id).await? {
            Some(w) => w,
            None => return Ok(None),
        };
        Ok(Some((workspace, session)))
    }

    pub async fn load_context(
        pool: &SqlitePool,
        exec_id: Uuid,
    ) -> Result<ExecutionContext, sqlx::Error> {
        let execution_process = Self::find_by_id(pool, exec_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        let session = Session::find_by_id(pool, execution_process.session_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        let workspace = Workspace::find_by_id(pool, session.workspace_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        let repos = WorkspaceRepo::find_repos_for_workspace(pool, workspace.id).await?;

        Ok(ExecutionContext {
            execution_process,
            session,
            workspace,
            repos,
        })
    }

    pub async fn latest_executor_profile_for_session(
        pool: &SqlitePool,
        session_id: Uuid,
    ) -> Result<Option<ExecutorProfileId>, ExecutionProcessError> {
        let latest_execution_process = sqlx::query_as::<_, ExecutionProcess>(
            r#"SELECT id, session_id, run_reason, executor_action, status, exit_code,
                      dropped, started_at, completed_at, created_at, updated_at
               FROM execution_processes
               WHERE session_id = $1 AND run_reason = 'codingagent' AND dropped = 0
               ORDER BY created_at DESC LIMIT 1"#,
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await?;

        let Some(latest_execution_process) = latest_execution_process else {
            return Ok(None);
        };

        let action = latest_execution_process
            .executor_action()
            .map_err(|e| ExecutionProcessError::ValidationError(e.to_string()))?;

        match &action.typ {
            ExecutorActionType::CodingAgentInitialRequest(request) => {
                Ok(Some(request.executor_config.profile_id()))
            }
            ExecutorActionType::CodingAgentFollowUpRequest(request) => {
                Ok(Some(request.executor_config.profile_id()))
            }
            ExecutorActionType::ReviewRequest(request) => {
                Ok(Some(request.executor_config.profile_id()))
            }
            _ => Err(ExecutionProcessError::ValidationError(
                "Couldn't find profile from initial request".to_string(),
            )),
        }
    }

    pub async fn find_latest_for_workspaces(
        pool: &SqlitePool,
        archived: bool,
    ) -> Result<HashMap<Uuid, LatestProcessInfo>, sqlx::Error> {
        let archived_val: i32 = if archived { 1 } else { 0 };

        let rows: Vec<LatestProcessInfo> = sqlx::query_as::<_, LatestProcessInfo>(
            r#"SELECT workspace_id, execution_process_id, session_id, status, completed_at
            FROM (
                SELECT
                    s.workspace_id,
                    ep.id as execution_process_id,
                    ep.session_id,
                    ep.status,
                    ep.completed_at,
                    ROW_NUMBER() OVER (
                        PARTITION BY s.workspace_id
                        ORDER BY ep.created_at DESC
                    ) as rn
                FROM execution_processes ep
                JOIN sessions s ON ep.session_id = s.id
                JOIN workspaces w ON s.workspace_id = w.id
                WHERE w.archived = $1
                  AND ep.run_reason IN ('codingagent', 'setupscript', 'cleanupscript')
                  AND ep.dropped = 0
            )
            WHERE rn = 1"#,
        )
        .bind(archived_val)
        .fetch_all(pool)
        .await?;

        let result = rows
            .into_iter()
            .map(|info| (info.workspace_id, info))
            .collect();

        Ok(result)
    }

    pub async fn find_workspaces_with_running_dev_servers(
        pool: &SqlitePool,
        archived: bool,
    ) -> Result<HashSet<Uuid>, sqlx::Error> {
        let archived_val: i32 = if archived { 1 } else { 0 };

        let rows: Vec<(Uuid,)> = sqlx::query_as(
            r#"SELECT DISTINCT s.workspace_id
            FROM execution_processes ep
            JOIN sessions s ON ep.session_id = s.id
            JOIN workspaces w ON s.workspace_id = w.id
            WHERE w.archived = $1 AND ep.status = 'running' AND ep.run_reason = 'devserver'"#,
        )
        .bind(archived_val)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }
}
