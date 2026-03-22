use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct Tag {
    pub id: Uuid,
    pub tag_name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateTag {
    pub tag_name: String,
    pub content: String,
}

#[derive(Debug, Deserialize, TS)]
pub struct UpdateTag {
    pub tag_name: Option<String>,
    pub content: Option<String>,
}

impl Tag {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Tag>(
            r#"SELECT id, tag_name, content, created_at, updated_at
               FROM tags
               ORDER BY tag_name ASC"#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Tag>(
            r#"SELECT id, tag_name, content, created_at, updated_at
               FROM tags
               WHERE id = $1"#
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, data: &CreateTag) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        sqlx::query(
            r#"INSERT INTO tags (id, tag_name, content, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5)"#
        )
        .bind(id)
        .bind(&data.tag_name)
        .bind(&data.content)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn update(
        pool: &SqlitePool,
        id: Uuid,
        data: &UpdateTag,
    ) -> Result<Self, sqlx::Error> {
        let existing = Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        let tag_name = data.tag_name.as_ref().unwrap_or(&existing.tag_name);
        let content = data.content.as_ref().unwrap_or(&existing.content);
        let now = Utc::now();

        sqlx::query(
            r#"UPDATE tags
               SET tag_name = $2, content = $3, updated_at = $4
               WHERE id = $1"#
        )
        .bind(id)
        .bind(tag_name)
        .bind(content)
        .bind(now)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn delete(pool: &SqlitePool, id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM tags WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}
