use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use super::calculate_congestion_tax_request::CalculateCongestionTaxRequest;
use super::congestion_tax_response::CongestionTaxResponse;
use super::error_response;
use crate::application::ports::inbound::CalculatingCongestionTax;

pub(crate) async fn calculate_congestion_tax<P: CalculatingCongestionTax>(
    State(calculating_congestion_tax): State<Arc<P>>,
    payload: Result<Json<CalculateCongestionTaxRequest>, JsonRejection>,
) -> Response {
    let request = match payload {
        Ok(Json(request)) => request,
        Err(rejection) => return error_response::invalid_request(&rejection),
    };

    match calculating_congestion_tax.calculate(request.into()).await {
        Ok(tax) => (StatusCode::OK, Json(CongestionTaxResponse::from(tax))).into_response(),
        Err(failure) => error_response::from_failure(&failure),
    }
}
