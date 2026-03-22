use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, Type};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, TS, Type)]
#[sqlx(type_name = "merge_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MergeStatus {
    Open,
    Merged,
    Closed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Merge {
    Direct(DirectMerge),
    Pr(PrMerge),
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct DirectMerge {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub repo_id: Uuid,
    pub merge_commit: String,
    pub target_branch_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct PrMerge {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub repo_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub target_branch_name: String,
    pub pr_info: PullRequestInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct PullRequestInfo {
    pub number: i64,
    pub url: String,
    pub status: MergeStatus,
    pub merged_at: Option<chrono::DateTime<chrono::Utc>>,
    pub merge_commit_sha: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum MergeType {
    Direct,
    Pr,
}

#[derive(FromRow)]
struct MergeRow {
    id: Uuid,
    workspace_id: Uuid,
    repo_id: Uuid,
    merge_type: MergeType,
    merge_commit: Option<String>,
    target_branch_name: String,
    pr_number: Option<i64>,
    pr_url: Option<String>,
    pr_status: Option<MergeStatus>,
    pr_merged_at: Option<DateTime<Utc>>,
    pr_merge_commit_sha: Option<String>,
    created_at: DateTime<Utc>,
}

impl Merge {
    pub fn merge_commit(&self) -> Option<String> {
        match self {
            Merge::Direct(direct) => Some(direct.merge_commit.clone()),
            Merge::Pr(pr) => pr.pr_info.merge_commit_sha.clone(),
        }
    }

    pub async fn create_direct(
        pool: &SqlitePool,
        workspace_id: Uuid,
        repo_id: Uuid,
        target_branch_name: &str,
        merge_commit: &str,
    ) -> Result<DirectMerge, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO merges (id, workspace_id, repo_id, merge_type, merge_commit, target_branch_name, created_at)
               VALUES ($1, $2, $3, 'direct', $4, $5, $6)"#
        )
        .bind(id)
        .bind(workspace_id)
        .bind(repo_id)
        .bind(merge_commit)
        .bind(target_branch_name)
        .bind(now)
        .execute(pool)
        .await?;

        let merge = Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        match merge {
            Merge::Direct(direct) => Ok(direct),
            Merge::Pr(_) => Err(sqlx::Error::RowNotFound),
        }
    }

    pub async fn create_pr(
        pool: &SqlitePool,
        workspace_id: Uuid,
        repo_id: Uuid,
        target_branch_name: &str,
        pr_number: i64,
        pr_url: &str,
    ) -> Result<PrMerge, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO merges (id, workspace_id, repo_id, merge_type, pr_number, pr_url, pr_status, target_branch_name, created_at)
               VALUES ($1, $2, $3, 'pr', $4, $5, 'open', $6, $7)"#
        )
        .bind(id)
        .bind(workspace_id)
        .bind(repo_id)
        .bind(pr_number)
        .bind(pr_url)
        .bind(target_branch_name)
        .bind(now)
        .execute(pool)
        .await?;

        let merge = Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        match merge {
            Merge::Pr(pr) => Ok(pr),
            Merge::Direct(_) => Err(sqlx::Error::RowNotFound),
        }
    }

    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query_as::<_, MergeRow>(
            r#"SELECT id, workspace_id, repo_id, merge_type, merge_commit, target_branch_name,
                      pr_number, pr_url, pr_status, pr_merged_at, pr_merge_commit_sha, created_at
               FROM merges WHERE id = $1"#
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(Into::into))
    }

    pub async fn find_all_pr(pool: &SqlitePool) -> Result<Vec<PrMerge>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MergeRow>(
            r#"SELECT id, workspace_id, repo_id, merge_type, merge_commit, target_branch_name,
                      pr_number, pr_url, pr_status, pr_merged_at, pr_merge_commit_sha, created_at
               FROM merges WHERE merge_type = 'pr'
               ORDER BY created_at ASC"#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|row| {
            let row: Merge = row.into();
            match row {
                Merge::Pr(pr) => pr,
                Merge::Direct(_) => unreachable!(),
            }
        }).collect())
    }

    pub async fn get_open_prs(pool: &SqlitePool) -> Result<Vec<PrMerge>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MergeRow>(
            r#"SELECT id, workspace_id, repo_id, merge_type, merge_commit, target_branch_name,
                      pr_number, pr_url, pr_status, pr_merged_at, pr_merge_commit_sha, created_at
               FROM merges WHERE merge_type = 'pr' AND pr_status = 'open'
               ORDER BY created_at DESC"#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|row| {
            let row: Merge = row.into();
            match row {
                Merge::Pr(pr) => pr,
                Merge::Direct(_) => unreachable!(),
            }
        }).collect())
    }

    pub async fn count_open_prs_for_workspace(
        pool: &SqlitePool,
        workspace_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(1) FROM merges
               WHERE workspace_id = $1 AND merge_type = 'pr' AND pr_status = 'open'"#
        )
        .bind(workspace_id)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }

    pub async fn update_status(
        pool: &SqlitePool,
        merge_id: Uuid,
        pr_status: MergeStatus,
        merge_commit_sha: Option<String>,
    ) -> Result<(), sqlx::Error> {
        let merged_at = if matches!(pr_status, MergeStatus::Merged) {
            Some(Utc::now())
        } else {
            None
        };

        let pr_status_str = match pr_status {
            MergeStatus::Open => "open",
            MergeStatus::Merged => "merged",
            MergeStatus::Closed => "closed",
            MergeStatus::Unknown => "unknown",
        };

        sqlx::query(
            r#"UPDATE merges SET pr_status = $1, pr_merge_commit_sha = $2, pr_merged_at = $3 WHERE id = $4"#
        )
        .bind(pr_status_str)
        .bind(&merge_commit_sha)
        .bind(merged_at)
        .bind(merge_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_workspace_id(
        pool: &SqlitePool,
        workspace_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MergeRow>(
            r#"SELECT id, workspace_id, repo_id, merge_type, merge_commit, target_branch_name,
                      pr_number, pr_url, pr_status, pr_merged_at, pr_merge_commit_sha, created_at
               FROM merges WHERE workspace_id = $1
               ORDER BY created_at DESC"#
        )
        .bind(workspace_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn find_by_workspace_and_repo_id(
        pool: &SqlitePool,
        workspace_id: Uuid,
        repo_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MergeRow>(
            r#"SELECT id, workspace_id, repo_id, merge_type, merge_commit, target_branch_name,
                      pr_number, pr_url, pr_status, pr_merged_at, pr_merge_commit_sha, created_at
               FROM merges WHERE workspace_id = $1 AND repo_id = $2
               ORDER BY created_at DESC"#
        )
        .bind(workspace_id)
        .bind(repo_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn get_latest_pr_status_for_workspaces(
        pool: &SqlitePool,
        archived: bool,
    ) -> Result<HashMap<Uuid, PrMerge>, sqlx::Error> {
        let archived_int = if archived { 1 } else { 0 };

        let rows = sqlx::query_as::<_, MergeRow>(
            r#"SELECT m.id, m.workspace_id, m.repo_id, m.merge_type, m.merge_commit, m.target_branch_name,
                      m.pr_number, m.pr_url, m.pr_status, m.pr_merged_at, m.pr_merge_commit_sha, m.created_at
               FROM merges m
               INNER JOIN (
                   SELECT workspace_id, MAX(created_at) as max_created_at
                   FROM merges
                   WHERE merge_type = 'pr'
                   GROUP BY workspace_id
               ) latest ON m.workspace_id = latest.workspace_id AND m.created_at = latest.max_created_at
               INNER JOIN workspaces w ON m.workspace_id = w.id
               WHERE m.merge_type = 'pr' AND w.archived = $1"#
        )
        .bind(archived_int)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let workspace_id = row.workspace_id;
                let pr_merge = match Merge::from(row) {
                    Merge::Pr(pr) => pr,
                    Merge::Direct(_) => unreachable!(),
                };
                (workspace_id, pr_merge)
            })
            .collect())
    }
}

impl From<MergeRow> for DirectMerge {
    fn from(row: MergeRow) -> Self {
        DirectMerge {
            id: row.id,
            workspace_id: row.workspace_id,
            repo_id: row.repo_id,
            merge_commit: row
                .merge_commit
                .expect("direct merge must have merge_commit"),
            target_branch_name: row.target_branch_name,
            created_at: row.created_at,
        }
    }
}

impl From<MergeRow> for PrMerge {
    fn from(row: MergeRow) -> Self {
        PrMerge {
            id: row.id,
            workspace_id: row.workspace_id,
            repo_id: row.repo_id,
            target_branch_name: row.target_branch_name,
            pr_info: PullRequestInfo {
                number: row.pr_number.expect("pr merge must have pr_number"),
                url: row.pr_url.expect("pr merge must have pr_url"),
                status: row.pr_status.expect("pr merge must have status"),
                merged_at: row.pr_merged_at,
                merge_commit_sha: row.pr_merge_commit_sha,
            },
            created_at: row.created_at,
        }
    }
}

impl From<MergeRow> for Merge {
    fn from(row: MergeRow) -> Self {
        match row.merge_type {
            MergeType::Direct => Merge::Direct(DirectMerge::from(row)),
            MergeType::Pr => Merge::Pr(PrMerge::from(row)),
        }
    }
}
