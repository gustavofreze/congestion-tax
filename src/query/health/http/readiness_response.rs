use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use super::dependency_status_response::DependencyStatusResponse;

#[derive(Serialize)]
pub(super) struct ReadinessResponse {
    checks: Vec<DependencyStatusResponse>,
    status: &'static str,
    #[serde(skip)]
    http_status: StatusCode,
}

impl From<DependencyStatusResponse> for ReadinessResponse {
    fn from(dependency: DependencyStatusResponse) -> Self {
        if dependency.is_down() {
            return Self {
                checks: vec![dependency],
                status: "UNAVAILABLE",
                http_status: StatusCode::SERVICE_UNAVAILABLE,
            };
        }

        Self {
            checks: vec![dependency],
            status: "HEALTHY",
            http_status: StatusCode::OK,
        }
    }
}

impl IntoResponse for ReadinessResponse {
    fn into_response(self) -> Response {
        (self.http_status, Json(self)).into_response()
    }
}
