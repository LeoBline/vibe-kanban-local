use chrono::{DateTime, Utc};
use executors::profile::ExecutorConfig;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use strum_macros::{Display, EnumDiscriminants, EnumString};
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ScratchError {
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("Scratch type mismatch: expected '{expected}' but got '{actual}'")]
    TypeMismatch { expected: String, actual: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct DraftFollowUpData {
    pub message: String,
    #[serde(alias = "executor_profile_id", alias = "config")]
    pub executor_config: ExecutorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct PreviewSettingsData {
    pub url: String,
    #[serde(default)]
    pub screen_size: Option<String>,
    #[serde(default)]
    pub responsive_width: Option<i32>,
    #[serde(default)]
    pub responsive_height: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct WorkspaceNotesData {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct WorkspacePanelStateData {
    pub right_main_panel_mode: Option<String>,
    pub is_left_main_panel_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkspacePrFilterData {
    #[default]
    All,
    HasPr,
    NoPr,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceSortByData {
    #[default]
    UpdatedAt,
    CreatedAt,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceSortOrderData {
    Asc,
    #[default]
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, Default)]
pub struct WorkspaceFilterStateData {
    #[serde(default)]
    pub project_ids: Vec<String>,
    #[serde(default)]
    pub pr_filter: WorkspacePrFilterData,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, Default)]
pub struct WorkspaceSortStateData {
    #[serde(default)]
    pub sort_by: WorkspaceSortByData,
    #[serde(default)]
    pub sort_order: WorkspaceSortOrderData,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct UiPreferencesData {
    #[serde(default)]
    pub repo_actions: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub expanded: std::collections::HashMap<String, bool>,
    #[serde(default)]
    pub context_bar_position: Option<String>,
    #[serde(default)]
    pub pane_sizes: std::collections::HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub collapsed_paths: std::collections::HashMap<String, Vec<String>>,
    #[serde(default)]
    pub file_search_repo_id: Option<String>,
    #[serde(default)]
    pub is_left_sidebar_visible: Option<bool>,
    #[serde(default)]
    pub is_right_sidebar_visible: Option<bool>,
    #[serde(default)]
    pub is_terminal_visible: Option<bool>,
    #[serde(default)]
    pub workspace_panel_states: std::collections::HashMap<String, WorkspacePanelStateData>,
    #[serde(default)]
    pub workspace_filters: WorkspaceFilterStateData,
    #[serde(default)]
    pub workspace_sort: WorkspaceSortStateData,
    #[serde(default)]
    pub selected_org_id: Option<String>,
    #[serde(default)]
    pub selected_project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct DraftWorkspaceLinkedIssue {
    pub issue_id: String,
    pub simple_id: String,
    pub title: String,
    pub remote_project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct DraftWorkspaceImage {
    pub id: Uuid,
    pub file_path: String,
    pub original_name: String,
    #[serde(default)]
    pub mime_type: Option<String>,
    pub size_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct DraftWorkspaceData {
    pub message: String,
    #[serde(default)]
    pub repos: Vec<DraftWorkspaceRepo>,
    #[serde(default, alias = "selected_profile", alias = "config")]
    pub executor_config: Option<ExecutorConfig>,
    #[serde(default)]
    pub linked_issue: Option<DraftWorkspaceLinkedIssue>,
    #[serde(default)]
    pub images: Vec<DraftWorkspaceImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct DraftWorkspaceRepo {
    pub repo_id: Uuid,
    pub target_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ProjectRepoDefaultsData {
    pub repos: Vec<DraftWorkspaceRepo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct DraftIssueData {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub status_id: String,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub assignee_ids: Vec<String>,
    #[serde(default)]
    pub tag_ids: Vec<String>,
    #[serde(default)]
    pub create_draft_workspace: bool,
    pub project_id: String,
    #[serde(default)]
    pub parent_issue_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, EnumDiscriminants)]
#[serde(tag = "type", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
#[strum_discriminants(name(ScratchType))]
#[strum_discriminants(derive(Display, EnumString, Serialize, Deserialize, TS))]
#[strum_discriminants(ts(use_ts_enum))]
#[strum_discriminants(serde(rename_all = "SCREAMING_SNAKE_CASE"))]
#[strum_discriminants(strum(serialize_all = "SCREAMING_SNAKE_CASE"))]
pub enum ScratchPayload {
    DraftTask(String),
    DraftFollowUp(DraftFollowUpData),
    DraftWorkspace(DraftWorkspaceData),
    DraftIssue(DraftIssueData),
    PreviewSettings(PreviewSettingsData),
    WorkspaceNotes(WorkspaceNotesData),
    UiPreferences(UiPreferencesData),
    ProjectRepoDefaults(ProjectRepoDefaultsData),
}

impl ScratchPayload {
    pub fn scratch_type(&self) -> ScratchType {
        ScratchType::from(self)
    }

    pub fn validate_type(&self, expected: ScratchType) -> Result<(), ScratchError> {
        let actual = self.scratch_type();
        if actual != expected {
            return Err(ScratchError::TypeMismatch {
                expected: expected.to_string(),
                actual: actual.to_string(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, FromRow)]
struct ScratchRow {
    pub id: Uuid,
    pub scratch_type: String,
    pub payload: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct Scratch {
    pub id: Uuid,
    pub payload: ScratchPayload,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Scratch {
    pub fn scratch_type(&self) -> ScratchType {
        self.payload.scratch_type()
    }
}

impl TryFrom<ScratchRow> for Scratch {
    type Error = ScratchError;
    fn try_from(r: ScratchRow) -> Result<Self, ScratchError> {
        let payload: ScratchPayload = serde_json::from_str(&r.payload)?;
        payload.validate_type(r.scratch_type.parse().map_err(|_| {
            ScratchError::TypeMismatch {
                expected: r.scratch_type.clone(),
                actual: payload.scratch_type().to_string(),
            }
        })?)?;
        Ok(Scratch {
            id: r.id,
            payload,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct CreateScratch {
    pub payload: ScratchPayload,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct UpdateScratch {
    pub payload: ScratchPayload,
}

impl Scratch {
    pub async fn create(
        pool: &SqlitePool,
        id: Uuid,
        data: &CreateScratch,
    ) -> Result<Self, ScratchError> {
        let scratch_type_str = data.payload.scratch_type().to_string();
        let payload_str = serde_json::to_string(&data.payload)?;

        sqlx::query(
            r#"INSERT INTO scratch (id, scratch_type, payload, created_at, updated_at)
               VALUES ($1, $2, $3, datetime('now', 'subsec'), datetime('now', 'subsec'))"#,
        )
        .bind(id)
        .bind(&scratch_type_str)
        .bind(&payload_str)
        .execute(pool)
        .await?;

        Self::find_by_id_internal(pool, id, &scratch_type_str).await
    }

    async fn find_by_id_internal(
        pool: &SqlitePool,
        id: Uuid,
        scratch_type: &str,
    ) -> Result<Self, ScratchError> {
        let row = sqlx::query_as::<_, ScratchRow>(
            r#"SELECT id, scratch_type, payload, created_at, updated_at
               FROM scratch
               WHERE id = $1 AND scratch_type = $2"#,
        )
        .bind(id)
        .bind(scratch_type)
        .fetch_optional(pool)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

        Scratch::try_from(row)
    }

    pub async fn find_by_id(
        pool: &SqlitePool,
        id: Uuid,
        scratch_type: &ScratchType,
    ) -> Result<Option<Self>, ScratchError> {
        let scratch_type_str = scratch_type.to_string();
        let row = sqlx::query_as::<_, ScratchRow>(
            r#"SELECT id, scratch_type, payload, created_at, updated_at
               FROM scratch
               WHERE id = $1 AND scratch_type = $2"#,
        )
        .bind(id)
        .bind(&scratch_type_str)
        .fetch_optional(pool)
        .await?;

        let scratch = row.map(Scratch::try_from).transpose()?;
        Ok(scratch)
    }

    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Self>, ScratchError> {
        let rows = sqlx::query_as::<_, ScratchRow>(
            r#"SELECT id, scratch_type, payload, created_at, updated_at
               FROM scratch
               ORDER BY created_at DESC"#,
        )
        .fetch_all(pool)
        .await?;

        let scratches = rows
            .into_iter()
            .filter_map(|row| Scratch::try_from(row).ok())
            .collect();

        Ok(scratches)
    }

    pub async fn update(
        pool: &SqlitePool,
        id: Uuid,
        scratch_type: &ScratchType,
        data: &UpdateScratch,
    ) -> Result<Self, ScratchError> {
        let payload_str = serde_json::to_string(&data.payload)?;
        let scratch_type_str = scratch_type.to_string();

        let existing = Self::find_by_id(pool, id, scratch_type).await?;
        
        if existing.is_some() {
            sqlx::query(
                r#"UPDATE scratch SET payload = $1, updated_at = datetime('now', 'subsec')
                   WHERE id = $2 AND scratch_type = $3"#,
            )
            .bind(&payload_str)
            .bind(id)
            .bind(&scratch_type_str)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                r#"INSERT INTO scratch (id, scratch_type, payload, created_at, updated_at)
                   VALUES ($1, $2, $3, datetime('now', 'subsec'), datetime('now', 'subsec'))"#,
            )
            .bind(id)
            .bind(&scratch_type_str)
            .bind(&payload_str)
            .execute(pool)
            .await?;
        }

        Self::find_by_id_internal(pool, id, &scratch_type_str).await
    }

    pub async fn delete(
        pool: &SqlitePool,
        id: Uuid,
        scratch_type: &ScratchType,
    ) -> Result<u64, sqlx::Error> {
        let scratch_type_str = scratch_type.to_string();
        let result = sqlx::query("DELETE FROM scratch WHERE id = $1 AND scratch_type = $2")
            .bind(id)
            .bind(&scratch_type_str)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_rowid(
        pool: &SqlitePool,
        rowid: i64,
    ) -> Result<Option<Self>, ScratchError> {
        let row = sqlx::query_as::<_, ScratchRow>(
            r#"SELECT id, scratch_type, payload, created_at, updated_at
               FROM scratch
               WHERE rowid = $1"#,
        )
        .bind(rowid)
        .fetch_optional(pool)
        .await?;

        let scratch = row.map(Scratch::try_from).transpose()?;
        Ok(scratch)
    }
}
