use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, Type};
use strum_macros::{Display, EnumString};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum MigrationStateError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq, EnumString, Display)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum EntityType {
    Project,
    Task,
    PrMerge,
    Workspace,
}

#[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq, EnumString, Display, Default)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum MigrationStatus {
    #[default]
    Pending,
    Migrated,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MigrationState {
    pub id: Uuid,
    pub entity_type: EntityType,
    pub local_id: Uuid,
    pub remote_id: Option<Uuid>,
    pub status: MigrationStatus,
    pub error_message: Option<String>,
    pub attempt_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMigrationState {
    pub entity_type: EntityType,
    pub local_id: Uuid,
}

impl MigrationState {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Self>, MigrationStateError> {
        let records = sqlx::query_as::<_, (Uuid, String, Uuid, Option<Uuid>, String, Option<String>, i64, DateTime<Utc>, DateTime<Utc>)>(
            r#"SELECT id, entity_type, local_id, remote_id, status, error_message, attempt_count, created_at, updated_at
               FROM migration_state ORDER BY created_at ASC"#
        )
        .fetch_all(pool)
        .await?;

        Ok(records.into_iter().map(Self::from_row_tuple).collect())
    }

    pub async fn find_by_entity_type(
        pool: &SqlitePool,
        entity_type: EntityType,
    ) -> Result<Vec<Self>, MigrationStateError> {
        let entity_type_str = entity_type.to_string();
        let records = sqlx::query_as::<_, (Uuid, String, Uuid, Option<Uuid>, String, Option<String>, i64, DateTime<Utc>, DateTime<Utc>)>(
            r#"SELECT id, entity_type, local_id, remote_id, status, error_message, attempt_count, created_at, updated_at
               FROM migration_state WHERE entity_type = $1 ORDER BY created_at ASC"#
        )
        .bind(entity_type_str)
        .fetch_all(pool)
        .await?;

        Ok(records.into_iter().map(Self::from_row_tuple).collect())
    }

    pub async fn find_by_status(
        pool: &SqlitePool,
        status: MigrationStatus,
    ) -> Result<Vec<Self>, MigrationStateError> {
        let status_str = status.to_string();
        let records = sqlx::query_as::<_, (Uuid, String, Uuid, Option<Uuid>, String, Option<String>, i64, DateTime<Utc>, DateTime<Utc>)>(
            r#"SELECT id, entity_type, local_id, remote_id, status, error_message, attempt_count, created_at, updated_at
               FROM migration_state WHERE status = $1 ORDER BY created_at ASC"#
        )
        .bind(status_str)
        .fetch_all(pool)
        .await?;

        Ok(records.into_iter().map(Self::from_row_tuple).collect())
    }

    pub async fn find_pending_by_type(
        pool: &SqlitePool,
        entity_type: EntityType,
    ) -> Result<Vec<Self>, MigrationStateError> {
        let entity_type_str = entity_type.to_string();
        let records = sqlx::query_as::<_, (Uuid, String, Uuid, Option<Uuid>, String, Option<String>, i64, DateTime<Utc>, DateTime<Utc>)>(
            r#"SELECT id, entity_type, local_id, remote_id, status, error_message, attempt_count, created_at, updated_at
               FROM migration_state WHERE entity_type = $1 AND status = 'pending' ORDER BY created_at ASC"#
        )
        .bind(entity_type_str)
        .fetch_all(pool)
        .await?;

        Ok(records.into_iter().map(Self::from_row_tuple).collect())
    }

    pub async fn find_by_entity(
        pool: &SqlitePool,
        entity_type: EntityType,
        local_id: Uuid,
    ) -> Result<Option<Self>, MigrationStateError> {
        let entity_type_str = entity_type.to_string();
        let record = sqlx::query_as::<_, (Uuid, String, Uuid, Option<Uuid>, String, Option<String>, i64, DateTime<Utc>, DateTime<Utc>)>(
            r#"SELECT id, entity_type, local_id, remote_id, status, error_message, attempt_count, created_at, updated_at
               FROM migration_state WHERE entity_type = $1 AND local_id = $2"#
        )
        .bind(entity_type_str)
        .bind(local_id)
        .fetch_optional(pool)
        .await?;

        Ok(record.map(Self::from_row_tuple))
    }

    pub async fn get_remote_id(
        pool: &SqlitePool,
        entity_type: EntityType,
        local_id: Uuid,
    ) -> Result<Option<Uuid>, MigrationStateError> {
        let entity_type_str = entity_type.to_string();
        let record = sqlx::query_scalar::<_, Option<Uuid>>(
            r#"SELECT remote_id FROM migration_state
               WHERE entity_type = $1 AND local_id = $2 AND status = 'migrated'"#
        )
        .bind(entity_type_str)
        .bind(local_id)
        .fetch_optional(pool)
        .await?;

        Ok(record.flatten())
    }

    pub async fn create(
        pool: &SqlitePool,
        data: &CreateMigrationState,
    ) -> Result<Self, MigrationStateError> {
        let id = Uuid::new_v4();
        let entity_type_str = data.entity_type.to_string();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO migration_state (id, entity_type, local_id, status, attempt_count, created_at, updated_at)
               VALUES ($1, $2, $3, 'pending', 0, $4, $5)"#
        )
        .bind(id)
        .bind(entity_type_str)
        .bind(data.local_id)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Self::find_by_id_internal(pool, id)
            .await?
            .ok_or(MigrationStateError::Database(sqlx::Error::RowNotFound))
    }

    pub async fn upsert(
        pool: &SqlitePool,
        data: &CreateMigrationState,
    ) -> Result<Self, MigrationStateError> {
        let id = Uuid::new_v4();
        let entity_type_str = data.entity_type.to_string();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO migration_state (id, entity_type, local_id, status, attempt_count, created_at, updated_at)
               VALUES ($1, $2, $3, 'pending', 0, $4, $5)
               ON CONFLICT (entity_type, local_id) DO UPDATE SET
                   updated_at = datetime('now', 'subsec')"#
        )
        .bind(id)
        .bind(entity_type_str)
        .bind(data.local_id)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Self::find_by_entity_internal(pool, data.entity_type.clone(), data.local_id)
            .await?
            .ok_or(MigrationStateError::Database(sqlx::Error::RowNotFound))
    }

    pub async fn find_by_id_internal(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, MigrationStateError> {
        let record = sqlx::query_as::<_, (Uuid, String, Uuid, Option<Uuid>, String, Option<String>, i64, DateTime<Utc>, DateTime<Utc>)>(
            r#"SELECT id, entity_type, local_id, remote_id, status, error_message, attempt_count, created_at, updated_at
               FROM migration_state WHERE id = $1"#
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(record.map(Self::from_row_tuple))
    }

    pub async fn find_by_entity_internal(pool: &SqlitePool, entity_type: EntityType, local_id: Uuid) -> Result<Option<Self>, MigrationStateError> {
        let entity_type_str = entity_type.to_string();
        let record = sqlx::query_as::<_, (Uuid, String, Uuid, Option<Uuid>, String, Option<String>, i64, DateTime<Utc>, DateTime<Utc>)>(
            r#"SELECT id, entity_type, local_id, remote_id, status, error_message, attempt_count, created_at, updated_at
               FROM migration_state WHERE entity_type = $1 AND local_id = $2"#
        )
        .bind(entity_type_str)
        .bind(local_id)
        .fetch_optional(pool)
        .await?;

        Ok(record.map(Self::from_row_tuple))
    }

    pub async fn mark_migrated(
        pool: &SqlitePool,
        entity_type: EntityType,
        local_id: Uuid,
        remote_id: Uuid,
    ) -> Result<(), MigrationStateError> {
        let entity_type_str = entity_type.to_string();
        sqlx::query(
            r#"UPDATE migration_state
            SET status = 'migrated',
                remote_id = $3,
                error_message = NULL,
                updated_at = datetime('now', 'subsec')
            WHERE entity_type = $1 AND local_id = $2"#
        )
        .bind(entity_type_str)
        .bind(local_id)
        .bind(remote_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn mark_failed(
        pool: &SqlitePool,
        entity_type: EntityType,
        local_id: Uuid,
        error_message: &str,
    ) -> Result<(), MigrationStateError> {
        let entity_type_str = entity_type.to_string();
        sqlx::query(
            r#"UPDATE migration_state
            SET status = 'failed',
                error_message = $3,
                attempt_count = attempt_count + 1,
                updated_at = datetime('now', 'subsec')
            WHERE entity_type = $1 AND local_id = $2"#
        )
        .bind(entity_type_str)
        .bind(local_id)
        .bind(error_message)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn mark_skipped(
        pool: &SqlitePool,
        entity_type: EntityType,
        local_id: Uuid,
        reason: &str,
    ) -> Result<(), MigrationStateError> {
        let entity_type_str = entity_type.to_string();
        sqlx::query(
            r#"UPDATE migration_state
            SET status = 'skipped',
                error_message = $3,
                updated_at = datetime('now', 'subsec')
            WHERE entity_type = $1 AND local_id = $2"#
        )
        .bind(entity_type_str)
        .bind(local_id)
        .bind(reason)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn reset_failed(pool: &SqlitePool) -> Result<u64, MigrationStateError> {
        let result = sqlx::query(
            r#"UPDATE migration_state
            SET status = 'pending',
                error_message = NULL,
                updated_at = datetime('now', 'subsec')
            WHERE status = 'failed'"#
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn get_stats(pool: &SqlitePool) -> Result<MigrationStats, MigrationStateError> {
        let stats = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
            r#"SELECT
                COALESCE(SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END), 0) as pending,
                COALESCE(SUM(CASE WHEN status = 'migrated' THEN 1 ELSE 0 END), 0) as migrated,
                COALESCE(SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END), 0) as failed,
                COALESCE(SUM(CASE WHEN status = 'skipped' THEN 1 ELSE 0 END), 0) as skipped,
                COUNT(*) as total
            FROM migration_state"#
        )
        .fetch_one(pool)
        .await?;

        Ok(MigrationStats {
            pending: stats.0,
            migrated: stats.1,
            failed: stats.2,
            skipped: stats.3,
            total: stats.4,
        })
    }

    pub async fn clear_all(pool: &SqlitePool) -> Result<u64, MigrationStateError> {
        let result = sqlx::query("DELETE FROM migration_state")
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    fn from_row_tuple(row: (Uuid, String, Uuid, Option<Uuid>, String, Option<String>, i64, DateTime<Utc>, DateTime<Utc>)) -> Self {
        let entity_type: EntityType = row.1.parse().unwrap_or(EntityType::Project);
        let status: MigrationStatus = row.4.parse().unwrap_or(MigrationStatus::Pending);

        MigrationState {
            id: row.0,
            entity_type,
            local_id: row.2,
            remote_id: row.3,
            status,
            error_message: row.5,
            attempt_count: row.6,
            created_at: row.7,
            updated_at: row.8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MigrationStats {
    pub pending: i64,
    pub migrated: i64,
    pub failed: i64,
    pub skipped: i64,
    pub total: i64,
}
