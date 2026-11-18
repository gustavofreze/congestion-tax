use std::sync::Arc;

use axum::extract::State;
use axum::response::{IntoResponse, Response};

use super::dependency_status_response::DependencyStatusResponse;
use super::readiness_response::ReadinessResponse;
use crate::query::health::DependencyProbe;

pub(crate) async fn readiness<D: DependencyProbe>(State(database): State<Arc<D>>) -> Response {
    ReadinessResponse::from(DependencyStatusResponse::from(database.probe().await)).into_response()
}
