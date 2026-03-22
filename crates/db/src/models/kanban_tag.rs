use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use ts_rs::TS;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct KanbanTag {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateKanbanTag {
    pub project_id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Deserialize, TS)]
pub struct UpdateKanbanTag {
    pub name: Option<String>,
    pub color: Option<String>,
}

impl KanbanTag {
    pub async fn find_by_project(pool: &SqlitePool, project_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, KanbanTag>(
            r#"SELECT id, project_id, name, color, created_at
               FROM kanban_tags
               WHERE project_id = ?
               ORDER BY name ASC"#,
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, KanbanTag>(
            r#"SELECT id, project_id, name, color, created_at
               FROM kanban_tags
               WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, data: &CreateKanbanTag) -> Result<Self, sqlx::Error> {
        let id = format!("local-tag-{}", chrono::Utc::now().timestamp_millis());
        
        sqlx::query(
            r#"INSERT INTO kanban_tags (id, project_id, name, color)
               VALUES (?, ?, ?, ?)"#,
        )
        .bind(&id)
        .bind(&data.project_id)
        .bind(&data.name)
        .bind(&data.color)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, &id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn update(pool: &SqlitePool, id: &str, data: &UpdateKanbanTag) -> Result<Self, sqlx::Error> {
        let existing = Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        let name = data.name.as_ref().unwrap_or(&existing.name);
        let color = data.color.as_ref().unwrap_or(&existing.color);

        sqlx::query(
            r#"UPDATE kanban_tags
               SET name = ?, color = ?
               WHERE id = ?"#,
        )
        .bind(name)
        .bind(color)
        .bind(id)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM kanban_tags WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}
