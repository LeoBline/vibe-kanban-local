use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
use sqlx::{FromRow, SqlitePool};
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

pub type RepoTuple = (Uuid, String, String, String, Option<String>, Option<String>, Option<String>, Option<String>, bool, Option<String>, Option<String>, Option<String>, DateTime<Utc>, DateTime<Utc>);

#[derive(Debug, Serialize, TS)]
pub struct SearchResult {
    pub path: String,
    pub is_file: bool,
    pub match_type: SearchMatchType,
    #[serde(default)]
    pub score: i64,
}

#[derive(Debug, Clone, Serialize, TS)]
pub enum SearchMatchType {
    FileName,
    DirectoryName,
    FullPath,
}

#[derive(Debug, Error)]
pub enum RepoError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("Repository not found")]
    NotFound,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct Repo {
    pub id: Uuid,
    pub path: PathBuf,
    pub name: String,
    pub display_name: String,
    pub setup_script: Option<String>,
    pub cleanup_script: Option<String>,
    pub archive_script: Option<String>,
    pub copy_files: Option<String>,
    pub parallel_setup_script: bool,
    pub dev_server_script: Option<String>,
    pub default_target_branch: Option<String>,
    pub default_working_dir: Option<String>,
    #[ts(type = "Date")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "Date")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, TS)]
pub struct UpdateRepo {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    #[ts(optional, type = "string | null")]
    pub display_name: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    #[ts(optional, type = "string | null")]
    pub setup_script: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    #[ts(optional, type = "string | null")]
    pub cleanup_script: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    #[ts(optional, type = "string | null")]
    pub archive_script: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    #[ts(optional, type = "string | null")]
    pub copy_files: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    #[ts(optional, type = "boolean | null")]
    pub parallel_setup_script: Option<Option<bool>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    #[ts(optional, type = "string | null")]
    pub dev_server_script: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    #[ts(optional, type = "string | null")]
    pub default_target_branch: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "double_option"
    )]
    #[ts(optional, type = "string | null")]
    pub default_working_dir: Option<Option<String>>,
}

impl Repo {
    fn from_tuple(t: RepoTuple) -> Self {
        Repo {
            id: t.0,
            path: PathBuf::from(t.1),
            name: t.2,
            display_name: t.3,
            setup_script: t.4,
            cleanup_script: t.5,
            archive_script: t.6,
            copy_files: t.7,
            parallel_setup_script: t.8,
            dev_server_script: t.9,
            default_target_branch: t.10,
            default_working_dir: t.11,
            created_at: t.12,
            updated_at: t.13,
        }
    }

    pub async fn list_needing_name_fix(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RepoTuple>(
            r#"SELECT id, path, name, display_name, setup_script, cleanup_script,
                      archive_script, copy_files, parallel_setup_script, dev_server_script,
                      default_target_branch, default_working_dir, created_at, updated_at
               FROM repos
               WHERE name = '__NEEDS_BACKFILL__'"#,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(Self::from_tuple).collect())
    }

    pub async fn update_name(
        pool: &SqlitePool,
        id: Uuid,
        name: &str,
        display_name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE repos SET name = $1, display_name = $2, updated_at = datetime('now', 'subsec') WHERE id = $3",
        )
        .bind(name)
        .bind(display_name)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query_as::<_, RepoTuple>(
            r#"SELECT id, path, name, display_name, setup_script, cleanup_script,
                      archive_script, copy_files, parallel_setup_script, dev_server_script,
                      default_target_branch, default_working_dir, created_at, updated_at
               FROM repos
               WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(row.map(Self::from_tuple))
    }

    pub async fn find_by_ids(pool: &SqlitePool, ids: &[Uuid]) -> Result<Vec<Self>, sqlx::Error> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut repos = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(repo) = Self::find_by_id(pool, *id).await? {
                repos.push(repo);
            }
        }
        Ok(repos)
    }

    pub async fn find_or_create(
        pool: &SqlitePool,
        path: &Path,
        display_name: &str,
    ) -> Result<Self, sqlx::Error> {
        let path_str = path.to_string_lossy().to_string();
        let id = Uuid::new_v4();
        let repo_name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| id.to_string());

        sqlx::query(
            r#"INSERT OR IGNORE INTO repos (id, path, name, display_name, created_at, updated_at)
               VALUES ($1, $2, $3, $4, datetime('now', 'subsec'), datetime('now', 'subsec'))"#,
        )
        .bind(id)
        .bind(&path_str)
        .bind(&repo_name)
        .bind(display_name)
        .execute(pool)
        .await?;

        Self::find_by_path(pool, &path_str).await
    }

    async fn find_by_path(pool: &SqlitePool, path: &str) -> Result<Self, sqlx::Error> {
        let row = sqlx::query_as::<_, RepoTuple>(
            r#"SELECT id, path, name, display_name, setup_script, cleanup_script,
                      archive_script, copy_files, parallel_setup_script, dev_server_script,
                      default_target_branch, default_working_dir, created_at, updated_at
               FROM repos WHERE path = $1"#,
        )
        .bind(path)
        .fetch_one(pool)
        .await?;
        Ok(Self::from_tuple(row))
    }

    pub async fn delete_orphaned(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"DELETE FROM repos
               WHERE id NOT IN (SELECT repo_id FROM project_repos)
                 AND id NOT IN (SELECT repo_id FROM workspace_repos)"#,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RepoTuple>(
            r#"SELECT id, path, name, display_name, setup_script, cleanup_script,
                      archive_script, copy_files, parallel_setup_script, dev_server_script,
                      default_target_branch, default_working_dir, created_at, updated_at
               FROM repos
               ORDER BY display_name ASC"#,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(Self::from_tuple).collect())
    }

    pub async fn list_by_recent_workspace_usage(
        pool: &SqlitePool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RepoTuple>(
            r#"SELECT r.id, r.path, r.name, r.display_name, r.setup_script, r.cleanup_script,
                      r.archive_script, r.copy_files, r.parallel_setup_script, r.dev_server_script,
                      r.default_target_branch, r.default_working_dir, r.created_at, r.updated_at
               FROM repos r
               LEFT JOIN (
                   SELECT repo_id, MAX(updated_at) AS last_used_at
                   FROM workspace_repos
                   GROUP BY repo_id
               ) wr ON wr.repo_id = r.id
               ORDER BY wr.last_used_at DESC, r.display_name ASC"#,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(Self::from_tuple).collect())
    }

    pub async fn active_workspace_names(
        pool: &SqlitePool,
        repo_id: Uuid,
    ) -> Result<Vec<String>, sqlx::Error> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"SELECT w.name
               FROM workspaces w
               JOIN workspace_repos wr ON wr.workspace_id = w.id
               WHERE wr.repo_id = $1 AND w.archived = 0"#,
        )
        .bind(repo_id)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(name,)| name)
            .collect())
    }

    pub async fn delete(pool: &SqlitePool, id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM repos WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn update(
        pool: &SqlitePool,
        id: Uuid,
        payload: &UpdateRepo,
    ) -> Result<Self, RepoError> {
        let existing = Self::find_by_id(pool, id)
            .await?
            .ok_or(RepoError::NotFound)?;

        let display_name = match &payload.display_name {
            None => existing.display_name,
            Some(v) => v.clone().unwrap_or_default(),
        };
        let setup_script = match &payload.setup_script {
            None => existing.setup_script,
            Some(v) => v.clone(),
        };
        let cleanup_script = match &payload.cleanup_script {
            None => existing.cleanup_script,
            Some(v) => v.clone(),
        };
        let archive_script = match &payload.archive_script {
            None => existing.archive_script,
            Some(v) => v.clone(),
        };
        let copy_files = match &payload.copy_files {
            None => existing.copy_files,
            Some(v) => v.clone(),
        };
        let parallel_setup_script = match &payload.parallel_setup_script {
            None => existing.parallel_setup_script,
            Some(v) => v.unwrap_or(false),
        };
        let dev_server_script = match &payload.dev_server_script {
            None => existing.dev_server_script,
            Some(v) => v.clone(),
        };
        let default_target_branch = match &payload.default_target_branch {
            None => existing.default_target_branch,
            Some(v) => v.clone(),
        };
        let default_working_dir = match &payload.default_working_dir {
            None => existing.default_working_dir,
            Some(v) => v.clone(),
        };

        sqlx::query(
            r#"UPDATE repos
               SET display_name = $1,
                   setup_script = $2,
                   cleanup_script = $3,
                   archive_script = $4,
                   copy_files = $5,
                   parallel_setup_script = $6,
                   dev_server_script = $7,
                   default_target_branch = $8,
                   default_working_dir = $9,
                   updated_at = datetime('now', 'subsec')
               WHERE id = $10"#,
        )
        .bind(&display_name)
        .bind(&setup_script)
        .bind(&cleanup_script)
        .bind(&archive_script)
        .bind(&copy_files)
        .bind(parallel_setup_script)
        .bind(&dev_server_script)
        .bind(&default_target_branch)
        .bind(&default_working_dir)
        .bind(id)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, id)
            .await?
            .ok_or(RepoError::NotFound)
    }
}
