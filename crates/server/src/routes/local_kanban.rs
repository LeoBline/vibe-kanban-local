use axum::{
    Router,
    extract::{Path, Query, State, Json},
    response::Json as ResponseJson,
    routing::{delete, get, post, put},
};
use db::models::issue::{CreateIssue, Issue, UpdateIssue};
use db::models::kanban_project::{CreateKanbanProject, KanbanProject, UpdateKanbanProject};
use db::models::kanban_tag::{CreateKanbanTag, KanbanTag, UpdateKanbanTag};
use db::models::organization::Organization;
use db::models::project_status::{CreateProjectStatus, ProjectStatus, UpdateProjectStatus};
use deployment::Deployment;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utils::response::ApiResponse;

use crate::{DeploymentImpl, error::ApiError};

#[derive(Debug, Deserialize, TS)]
pub struct ListProjectsQuery {
    pub organization_id: String,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateOrganizationRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, TS)]
pub struct UpdateOrganizationRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, TS)]
pub struct GetIssuesQuery {
    pub project_id: String,
}

#[derive(Debug, Deserialize, TS)]
pub struct GetStatusesQuery {
    pub project_id: String,
}

#[derive(Debug, Deserialize, TS)]
pub struct GetTagsQuery {
    pub project_id: String,
}

#[derive(Debug, Deserialize, Serialize, TS)]
pub struct BulkUpdateIssueItem {
    pub id: String,
    pub changes: UpdateIssue,
}

#[derive(Debug, Deserialize, Serialize, TS)]
pub struct BulkUpdateIssuesRequest {
    pub updates: Vec<BulkUpdateIssueItem>,
}

pub fn router() -> Router<DeploymentImpl> {
    Router::new()
        .route("/local/organizations", get(list_organizations))
        .route("/local/organizations", post(create_organization))
        .route("/local/organizations/{id}", get(get_organization))
        .route("/local/organizations/{id}", put(update_organization))
        .route("/local/organizations/{id}", delete(delete_organization))
        .route("/local/projects", get(list_projects))
        .route("/local/projects", post(create_project))
        .route("/local/projects/{id}", get(get_project))
        .route("/local/projects/{id}", put(update_project))
        .route("/local/projects/{id}", delete(delete_project))
        .route("/local/statuses", get(list_statuses))
        .route("/local/statuses", post(create_status))
        .route("/local/statuses/{id}", get(get_status))
        .route("/local/statuses/{id}", put(update_status))
        .route("/local/statuses/{id}", delete(delete_status))
        .route("/local/statuses/default/{project_id}", post(create_default_statuses))
        .route("/local/tags", get(list_tags))
        .route("/local/tags", post(create_tag))
        .route("/local/tags/{id}", get(get_tag))
        .route("/local/tags/{id}", put(update_tag))
        .route("/local/tags/{id}", delete(delete_tag))
        .route("/local/issues", get(list_issues))
        .route("/local/issues", post(create_issue))
        .route("/local/issues/{id}", get(get_issue))
        .route("/local/issues/{id}", put(update_issue))
        .route("/local/issues/{id}", delete(delete_issue))
        .route("/local/issues/bulk", post(bulk_update_issues))
}

async fn list_organizations(
    State(deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<Vec<Organization>>>, ApiError> {
    let organizations = Organization::find_all(&deployment.db().pool).await?;
    Ok(ResponseJson(ApiResponse::success(organizations)))
}

async fn create_organization(
    State(deployment): State<DeploymentImpl>,
    Json(payload): Json<CreateOrganizationRequest>,
) -> Result<ResponseJson<ApiResponse<Organization>>, ApiError> {
    let organization = Organization::create(&deployment.db().pool, &db::models::organization::CreateOrganization { name: payload.name }).await?;
    Ok(ResponseJson(ApiResponse::success(organization)))
}

async fn get_organization(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<String>,
) -> Result<ResponseJson<ApiResponse<Organization>>, ApiError> {
    let organization = Organization::find_by_id(&deployment.db().pool, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(ResponseJson(ApiResponse::success(organization)))
}

async fn update_organization(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateOrganizationRequest>,
) -> Result<ResponseJson<ApiResponse<Organization>>, ApiError> {
    let organization = Organization::update(&deployment.db().pool, &id, &payload.name).await?;
    Ok(ResponseJson(ApiResponse::success(organization)))
}

async fn delete_organization(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<String>,
) -> Result<ResponseJson<ApiResponse<()>>, ApiError> {
    let _ = Organization::delete(&deployment.db().pool, &id).await?;
    Ok(ResponseJson(ApiResponse::success(())))
}

async fn list_projects(
    State(deployment): State<DeploymentImpl>,
    Query(query): Query<ListProjectsQuery>,
) -> Result<ResponseJson<ApiResponse<Vec<KanbanProject>>>, ApiError> {
    let projects = KanbanProject::find_by_organization(&deployment.db().pool, &query.organization_id).await?;
    Ok(ResponseJson(ApiResponse::success(projects)))
}

async fn create_project(
    State(deployment): State<DeploymentImpl>,
    Json(payload): Json<CreateKanbanProject>,
) -> Result<ResponseJson<ApiResponse<KanbanProject>>, ApiError> {
    let project = KanbanProject::create(&deployment.db().pool, &payload).await?;
    Ok(ResponseJson(ApiResponse::success(project)))
}

async fn get_project(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<String>,
) -> Result<ResponseJson<ApiResponse<KanbanProject>>, ApiError> {
    let project = KanbanProject::find_by_id(&deployment.db().pool, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(ResponseJson(ApiResponse::success(project)))
}

async fn update_project(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateKanbanProject>,
) -> Result<ResponseJson<ApiResponse<KanbanProject>>, ApiError> {
    let project = KanbanProject::update(&deployment.db().pool, &id, &payload).await?;
    Ok(ResponseJson(ApiResponse::success(project)))
}

async fn delete_project(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<String>,
) -> Result<ResponseJson<ApiResponse<()>>, ApiError> {
    let _ = KanbanProject::delete(&deployment.db().pool, &id).await?;
    Ok(ResponseJson(ApiResponse::success(())))
}

async fn list_statuses(
    State(deployment): State<DeploymentImpl>,
    Query(query): Query<GetStatusesQuery>,
) -> Result<ResponseJson<ApiResponse<Vec<ProjectStatus>>>, ApiError> {
    let statuses = ProjectStatus::find_by_project(&deployment.db().pool, &query.project_id).await?;
    Ok(ResponseJson(ApiResponse::success(statuses)))
}

async fn create_status(
    State(deployment): State<DeploymentImpl>,
    Json(payload): Json<CreateProjectStatus>,
) -> Result<ResponseJson<ApiResponse<ProjectStatus>>, ApiError> {
    let status = ProjectStatus::create(&deployment.db().pool, &payload).await?;
    Ok(ResponseJson(ApiResponse::success(status)))
}

async fn get_status(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<String>,
) -> Result<ResponseJson<ApiResponse<ProjectStatus>>, ApiError> {
    let status = ProjectStatus::find_by_id(&deployment.db().pool, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(ResponseJson(ApiResponse::success(status)))
}

async fn update_status(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateProjectStatus>,
) -> Result<ResponseJson<ApiResponse<ProjectStatus>>, ApiError> {
    let status = ProjectStatus::update(&deployment.db().pool, &id, &payload).await?;
    Ok(ResponseJson(ApiResponse::success(status)))
}

async fn delete_status(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<String>,
) -> Result<ResponseJson<ApiResponse<()>>, ApiError> {
    let _ = ProjectStatus::delete(&deployment.db().pool, &id).await?;
    Ok(ResponseJson(ApiResponse::success(())))
}

async fn create_default_statuses(
    State(deployment): State<DeploymentImpl>,
    Path(project_id): Path<String>,
) -> Result<ResponseJson<ApiResponse<Vec<ProjectStatus>>>, ApiError> {
    let statuses = ProjectStatus::create_default_statuses(&deployment.db().pool, &project_id).await?;
    Ok(ResponseJson(ApiResponse::success(statuses)))
}

async fn list_tags(
    State(deployment): State<DeploymentImpl>,
    Query(query): Query<GetTagsQuery>,
) -> Result<ResponseJson<ApiResponse<Vec<KanbanTag>>>, ApiError> {
    let tags = KanbanTag::find_by_project(&deployment.db().pool, &query.project_id).await?;
    Ok(ResponseJson(ApiResponse::success(tags)))
}

async fn create_tag(
    State(deployment): State<DeploymentImpl>,
    Json(payload): Json<CreateKanbanTag>,
) -> Result<ResponseJson<ApiResponse<KanbanTag>>, ApiError> {
    let tag = KanbanTag::create(&deployment.db().pool, &payload).await?;
    Ok(ResponseJson(ApiResponse::success(tag)))
}

async fn get_tag(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<String>,
) -> Result<ResponseJson<ApiResponse<KanbanTag>>, ApiError> {
    let tag = KanbanTag::find_by_id(&deployment.db().pool, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(ResponseJson(ApiResponse::success(tag)))
}

async fn update_tag(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateKanbanTag>,
) -> Result<ResponseJson<ApiResponse<KanbanTag>>, ApiError> {
    let tag = KanbanTag::update(&deployment.db().pool, &id, &payload).await?;
    Ok(ResponseJson(ApiResponse::success(tag)))
}

async fn delete_tag(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<String>,
) -> Result<ResponseJson<ApiResponse<()>>, ApiError> {
    let _ = KanbanTag::delete(&deployment.db().pool, &id).await?;
    Ok(ResponseJson(ApiResponse::success(())))
}

async fn list_issues(
    State(deployment): State<DeploymentImpl>,
    Query(query): Query<GetIssuesQuery>,
) -> Result<ResponseJson<ApiResponse<Vec<Issue>>>, ApiError> {
    let issues = Issue::find_by_project(&deployment.db().pool, &query.project_id).await?;
    Ok(ResponseJson(ApiResponse::success(issues)))
}

async fn create_issue(
    State(deployment): State<DeploymentImpl>,
    Json(payload): Json<CreateIssue>,
) -> Result<ResponseJson<ApiResponse<Issue>>, ApiError> {
    let issue = Issue::create(&deployment.db().pool, &payload).await?;
    Ok(ResponseJson(ApiResponse::success(issue)))
}

async fn get_issue(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<String>,
) -> Result<ResponseJson<ApiResponse<Issue>>, ApiError> {
    let issue = Issue::find_by_id(&deployment.db().pool, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(ResponseJson(ApiResponse::success(issue)))
}

async fn update_issue(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateIssue>,
) -> Result<ResponseJson<ApiResponse<Issue>>, ApiError> {
    let issue = Issue::update(&deployment.db().pool, &id, &payload).await?;
    Ok(ResponseJson(ApiResponse::success(issue)))
}

async fn delete_issue(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<String>,
) -> Result<ResponseJson<ApiResponse<()>>, ApiError> {
    let _ = Issue::delete(&deployment.db().pool, &id).await?;
    Ok(ResponseJson(ApiResponse::success(())))
}

async fn bulk_update_issues(
    State(deployment): State<DeploymentImpl>,
    body: String,
) -> Result<ResponseJson<ApiResponse<Vec<Issue>>>, ApiError> {
    eprintln!("[DEBUG] bulk_update_issues called with raw body: {}", body);
    
    let payload: BulkUpdateIssuesRequest = serde_json::from_str(&body).map_err(|e| {
        eprintln!("[ERROR] Failed to parse bulk update request: {}", e);
        ApiError::BadRequest(format!("Failed to parse request: {}", e))
    })?;
    
    eprintln!("[DEBUG] Successfully parsed payload");
    let mut updated_issues = Vec::new();
    for item in &payload.updates {
        eprintln!("[DEBUG] Updating issue {}", item.id);
        match Issue::update(&deployment.db().pool, &item.id, &item.changes).await {
            Ok(issue) => updated_issues.push(issue),
            Err(e) => {
                eprintln!("[ERROR] Failed to update issue {}: {:?}", item.id, e);
                return Err(ApiError::BadRequest(format!("Failed to update issue {}: {:?}", item.id, e)));
            }
        }
    }
    Ok(ResponseJson(ApiResponse::success(updated_issues)))
}
