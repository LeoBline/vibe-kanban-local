use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use ts_rs::TS;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct ProjectStatus {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub color: String,
    pub sort_order: i64,
    pub hidden: i64,
    #[ts(type = "Date")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateProjectStatus {
    pub project_id: String,
    pub name: String,
    pub color: String,
    pub sort_order: i32,
    pub hidden: bool,
}

#[derive(Debug, Deserialize, TS)]
pub struct UpdateProjectStatus {
    pub name: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
    pub hidden: Option<bool>,
}

impl ProjectStatus {
    pub fn is_hidden(&self) -> bool {
        self.hidden != 0
    }

    pub async fn find_by_project(pool: &SqlitePool, project_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, ProjectStatus>(
            r#"SELECT id, project_id, name, color, sort_order, COALESCE(CAST(hidden AS INTEGER), 0) as hidden, created_at
               FROM project_statuses
               WHERE project_id = ?
               ORDER BY sort_order ASC"#,
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, ProjectStatus>(
            r#"SELECT id, project_id, name, color, sort_order, COALESCE(CAST(hidden AS INTEGER), 0) as hidden, created_at
               FROM project_statuses
               WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, data: &CreateProjectStatus) -> Result<Self, sqlx::Error> {
        let id = format!("local-status-{}", chrono::Utc::now().timestamp_millis());
        let hidden_val: i64 = if data.hidden { 1 } else { 0 };
        let sort_order_val: i64 = data.sort_order as i64;
        
        sqlx::query(
            r#"INSERT INTO project_statuses (id, project_id, name, color, sort_order, hidden)
               VALUES (?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&id)
        .bind(&data.project_id)
        .bind(&data.name)
        .bind(&data.color)
        .bind(sort_order_val)
        .bind(hidden_val)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, &id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn create_default_statuses(pool: &SqlitePool, project_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        let default_statuses = vec![
            CreateProjectStatus { project_id: project_id.to_string(), name: "Backlog".to_string(), color: "220 9% 46%".to_string(), sort_order: 0, hidden: true },
            CreateProjectStatus { project_id: project_id.to_string(), name: "Todo".to_string(), color: "59 130 246".to_string(), sort_order: 1, hidden: false },
            CreateProjectStatus { project_id: project_id.to_string(), name: "In Progress".to_string(), color: "245 158 11".to_string(), sort_order: 2, hidden: false },
            CreateProjectStatus { project_id: project_id.to_string(), name: "In Review".to_string(), color: "168 85 247".to_string(), sort_order: 3, hidden: false },
            CreateProjectStatus { project_id: project_id.to_string(), name: "Done".to_string(), color: "34 197 94".to_string(), sort_order: 4, hidden: false },
        ];

        let mut results = Vec::new();
        for status_data in default_statuses {
            let status = Self::create(pool, &status_data).await?;
            results.push(status);
        }
        Ok(results)
    }

    pub async fn update(pool: &SqlitePool, id: &str, data: &UpdateProjectStatus) -> Result<Self, sqlx::Error> {
        let existing = Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        let name = data.name.clone().unwrap_or(existing.name.clone());
        let color = data.color.clone().unwrap_or(existing.color.clone());
        let sort_order: i64 = data.sort_order.map(|v| v as i64).unwrap_or(existing.sort_order);
        let hidden_val: i64 = if data.hidden.unwrap_or(existing.is_hidden()) { 1 } else { 0 };

        sqlx::query(
            r#"UPDATE project_statuses
               SET name = ?, color = ?, sort_order = ?, hidden = ?
               WHERE id = ?"#,
        )
        .bind(&name)
        .bind(&color)
        .bind(sort_order)
        .bind(hidden_val)
        .bind(id)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM project_statuses WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}
