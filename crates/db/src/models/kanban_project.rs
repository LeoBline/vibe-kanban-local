use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use ts_rs::TS;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct KanbanProject {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub color: String,
    pub sort_order: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateKanbanProject {
    pub organization_id: String,
    pub name: String,
    pub color: String,
    #[serde(default)]
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, TS)]
pub struct UpdateKanbanProject {
    pub name: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i64>,
}

impl KanbanProject {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, KanbanProject>(
            r#"SELECT id, organization_id, name, color, sort_order, created_at, updated_at
               FROM kanban_projects
               ORDER BY sort_order ASC, created_at DESC"#,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, KanbanProject>(
            r#"SELECT id, organization_id, name, color, sort_order, created_at, updated_at
               FROM kanban_projects
               WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_organization(pool: &SqlitePool, org_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, KanbanProject>(
            r#"SELECT id, organization_id, name, color, sort_order, created_at, updated_at
               FROM kanban_projects
               WHERE organization_id = ?
               ORDER BY sort_order ASC, created_at DESC"#,
        )
        .bind(org_id)
        .fetch_all(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, data: &CreateKanbanProject) -> Result<Self, sqlx::Error> {
        let id = data.id.clone().unwrap_or_else(|| format!("local-project-{}", chrono::Utc::now().timestamp_millis()));
        
        let max_sort_order: Option<(i64,)> = sqlx::query_as(
            "SELECT COALESCE(MAX(sort_order), -1) FROM kanban_projects WHERE organization_id = ?"
        )
        .bind(&data.organization_id)
        .fetch_optional(pool)
        .await?;
        
        let sort_order = max_sort_order.map(|r| r.0 + 1).unwrap_or(0);

        sqlx::query(
            r#"INSERT INTO kanban_projects (id, organization_id, name, color, sort_order)
               VALUES (?, ?, ?, ?, ?)"#,
        )
        .bind(&id)
        .bind(&data.organization_id)
        .bind(&data.name)
        .bind(&data.color)
        .bind(sort_order)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, &id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn update(pool: &SqlitePool, id: &str, data: &UpdateKanbanProject) -> Result<Self, sqlx::Error> {
        let existing = Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        let name = data.name.as_ref().unwrap_or(&existing.name);
        let color = data.color.as_ref().unwrap_or(&existing.color);
        let sort_order = data.sort_order.unwrap_or(existing.sort_order);

        sqlx::query(
            r#"UPDATE kanban_projects
               SET name = ?, color = ?, sort_order = ?, updated_at = datetime('now', 'subsec')
               WHERE id = ?"#,
        )
        .bind(name)
        .bind(color)
        .bind(sort_order)
        .bind(id)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM kanban_projects WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}
