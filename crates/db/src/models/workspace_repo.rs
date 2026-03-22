use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use ts_rs::TS;
use uuid::Uuid;

use super::repo::Repo;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct WorkspaceRepo {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub repo_id: Uuid,
    pub target_branch: String,
    #[ts(type = "Date")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "Date")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, TS)]
pub struct CreateWorkspaceRepo {
    pub repo_id: Uuid,
    pub target_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct RepoWithTargetBranch {
    #[serde(flatten)]
    pub repo: Repo,
    pub target_branch: String,
}

#[derive(Debug, Clone)]
pub struct RepoWithCopyFiles {
    pub id: Uuid,
    pub path: PathBuf,
    pub name: String,
    pub copy_files: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
struct RepoRow {
    pub id: Uuid,
    pub path: String,
    pub name: String,
    pub display_name: Option<String>,
    pub setup_script: Option<String>,
    pub cleanup_script: Option<String>,
    pub archive_script: Option<String>,
    pub copy_files: Option<String>,
    pub parallel_setup_script: bool,
    pub dev_server_script: Option<String>,
    pub default_target_branch: Option<String>,
    pub default_working_dir: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
struct RepoWithTargetBranchRow {
    pub id: Uuid,
    pub path: String,
    pub name: String,
    pub display_name: Option<String>,
    pub setup_script: Option<String>,
    pub cleanup_script: Option<String>,
    pub archive_script: Option<String>,
    pub copy_files: Option<String>,
    pub parallel_setup_script: bool,
    pub dev_server_script: Option<String>,
    pub default_target_branch: Option<String>,
    pub default_working_dir: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub target_branch: String,
}

#[derive(Debug, Clone, FromRow)]
struct RepoWithCopyFilesRow {
    pub id: Uuid,
    pub path: String,
    pub name: String,
    pub copy_files: Option<String>,
}

impl From<RepoRow> for Repo {
    fn from(row: RepoRow) -> Self {
        Repo {
            id: row.id,
            path: PathBuf::from(row.path),
            name: row.name.clone(),
            display_name: row.display_name.unwrap_or(row.name),
            setup_script: row.setup_script,
            cleanup_script: row.cleanup_script,
            archive_script: row.archive_script,
            copy_files: row.copy_files,
            parallel_setup_script: row.parallel_setup_script,
            dev_server_script: row.dev_server_script,
            default_target_branch: row.default_target_branch,
            default_working_dir: row.default_working_dir,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<RepoWithTargetBranchRow> for RepoWithTargetBranch {
    fn from(row: RepoWithTargetBranchRow) -> Self {
        RepoWithTargetBranch {
            repo: Repo::from(RepoRow {
                id: row.id,
                path: row.path,
                name: row.name,
                display_name: row.display_name,
                setup_script: row.setup_script,
                cleanup_script: row.cleanup_script,
                archive_script: row.archive_script,
                copy_files: row.copy_files,
                parallel_setup_script: row.parallel_setup_script,
                dev_server_script: row.dev_server_script,
                default_target_branch: row.default_target_branch,
                default_working_dir: row.default_working_dir,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }),
            target_branch: row.target_branch,
        }
    }
}

impl From<RepoWithCopyFilesRow> for RepoWithCopyFiles {
    fn from(row: RepoWithCopyFilesRow) -> Self {
        RepoWithCopyFiles {
            id: row.id,
            path: PathBuf::from(row.path),
            name: row.name,
            copy_files: row.copy_files,
        }
    }
}

impl WorkspaceRepo {
    pub async fn create_many(
        pool: &SqlitePool,
        workspace_id: Uuid,
        repos: &[CreateWorkspaceRepo],
    ) -> Result<Vec<Self>, sqlx::Error> {
        if repos.is_empty() {
            return Ok(Vec::new());
        }

        let mut tx = pool.begin().await?;
        let mut results = Vec::with_capacity(repos.len());

        for repo in repos {
            let id = Uuid::new_v4();
            sqlx::query(
                r#"INSERT INTO workspace_repos (id, workspace_id, repo_id, target_branch, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, datetime('now', 'subsec'), datetime('now', 'subsec'))"#,
            )
            .bind(id)
            .bind(workspace_id)
            .bind(repo.repo_id)
            .bind(&repo.target_branch)
            .execute(&mut *tx)
            .await?;

            let row = sqlx::query_as::<_, WorkspaceRepo>(
                r#"SELECT id, workspace_id, repo_id, target_branch, created_at, updated_at
                   FROM workspace_repos WHERE id = $1"#,
            )
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
            results.push(row);
        }

        tx.commit().await?;
        Ok(results)
    }

    pub async fn find_by_workspace_id(
        pool: &SqlitePool,
        workspace_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, WorkspaceRepo>(
            r#"SELECT id, workspace_id, repo_id, target_branch, created_at, updated_at
               FROM workspace_repos
               WHERE workspace_id = $1"#,
        )
        .bind(workspace_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_repos_for_workspace(
        pool: &SqlitePool,
        workspace_id: Uuid,
    ) -> Result<Vec<Repo>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RepoRow>(
            r#"SELECT r.id, r.path, r.name, r.display_name, r.setup_script,
                      r.cleanup_script, r.archive_script, r.copy_files,
                      r.parallel_setup_script, r.dev_server_script,
                      r.default_target_branch, r.default_working_dir,
                      r.created_at, r.updated_at
               FROM repos r
               JOIN workspace_repos wr ON r.id = wr.repo_id
               WHERE wr.workspace_id = $1
               ORDER BY r.display_name ASC"#,
        )
        .bind(workspace_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Repo::from).collect())
    }

    pub async fn find_repos_with_target_branch_for_workspace(
        pool: &SqlitePool,
        workspace_id: Uuid,
    ) -> Result<Vec<RepoWithTargetBranch>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RepoWithTargetBranchRow>(
            r#"SELECT r.id, r.path, r.name, r.display_name, r.setup_script,
                      r.cleanup_script, r.archive_script, r.copy_files,
                      r.parallel_setup_script, r.dev_server_script,
                      r.default_target_branch, r.default_working_dir,
                      r.created_at, r.updated_at, wr.target_branch
               FROM repos r
               JOIN workspace_repos wr ON r.id = wr.repo_id
               WHERE wr.workspace_id = $1
               ORDER BY r.display_name ASC"#,
        )
        .bind(workspace_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(RepoWithTargetBranch::from).collect())
    }

    pub async fn find_by_workspace_and_repo_id(
        pool: &SqlitePool,
        workspace_id: Uuid,
        repo_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, WorkspaceRepo>(
            r#"SELECT id, workspace_id, repo_id, target_branch, created_at, updated_at
               FROM workspace_repos
               WHERE workspace_id = $1 AND repo_id = $2"#,
        )
        .bind(workspace_id)
        .bind(repo_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update_target_branch(
        pool: &SqlitePool,
        workspace_id: Uuid,
        repo_id: Uuid,
        new_target_branch: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE workspace_repos SET target_branch = $1, updated_at = datetime('now', 'subsec') WHERE workspace_id = $2 AND repo_id = $3",
        )
        .bind(new_target_branch)
        .bind(workspace_id)
        .bind(repo_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn update_target_branch_for_children_of_workspace(
        pool: &SqlitePool,
        parent_workspace_id: Uuid,
        old_branch: &str,
        new_branch: &str,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE workspace_repos
               SET target_branch = $1, updated_at = datetime('now', 'subsec')
               WHERE target_branch = $2
                 AND workspace_id IN (
                     SELECT w.id FROM workspaces w
                     JOIN tasks t ON w.task_id = t.id
                     WHERE t.parent_workspace_id = $3
                 )"#,
        )
        .bind(new_branch)
        .bind(old_branch)
        .bind(parent_workspace_id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_unique_repos_for_task(
        pool: &SqlitePool,
        task_id: Uuid,
    ) -> Result<Vec<Repo>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RepoRow>(
            r#"SELECT DISTINCT r.id, r.path, r.name, r.display_name, r.setup_script,
                      r.cleanup_script, r.archive_script, r.copy_files,
                      r.parallel_setup_script, r.dev_server_script,
                      r.default_target_branch, r.default_working_dir,
                      r.created_at, r.updated_at
               FROM repos r
               JOIN workspace_repos wr ON r.id = wr.repo_id
               JOIN workspaces w ON wr.workspace_id = w.id
               WHERE w.task_id = $1
               ORDER BY r.display_name ASC"#,
        )
        .bind(task_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Repo::from).collect())
    }

    pub async fn find_repos_with_copy_files(
        pool: &SqlitePool,
        workspace_id: Uuid,
    ) -> Result<Vec<RepoWithCopyFiles>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RepoWithCopyFilesRow>(
            r#"SELECT r.id, r.path, r.name, r.copy_files
               FROM repos r
               JOIN workspace_repos wr ON r.id = wr.repo_id
               WHERE wr.workspace_id = $1"#,
        )
        .bind(workspace_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(RepoWithCopyFiles::from).collect())
    }
}
