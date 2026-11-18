use axum::Json;

use super::liveness_response::LivenessResponse;

pub(crate) async fn liveness() -> Json<LivenessResponse> {
    Json(LivenessResponse::up())
}
