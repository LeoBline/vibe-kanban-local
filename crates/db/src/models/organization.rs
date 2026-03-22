use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use ts_rs::TS;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct Organization {
    pub id: String,
    pub name: String,
    #[ts(type = "Date")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "Date")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateOrganization {
    pub name: String,
}

impl Organization {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Organization>(
            r#"SELECT id, name, created_at, updated_at
               FROM organizations
               ORDER BY created_at DESC"#,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Organization>(
            r#"SELECT id, name, created_at, updated_at
               FROM organizations
               WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, data: &CreateOrganization) -> Result<Self, sqlx::Error> {
        let id = format!("local-org-{}", chrono::Utc::now().timestamp_millis());
        
        sqlx::query(
            r#"INSERT INTO organizations (id, name)
               VALUES (?, ?)"#,
        )
        .bind(&id)
        .bind(&data.name)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, &id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn update(pool: &SqlitePool, id: &str, name: &str) -> Result<Self, sqlx::Error> {
        sqlx::query(
            r#"UPDATE organizations
               SET name = ?, updated_at = datetime('now', 'subsec')
               WHERE id = ?"#,
        )
        .bind(name)
        .bind(id)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM organizations WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}
