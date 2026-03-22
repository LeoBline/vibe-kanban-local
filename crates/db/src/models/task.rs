use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, Type};
use strum_macros::{Display, EnumString};
use ts_rs::TS;
use uuid::Uuid;

#[derive(
    Debug, Clone, Type, Serialize, Deserialize, PartialEq, TS, EnumString, Display, Default,
)]
#[sqlx(type_name = "task_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum TaskStatus {
    #[default]
    Todo,
    InProgress,
    InReview,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct Task {
    pub id: Uuid,
    pub project_id: Uuid, // Foreign key to Project
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub parent_workspace_id: Option<Uuid>, // Foreign key to parent Workspace
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, String, Option<Uuid>, DateTime<Utc>, DateTime<Utc>)>(
            r#"SELECT id, project_id, title, description, status, parent_workspace_id, created_at, updated_at
               FROM tasks
               ORDER BY created_at ASC"#
        )
        .fetch_all(pool)
        .await?;

        let tasks = rows.into_iter().map(|r| {
            let status_str = r.4.to_lowercase();
            let status = match status_str.as_str() {
                "inprogress" => TaskStatus::InProgress,
                "inreview" => TaskStatus::InReview,
                "done" => TaskStatus::Done,
                "cancelled" => TaskStatus::Cancelled,
                _ => TaskStatus::Todo,
            };
            Task {
                id: r.0,
                project_id: r.1,
                title: r.2,
                description: r.3,
                status,
                parent_workspace_id: r.5,
                created_at: r.6,
                updated_at: r.7,
            }
        }).collect();

        Ok(tasks)
    }

    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, String, Option<Uuid>, DateTime<Utc>, DateTime<Utc>)>(
            r#"SELECT id, project_id, title, description, status, parent_workspace_id, created_at, updated_at
               FROM tasks
               WHERE id = $1"#
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| {
            let status_str = r.4.to_lowercase();
            let status = match status_str.as_str() {
                "inprogress" => TaskStatus::InProgress,
                "inreview" => TaskStatus::InReview,
                "done" => TaskStatus::Done,
                "cancelled" => TaskStatus::Cancelled,
                _ => TaskStatus::Todo,
            };
            Task {
                id: r.0,
                project_id: r.1,
                title: r.2,
                description: r.3,
                status,
                parent_workspace_id: r.5,
                created_at: r.6,
                updated_at: r.7,
            }
        }))
    }
}
