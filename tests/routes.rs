use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use congestion_tax::application::ports::inbound::CongestionTax;
use congestion_tax::query::tax_rules::PostgresTaxRulesFinding;
use congestion_tax::routes::router;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::support::fixtures::database_fixtures::unreachable_database;
use crate::support::spies::calculating_congestion_tax_spy::CalculatingCongestionTaxSpy;
use crate::support::spies::dependency_probe_spy::DependencyProbeSpy;

async fn send(request: Request<Body>) -> (StatusCode, Value) {
    let response = router(
        Arc::new(DependencyProbeSpy::up(Duration::ZERO)),
        Arc::new(PostgresTaxRulesFinding::new(unreachable_database())),
        Arc::new(CalculatingCongestionTaxSpy::calculating(CongestionTax {
            days: Vec::new(),
            amount: 0,
            currency: "SEK".to_owned(),
        })),
    )
    .oneshot(request)
    .await
    .expect("the router always answers");

    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("the body is readable")
        .to_bytes();

    (status, serde_json::from_slice(&bytes).expect("the body is JSON"))
}

#[tokio::test]
async fn router_when_path_is_unknown_then_responds_not_found() {
    // Given a path the router does not mount
    let request = Request::get("/congestion-tax")
        .body(Body::empty())
        .expect("the request is well formed");

    // When the request is sent
    let (status, body) = send(request).await;

    // Then the response is not found
    assert_eq!(status, StatusCode::NOT_FOUND);

    // And the body carries the not found envelope
    assert_eq!(body, json!({"code": "NOT_FOUND", "message": "Not Found."}));
}

#[tokio::test]
async fn router_when_method_is_not_allowed_then_responds_method_not_allowed() {
    // Given a mounted path under a method it does not route
    let request = Request::get("/congestion-taxes")
        .body(Body::empty())
        .expect("the request is well formed");

    // When the request is sent
    let (status, body) = send(request).await;

    // Then the response is method not allowed
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);

    // And the body carries the method not allowed envelope
    assert_eq!(
        body,
        json!({"code": "METHOD_NOT_ALLOWED", "message": "Method Not Allowed."})
    );
}
