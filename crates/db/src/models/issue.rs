use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use ts_rs::TS;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct Issue {
    pub id: String,
    pub project_id: String,
    pub issue_number: i64,
    pub simple_id: String,
    pub status_id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub sort_order: i64,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub completed_at: Option<String>,
    pub parent_issue_id: Option<String>,
    pub parent_issue_sort_order: Option<i64>,
    pub extension_metadata: Option<String>,
    pub creator_user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct IssueWithRelations {
    #[serde(flatten)]
    pub issue: Issue,
    pub assignees: Vec<IssueAssignee>,
    pub followers: Vec<IssueFollower>,
    pub tags: Vec<String>,
    pub relationships: Vec<IssueRelationship>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct IssueAssignee {
    pub id: String,
    pub issue_id: String,
    pub user_id: String,
    pub project_id: String,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct IssueFollower {
    pub id: String,
    pub issue_id: String,
    pub user_id: String,
    pub project_id: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct IssueRelationship {
    pub id: String,
    pub issue_id: String,
    pub related_issue_id: String,
    pub relationship_type: String,
    pub project_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateIssue {
    pub project_id: String,
    pub status_id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub sort_order: Option<i64>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub completed_at: Option<String>,
    pub parent_issue_id: Option<String>,
    pub parent_issue_sort_order: Option<i64>,
    pub extension_metadata: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, TS)]
pub struct UpdateIssue {
    pub status_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub sort_order: Option<i64>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub completed_at: Option<String>,
    pub parent_issue_id: Option<String>,
    pub parent_issue_sort_order: Option<i64>,
    pub extension_metadata: Option<String>,
}

impl Issue {
    pub async fn find_by_project(pool: &SqlitePool, project_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Issue>(
            r#"SELECT id, project_id, issue_number, simple_id, status_id, title, description, priority, sort_order, start_date, target_date, completed_at, parent_issue_id, parent_issue_sort_order, extension_metadata, creator_user_id, created_at, updated_at
               FROM issues
               WHERE project_id = ?
               ORDER BY sort_order ASC, created_at DESC"#,
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_status(pool: &SqlitePool, status_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Issue>(
            r#"SELECT id, project_id, issue_number, simple_id, status_id, title, description, priority, sort_order, start_date, target_date, completed_at, parent_issue_id, parent_issue_sort_order, extension_metadata, creator_user_id, created_at, updated_at
               FROM issues
               WHERE status_id = ?
               ORDER BY sort_order ASC, created_at DESC"#,
        )
        .bind(status_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Issue>(
            r#"SELECT id, project_id, issue_number, simple_id, status_id, title, description, priority, sort_order, start_date, target_date, completed_at, parent_issue_id, parent_issue_sort_order, extension_metadata, creator_user_id, created_at, updated_at
               FROM issues
               WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, data: &CreateIssue) -> Result<Self, sqlx::Error> {
        let id = format!("local-issue-{}", chrono::Utc::now().timestamp_millis());
        let priority = data.priority.clone().unwrap_or_else(|| "medium".to_string());
        
        let max_sort_order: Option<(i64,)> = sqlx::query_as(
            "SELECT COALESCE(MAX(sort_order), 0) FROM issues WHERE status_id = ?"
        )
        .bind(&data.status_id)
        .fetch_optional(pool)
        .await?;
        
        let max_issue_number: Option<(i64,)> = sqlx::query_as(
            "SELECT COALESCE(MAX(issue_number), 0) FROM issues WHERE project_id = ?"
        )
        .bind(&data.project_id)
        .fetch_optional(pool)
        .await?;
        
        let issue_number = max_issue_number.map(|r| r.0 + 1).unwrap_or(1);
        let simple_id = format!("{}", issue_number);
        let sort_order: i64 = data.sort_order.unwrap_or(max_sort_order.map(|r| r.0 + 1).unwrap_or(0));

        sqlx::query(
            r#"INSERT INTO issues (id, project_id, issue_number, simple_id, status_id, title, description, priority, sort_order, start_date, target_date, completed_at, parent_issue_id, parent_issue_sort_order, extension_metadata)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&id)
        .bind(&data.project_id)
        .bind(issue_number)
        .bind(&simple_id)
        .bind(&data.status_id)
        .bind(&data.title)
        .bind(&data.description)
        .bind(&priority)
        .bind(sort_order)
        .bind(&data.start_date)
        .bind(&data.target_date)
        .bind(&data.completed_at)
        .bind(&data.parent_issue_id)
        .bind(data.parent_issue_sort_order)
        .bind(&data.extension_metadata)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, &id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn update(pool: &SqlitePool, id: &str, data: &UpdateIssue) -> Result<Self, sqlx::Error> {
        let existing = Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        let status_id = data.status_id.clone().unwrap_or(existing.status_id.clone());
        let title = data.title.clone().unwrap_or(existing.title.clone());
        let description = data.description.clone().or(existing.description.clone());
        let priority = data.priority.clone().unwrap_or(existing.priority.clone().unwrap_or_else(|| "medium".to_string()));
        let sort_order = data.sort_order.unwrap_or(existing.sort_order);
        let start_date = data.start_date.clone().or(existing.start_date.clone());
        let target_date = data.target_date.clone().or(existing.target_date.clone());
        let completed_at = data.completed_at.clone().or(existing.completed_at.clone());
        let parent_issue_id = data.parent_issue_id.clone().or(existing.parent_issue_id.clone());
        let parent_issue_sort_order = data.parent_issue_sort_order.or(existing.parent_issue_sort_order);
        let extension_metadata = data.extension_metadata.clone().or(existing.extension_metadata.clone());

        sqlx::query(
            r#"UPDATE issues
               SET status_id = ?, title = ?, description = ?, priority = ?, sort_order = ?, 
                   start_date = ?, target_date = ?, completed_at = ?, parent_issue_id = ?, 
                   parent_issue_sort_order = ?, extension_metadata = ?, updated_at = datetime('now', 'subsec')
               WHERE id = ?"#,
        )
        .bind(&status_id)
        .bind(&title)
        .bind(&description)
        .bind(&priority)
        .bind(sort_order)
        .bind(&start_date)
        .bind(&target_date)
        .bind(&completed_at)
        .bind(&parent_issue_id)
        .bind(parent_issue_sort_order)
        .bind(&extension_metadata)
        .bind(id)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM issues WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn get_assignees(pool: &SqlitePool, issue_id: &str) -> Result<Vec<IssueAssignee>, sqlx::Error> {
        sqlx::query_as::<_, IssueAssignee>(
            r#"SELECT id, issue_id, user_id, project_id, assigned_at
               FROM issue_assignees
               WHERE issue_id = ?"#,
        )
        .bind(issue_id)
        .fetch_all(pool)
        .await
    }

    pub async fn get_followers(pool: &SqlitePool, issue_id: &str) -> Result<Vec<IssueFollower>, sqlx::Error> {
        sqlx::query_as::<_, IssueFollower>(
            r#"SELECT id, issue_id, user_id, project_id
               FROM issue_followers
               WHERE issue_id = ?"#,
        )
        .bind(issue_id)
        .fetch_all(pool)
        .await
    }

    pub async fn get_tags(pool: &SqlitePool, issue_id: &str) -> Result<Vec<String>, sqlx::Error> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"SELECT t.name FROM tags t
               INNER JOIN issue_tags it ON t.id = it.tag_id
               WHERE it.issue_id = ?"#,
        )
        .bind(issue_id)
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    pub async fn get_relationships(pool: &SqlitePool, issue_id: &str) -> Result<Vec<IssueRelationship>, sqlx::Error> {
        sqlx::query_as::<_, IssueRelationship>(
            r#"SELECT id, issue_id, related_issue_id, relationship_type, project_id, created_at
               FROM issue_relationships
               WHERE issue_id = ?"#,
        )
        .bind(issue_id)
        .fetch_all(pool)
        .await
    }

    pub async fn get_with_relations(pool: &SqlitePool, id: &str) -> Result<Option<IssueWithRelations>, sqlx::Error> {
        let issue = Self::find_by_id(pool, id).await?;
        
        match issue {
            Some(issue) => {
                let assignees = Self::get_assignees(pool, id).await?;
                let followers = Self::get_followers(pool, id).await?;
                let tags = Self::get_tags(pool, id).await?;
                let relationships = Self::get_relationships(pool, id).await?;
                
                Ok(Some(IssueWithRelations {
                    issue,
                    assignees,
                    followers,
                    tags,
                    relationships,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn update_status(pool: &SqlitePool, id: &str, status_id: &str) -> Result<Self, sqlx::Error> {
        let data = UpdateIssue {
            status_id: Some(status_id.to_string()),
            title: None,
            description: None,
            priority: None,
            sort_order: None,
            start_date: None,
            target_date: None,
            completed_at: None,
            parent_issue_id: None,
            parent_issue_sort_order: None,
            extension_metadata: None,
        };
        Self::update(pool, id, &data).await
    }

    pub async fn get_next_sort_order(pool: &SqlitePool, status_id: &str) -> Result<i64, sqlx::Error> {
        let result: Option<(i64,)> = sqlx::query_as(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM issues WHERE status_id = ?"
        )
        .bind(status_id)
        .fetch_optional(pool)
        .await?;
        Ok(result.map(|r| r.0).unwrap_or(0))
    }
}
