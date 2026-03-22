use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct Image {
    pub id: Uuid,
    pub file_path: String,
    pub original_name: String,
    pub mime_type: Option<String>,
    pub size_bytes: i64,
    pub hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateImage {
    pub file_path: String,
    pub original_name: String,
    pub mime_type: Option<String>,
    pub size_bytes: i64,
    pub hash: String,
}

impl Image {
    pub async fn create(pool: &SqlitePool, data: &CreateImage) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO images (id, file_path, original_name, mime_type, size_bytes, hash, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, datetime('now', 'subsec'), datetime('now', 'subsec'))"#
        )
        .bind(id)
        .bind(&data.file_path)
        .bind(&data.original_name)
        .bind(&data.mime_type)
        .bind(data.size_bytes)
        .bind(&data.hash)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn find_by_hash(pool: &SqlitePool, hash: &str) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query_as::<_, (Uuid, String, String, Option<String>, i64, String, DateTime<Utc>, DateTime<Utc>)>(
            r#"SELECT id, file_path, original_name, mime_type, size_bytes, hash, created_at, updated_at
               FROM images
               WHERE hash = $1"#
        )
        .bind(hash)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| Image {
            id: r.0,
            file_path: r.1,
            original_name: r.2,
            mime_type: r.3,
            size_bytes: r.4,
            hash: r.5,
            created_at: r.6,
            updated_at: r.7,
        }))
    }

    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query_as::<_, (Uuid, String, String, Option<String>, i64, String, DateTime<Utc>, DateTime<Utc>)>(
            r#"SELECT id, file_path, original_name, mime_type, size_bytes, hash, created_at, updated_at
               FROM images
               WHERE id = $1"#
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| Image {
            id: r.0,
            file_path: r.1,
            original_name: r.2,
            mime_type: r.3,
            size_bytes: r.4,
            hash: r.5,
            created_at: r.6,
            updated_at: r.7,
        }))
    }

    pub async fn find_by_file_path(
        pool: &SqlitePool,
        file_path: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query_as::<_, (Uuid, String, String, Option<String>, i64, String, DateTime<Utc>, DateTime<Utc>)>(
            r#"SELECT id, file_path, original_name, mime_type, size_bytes, hash, created_at, updated_at
               FROM images
               WHERE file_path = $1"#
        )
        .bind(file_path)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| Image {
            id: r.0,
            file_path: r.1,
            original_name: r.2,
            mime_type: r.3,
            size_bytes: r.4,
            hash: r.5,
            created_at: r.6,
            updated_at: r.7,
        }))
    }

    pub async fn find_by_workspace_id(
        pool: &SqlitePool,
        workspace_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (Uuid, String, String, Option<String>, i64, String, DateTime<Utc>, DateTime<Utc>)>(
            r#"SELECT i.id, i.file_path, i.original_name, i.mime_type, i.size_bytes, i.hash, i.created_at, i.updated_at
               FROM images i
               JOIN workspace_images wi ON i.id = wi.image_id
               WHERE wi.workspace_id = $1
               ORDER BY wi.created_at"#
        )
        .bind(workspace_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| Image {
            id: r.0,
            file_path: r.1,
            original_name: r.2,
            mime_type: r.3,
            size_bytes: r.4,
            hash: r.5,
            created_at: r.6,
            updated_at: r.7,
        }).collect())
    }

    pub async fn delete(pool: &SqlitePool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(r#"DELETE FROM images WHERE id = $1"#)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn find_orphaned_images(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (Uuid, String, String, Option<String>, i64, String, DateTime<Utc>, DateTime<Utc>)>(
            r#"SELECT i.id, i.file_path, i.original_name, i.mime_type, i.size_bytes, i.hash, i.created_at, i.updated_at
               FROM images i
               LEFT JOIN workspace_images wi ON i.id = wi.image_id
               WHERE wi.workspace_id IS NULL"#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| Image {
            id: r.0,
            file_path: r.1,
            original_name: r.2,
            mime_type: r.3,
            size_bytes: r.4,
            hash: r.5,
            created_at: r.6,
            updated_at: r.7,
        }).collect())
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WorkspaceImage {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub image_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl WorkspaceImage {
    pub async fn associate_many_dedup(
        pool: &SqlitePool,
        workspace_id: Uuid,
        image_ids: &[Uuid],
    ) -> Result<(), sqlx::Error> {
        for &image_id in image_ids {
            let id = Uuid::new_v4();
            sqlx::query(
                r#"INSERT INTO workspace_images (id, workspace_id, image_id)
                   SELECT $1, $2, $3
                   WHERE NOT EXISTS (
                       SELECT 1 FROM workspace_images WHERE workspace_id = $2 AND image_id = $3
                   )"#
            )
            .bind(id)
            .bind(workspace_id)
            .bind(image_id)
            .execute(pool)
            .await?;
        }
        Ok(())
    }
}
