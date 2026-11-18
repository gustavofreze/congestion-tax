use std::error::Error;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use super::city_tax_rules_response::CityTaxRulesResponse;
use super::error_body::ErrorBody;
use crate::query::tax_rules::TaxRulesFinding;

pub(crate) async fn find_tax_rules_by_city<F: TaxRulesFinding>(
    State(finding): State<Arc<F>>,
    Path(city): Path<String>,
) -> Response {
    match finding.find(&city).await {
        Ok(Some(rules)) => (StatusCode::OK, Json(CityTaxRulesResponse::from(rules))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorBody::new(
                "TAX_RULES_NOT_FOUND",
                "No tax rules found for the city.",
            )),
        )
            .into_response(),
        Err(unavailable) => {
            tracing::error!(cause = ?unavailable.source(), "{unavailable}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorBody::new("SERVICE_UNAVAILABLE", "The service is not ready.")),
            )
                .into_response()
        }
    }
}
