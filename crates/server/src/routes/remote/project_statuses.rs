use api_types::ListProjectStatusesResponse;
use axum::{
    Router,
    extract::{Query, State},
    response::Json as ResponseJson,
    routing::get,
};
use serde::Deserialize;
use utils::response::ApiResponse;
use uuid::Uuid;

use crate::{DeploymentImpl, error::ApiError};

#[derive(Debug, Deserialize)]
pub struct ListProjectStatusesQuery {
    pub project_id: Uuid,
}

pub fn router() -> Router<DeploymentImpl> {
    Router::new().route("/project-statuses", get(list_project_statuses))
}

async fn list_project_statuses(
    State(deployment): State<DeploymentImpl>,
    Query(query): Query<ListProjectStatusesQuery>,
) -> Result<ResponseJson<ApiResponse<ListProjectStatusesResponse>>, ApiError> {
    let client = match deployment.remote_client() {
        Ok(client) => client,
        Err(_) => {
            return Err(ApiError::BadRequest(
                "Remote client not configured. GitHub integration is required for this operation."
                    .to_string(),
            ));
        }
    };
    let response = client.list_project_statuses(query.project_id).await?;
    Ok(ResponseJson(ApiResponse::success(response)))
}
