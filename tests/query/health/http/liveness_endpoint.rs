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

#[tokio::test]
async fn liveness_when_process_is_running_then_responds_up() {
    // Given a running service whose database is down
    let service = router(
        Arc::new(DependencyProbeSpy::down("connection refused", Duration::ZERO)),
        Arc::new(PostgresTaxRulesFinding::new(unreachable_database())),
        Arc::new(CalculatingCongestionTaxSpy::calculating(CongestionTax {
            days: Vec::new(),
            amount: 0,
            currency: "SEK".to_owned(),
        })),
    );

    // When its liveness is asked
    let response = service
        .oneshot(
            Request::get("/health/liveness")
                .body(Body::empty())
                .expect("the request is well formed"),
        )
        .await
        .expect("the router always answers");

    // Then the response is OK
    assert_eq!(response.status(), StatusCode::OK);

    // And the body reports the process up
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("the body is readable")
        .to_bytes();
    assert_eq!(
        serde_json::from_slice::<Value>(&bytes).expect("the body is JSON"),
        json!({"status": "UP"})
    );
}
