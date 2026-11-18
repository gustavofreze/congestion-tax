use std::error::Error;

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use super::error_body::ErrorBody;
use crate::application::ports::inbound::CalculationFailure;

pub(crate) async fn route_not_found() -> Response {
    respond("NOT_FOUND", StatusCode::NOT_FOUND, "Not Found.".to_owned())
}

pub(crate) async fn method_not_allowed() -> Response {
    respond(
        "METHOD_NOT_ALLOWED",
        StatusCode::METHOD_NOT_ALLOWED,
        "Method Not Allowed.".to_owned(),
    )
}

pub(super) fn invalid_request(rejection: &JsonRejection) -> Response {
    respond(
        "INVALID_REQUEST",
        StatusCode::UNPROCESSABLE_ENTITY,
        rejection.body_text(),
    )
}

pub(super) fn from_failure(failure: &CalculationFailure) -> Response {
    let (code, status) = match failure {
        CalculationFailure::TaxRulesNotFound(_) => ("TAX_RULES_NOT_FOUND", StatusCode::NOT_FOUND),
        CalculationFailure::CityCannotBeEmpty(_) => ("CITY_CANNOT_BE_EMPTY", StatusCode::UNPROCESSABLE_ENTITY),
        CalculationFailure::TaxRulesUnavailable(unavailable) => {
            tracing::error!(cause = ?unavailable.source(), "{unavailable}");
            return respond(
                "SERVICE_UNAVAILABLE",
                StatusCode::SERVICE_UNAVAILABLE,
                "The service is not ready.".to_owned(),
            );
        }
        CalculationFailure::PassageFormatNotValid(_) => ("PASSAGE_FORMAT_NOT_VALID", StatusCode::UNPROCESSABLE_ENTITY),
        CalculationFailure::UnsupportedVehicleType(_) => ("UNSUPPORTED_VEHICLE_TYPE", StatusCode::UNPROCESSABLE_ENTITY),
        CalculationFailure::PassageOutsideTaxPeriods(_) => {
            ("PASSAGE_OUTSIDE_TAX_PERIODS", StatusCode::UNPROCESSABLE_ENTITY)
        }
    };

    respond(code, status, failure.to_string())
}

fn respond(code: &'static str, status: StatusCode, message: String) -> Response {
    (status, Json(ErrorBody::new(code, message))).into_response()
}
