use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct CodingAgentTurn {
    pub id: Uuid,
    pub execution_process_id: Uuid,
    pub agent_session_id: Option<String>,
    pub agent_message_id: Option<String>,
    pub prompt: Option<String>,
    pub summary: Option<String>,
    pub seen: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateCodingAgentTurn {
    pub execution_process_id: Uuid,
    pub prompt: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct CodingAgentResumeInfo {
    pub session_id: String,
    pub message_id: Option<String>,
}

impl CodingAgentTurn {
    pub async fn find_latest_session_info(
        pool: &SqlitePool,
        session_id: Uuid,
    ) -> Result<Option<CodingAgentResumeInfo>, sqlx::Error> {
        sqlx::query_as::<_, CodingAgentResumeInfo>(
            r#"SELECT
                cat.agent_session_id as session_id,
                cat.agent_message_id as message_id
               FROM execution_processes ep
               JOIN coding_agent_turns cat ON ep.id = cat.execution_process_id
               WHERE ep.session_id = $1
                 AND ep.run_reason = 'codingagent'
                 AND ep.dropped = 0
                 AND cat.agent_session_id IS NOT NULL
               ORDER BY ep.created_at DESC
               LIMIT 1"#
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_execution_process_id(
        pool: &SqlitePool,
        execution_process_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, CodingAgentTurn>(
            r#"SELECT
                id,
                execution_process_id,
                agent_session_id,
                agent_message_id,
                prompt,
                summary,
                seen,
                created_at,
                updated_at
               FROM coding_agent_turns
               WHERE execution_process_id = $1"#,
        )
        .bind(execution_process_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_agent_session_id(
        pool: &SqlitePool,
        agent_session_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, CodingAgentTurn>(
            r#"SELECT
                id,
                execution_process_id,
                agent_session_id,
                agent_message_id,
                prompt,
                summary,
                seen,
                created_at,
                updated_at
               FROM coding_agent_turns
               WHERE agent_session_id = $1
               ORDER BY updated_at DESC
               LIMIT 1"#,
        )
        .bind(agent_session_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &SqlitePool,
        data: &CreateCodingAgentTurn,
        id: Uuid,
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();

        tracing::debug!(
            "Creating coding agent turn: id={}, execution_process_id={}, agent_session_id=None (will be set later)",
            id,
            data.execution_process_id
        );

        sqlx::query(
            r#"INSERT INTO coding_agent_turns (
                id, execution_process_id, agent_session_id, agent_message_id, prompt, summary, seen,
                created_at, updated_at
               )
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#
        )
        .bind(id)
        .bind(data.execution_process_id)
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind(&data.prompt)
        .bind(Option::<String>::None)
        .bind(false)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Self::find_by_execution_process_id(pool, data.execution_process_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn update_agent_session_id(
        pool: &SqlitePool,
        execution_process_id: Uuid,
        agent_session_id: &str,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            r#"UPDATE coding_agent_turns
               SET agent_session_id = $1, updated_at = $2
               WHERE execution_process_id = $3"#,
        )
        .bind(agent_session_id)
        .bind(now)
        .bind(execution_process_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_agent_message_id(
        pool: &SqlitePool,
        execution_process_id: Uuid,
        agent_message_id: &str,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            r#"UPDATE coding_agent_turns
               SET agent_message_id = $1, updated_at = $2
               WHERE execution_process_id = $3"#,
        )
        .bind(agent_message_id)
        .bind(now)
        .bind(execution_process_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_summary(
        pool: &SqlitePool,
        execution_process_id: Uuid,
        summary: &str,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            r#"UPDATE coding_agent_turns
               SET summary = $1, updated_at = $2
               WHERE execution_process_id = $3"#,
        )
        .bind(summary)
        .bind(now)
        .bind(execution_process_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn mark_unseen_by_execution_process_id(
        pool: &SqlitePool,
        execution_process_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            r#"UPDATE coding_agent_turns
               SET seen = 0, updated_at = $1
               WHERE execution_process_id = $2
                 AND seen = 1"#,
        )
        .bind(now)
        .bind(execution_process_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn mark_seen_by_workspace_id(
        pool: &SqlitePool,
        workspace_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            r#"UPDATE coding_agent_turns
               SET seen = 1, updated_at = $1
               WHERE execution_process_id IN (
                   SELECT ep.id FROM execution_processes ep
                   JOIN sessions s ON ep.session_id = s.id
                   WHERE s.workspace_id = $2
               ) AND seen = 0"#,
        )
        .bind(now)
        .bind(workspace_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn has_unseen_by_workspace_id(
        pool: &SqlitePool,
        workspace_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i64>(
            r#"SELECT 1
               FROM coding_agent_turns cat
               JOIN execution_processes ep ON cat.execution_process_id = ep.id
               JOIN sessions s ON ep.session_id = s.id
               WHERE s.workspace_id = $1 AND cat.seen = 0
               LIMIT 1"#,
        )
        .bind(workspace_id)
        .fetch_optional(pool)
        .await?;

        Ok(result.is_some())
    }

    pub async fn find_workspaces_with_unseen(
        pool: &SqlitePool,
        archived: bool,
    ) -> Result<std::collections::HashSet<Uuid>, sqlx::Error> {
        let result: Vec<(Uuid,)> = sqlx::query_as(
            r#"SELECT DISTINCT s.workspace_id
               FROM coding_agent_turns cat
               JOIN execution_processes ep ON cat.execution_process_id = ep.id
               JOIN sessions s ON ep.session_id = s.id
               JOIN workspaces w ON s.workspace_id = w.id
               WHERE cat.seen = 0 AND w.archived = $1"#,
        )
        .bind(archived)
        .fetch_all(pool)
        .await?;

        Ok(result.into_iter().map(|(id,)| id).collect())
    }
}
