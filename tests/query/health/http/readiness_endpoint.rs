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

async fn readiness(database: DependencyProbeSpy) -> (StatusCode, Value) {
    let service = router(
        Arc::new(database),
        Arc::new(PostgresTaxRulesFinding::new(unreachable_database())),
        Arc::new(CalculatingCongestionTaxSpy::calculating(CongestionTax {
            days: Vec::new(),
            amount: 0,
            currency: "SEK".to_owned(),
        })),
    );

    let response = service
        .oneshot(
            Request::get("/health/readiness")
                .body(Body::empty())
                .expect("the request is well formed"),
        )
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
async fn readiness_when_database_answers_then_responds_healthy() {
    // Given a database that answered in 4.59 milliseconds
    let database = DependencyProbeSpy::up(Duration::from_micros(4_590));

    // When the readiness of the service is asked
    let (status, body) = readiness(database).await;

    // Then the response is OK
    assert_eq!(status, StatusCode::OK);

    // And the body reports the database up and the service healthy
    assert_eq!(
        body,
        json!({
            "checks": [{"status": "UP", "critical": true, "component": "database", "duration_in_milliseconds": 4.59}],
            "status": "HEALTHY"
        })
    );
}

#[tokio::test]
async fn readiness_when_database_is_down_then_responds_unavailable() {
    // Given a database that failed after 2001.37 milliseconds
    let database = DependencyProbeSpy::down(
        "pool timed out while waiting for an open connection",
        Duration::from_micros(2_001_370),
    );

    // When the readiness of the service is asked
    let (status, body) = readiness(database).await;

    // Then the response is service unavailable
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);

    // And the body reports the failure of the database and the service unavailable
    assert_eq!(
        body,
        json!({
            "checks": [{
                "detail": "pool timed out while waiting for an open connection",
                "status": "DOWN",
                "critical": true,
                "component": "database",
                "duration_in_milliseconds": 2001.37
            }],
            "status": "UNAVAILABLE"
        })
    );
}
