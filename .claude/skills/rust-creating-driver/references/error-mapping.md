<!-- Vendored from payment (.claude/skills/php-creating-driver/references/exception-mapping.md) on 2026-11-18. Adapted to this project: renamed from exception-mapping.md, the tiny-blocks ExceptionMappingTable became an exhaustive match over the inbound port's failure enum, worked from src/driver/http/error_response.rs. -->

# Error mapping shape

Use this shape for the write error mapping at `src/driver/http/error_response.rs` (and its counterpart in a query
slice's `http/` module, when that slice has failures to map). The placement invariants (every domain and application
error has an arm, the shielded-guard carve-out, the 422 versus 409 versus 404 decision test, the error envelope) are
owned by `rust-hex-driver-http` (§ Error to HTTP mapping). This file carries the literal code shape, mirroring the live
`src/driver/http/error_response.rs`.

- A module of functions, one per input the router can fail on: `from_failure` over the inbound port's failure enum,
  `invalid_request` over the `JsonRejection` of a body that does not match the contract, and the router fallbacks
  `route_not_found` and `method_not_allowed`. All of them answer an `axum::response::Response` through the one shared
  `respond`.
- The body is `ErrorBody`, a `#[derive(Serialize)]` struct in its own `error_body.rs` with `code: &'static str` and
  `message: String`, the `{code, message}` envelope.
- `from_failure` is one exhaustive `match` with no wildcard arm. Each arm answers the code and the status, and the
  message is the error's own `Display` (`failure.to_string()`), the sentence its `#[error("...")]` declares. An error
  that carries the value it rejected interpolates it there, so the mapping never formats a message itself.
- An arm whose cause must not reach the client (an outbound port's unavailable error) logs the error and its source
  with `tracing::error!` and answers a fixed message instead.
- The status always comes from an `axum::http::StatusCode` constant, never a bare integer. The code value follows the
  `SCREAMING_SNAKE_CASE` derivation owned by `rust-hex-driver-http` (§ Error response structure).

```rust
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
```
