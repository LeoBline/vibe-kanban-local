use api_types::{CreateIssueTagRequest, IssueTag, ListIssueTagsResponse, MutationResponse};
use axum::{
    Router,
    extract::{Json, Path, Query, State},
    response::Json as ResponseJson,
    routing::get,
};
use serde::Deserialize;
use utils::response::ApiResponse;
use uuid::Uuid;

use crate::{DeploymentImpl, error::ApiError};

#[derive(Debug, Deserialize)]
pub struct ListIssueTagsQuery {
    pub issue_id: Uuid,
}

pub fn router() -> Router<DeploymentImpl> {
    Router::new()
        .route("/issue-tags", get(list_issue_tags).post(create_issue_tag))
        .route(
            "/issue-tags/{issue_tag_id}",
            get(get_issue_tag).delete(delete_issue_tag),
        )
}

async fn list_issue_tags(
    State(deployment): State<DeploymentImpl>,
    Query(query): Query<ListIssueTagsQuery>,
) -> Result<ResponseJson<ApiResponse<ListIssueTagsResponse>>, ApiError> {
    let client = match deployment.remote_client() {
        Ok(client) => client,
        Err(_) => {
            return Err(ApiError::BadRequest(
                "Remote client not configured. GitHub integration is required for this operation."
                    .to_string(),
            ));
        }
    };
    let response = client.list_issue_tags(query.issue_id).await?;
    Ok(ResponseJson(ApiResponse::success(response)))
}

async fn get_issue_tag(
    State(deployment): State<DeploymentImpl>,
    Path(issue_tag_id): Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<IssueTag>>, ApiError> {
    let client = match deployment.remote_client() {
        Ok(client) => client,
        Err(_) => {
            return Err(ApiError::BadRequest(
                "Remote client not configured. GitHub integration is required for this operation."
                    .to_string(),
            ));
        }
    };
    let response = client.get_issue_tag(issue_tag_id).await?;
    Ok(ResponseJson(ApiResponse::success(response)))
}

async fn create_issue_tag(
    State(deployment): State<DeploymentImpl>,
    Json(request): Json<CreateIssueTagRequest>,
) -> Result<ResponseJson<ApiResponse<MutationResponse<IssueTag>>>, ApiError> {
    let client = match deployment.remote_client() {
        Ok(client) => client,
        Err(_) => {
            return Err(ApiError::BadRequest(
                "Remote client not configured. GitHub integration is required for this operation."
                    .to_string(),
            ));
        }
    };
    let response = client.create_issue_tag(&request).await?;
    Ok(ResponseJson(ApiResponse::success(response)))
}

async fn delete_issue_tag(
    State(deployment): State<DeploymentImpl>,
    Path(issue_tag_id): Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<()>>, ApiError> {
    let client = match deployment.remote_client() {
        Ok(client) => client,
        Err(_) => {
            return Err(ApiError::BadRequest(
                "Remote client not configured. GitHub integration is required for this operation."
                    .to_string(),
            ));
        }
    };
    client.delete_issue_tag(issue_tag_id).await?;
    Ok(ResponseJson(ApiResponse::success(())))
}
