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

use crate::support::fixtures::database_fixtures::{migrated_database, unreachable_database};
use crate::support::spies::calculating_congestion_tax_spy::CalculatingCongestionTaxSpy;
use crate::support::spies::dependency_probe_spy::DependencyProbeSpy;

async fn get(path: &str, finding: PostgresTaxRulesFinding) -> (StatusCode, Value) {
    let response = router(
        Arc::new(DependencyProbeSpy::up(Duration::ZERO)),
        Arc::new(finding),
        Arc::new(CalculatingCongestionTaxSpy::calculating(CongestionTax {
            days: Vec::new(),
            amount: 0,
            currency: "SEK".to_owned(),
        })),
    )
    .oneshot(
        Request::get(path)
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
async fn find_tax_rules_by_city_when_city_keeps_tax_rules_then_responds_with_them() {
    // Given the migrated database, seeded with the tax rules of Gothenburg
    let finding = PostgresTaxRulesFinding::new(migrated_database());

    // When the tax rules of Gothenburg are asked
    let (status, body) = get("/tax-rules/gothenburg", finding).await;

    // Then the response is OK
    assert_eq!(status, StatusCode::OK);

    // And the body carries every tax period of the city with its tariff, calendar, and exempt vehicles
    assert_eq!(
        body,
        json!({
            "city": "gothenburg",
            "periods": [{
                "starts_on": "2013-01-01",
                "ends_on": "2013-12-31",
                "tariff_bands": [
                    {"amount": 8, "starts_at": "06:00:00", "ends_at": "06:30:00"},
                    {"amount": 13, "starts_at": "06:30:00", "ends_at": "07:00:00"},
                    {"amount": 18, "starts_at": "07:00:00", "ends_at": "08:00:00"},
                    {"amount": 13, "starts_at": "08:00:00", "ends_at": "08:30:00"},
                    {"amount": 8, "starts_at": "08:30:00", "ends_at": "15:00:00"},
                    {"amount": 13, "starts_at": "15:00:00", "ends_at": "15:30:00"},
                    {"amount": 18, "starts_at": "15:30:00", "ends_at": "17:00:00"},
                    {"amount": 13, "starts_at": "17:00:00", "ends_at": "18:00:00"},
                    {"amount": 8, "starts_at": "18:00:00", "ends_at": "18:30:00"}
                ],
                "exempt_vehicles": ["BUS", "DIPLOMAT", "EMERGENCY", "FOREIGN", "MILITARY", "MOTORCYCLE"],
                "public_holidays": [
                    "2013-01-01", "2013-01-06", "2013-03-29", "2013-03-31", "2013-04-01", "2013-05-01", "2013-05-09",
                    "2013-05-19", "2013-06-06", "2013-06-22", "2013-11-02", "2013-12-25", "2013-12-26", "2014-01-01"
                ],
                "toll_free_months": [7],
                "toll_free_weekdays": [6, 7],
                "daily_maximum_amount": 60,
                "single_charge_window_minutes": 60,
                "toll_free_day_before_holiday": true
            }],
            "currency": "SEK"
        })
    );
}

#[tokio::test]
async fn find_tax_rules_by_city_when_city_keeps_no_tax_rules_then_responds_not_found() {
    // Given the migrated database, which keeps no tax rules for Stockholm
    let finding = PostgresTaxRulesFinding::new(migrated_database());

    // When the tax rules of Stockholm are asked
    let (status, body) = get("/tax-rules/stockholm", finding).await;

    // Then the response is not found
    assert_eq!(status, StatusCode::NOT_FOUND);

    // And the body names the missing tax rules
    assert_eq!(
        body,
        json!({"code": "TAX_RULES_NOT_FOUND", "message": "No tax rules found for the city."})
    );
}

#[tokio::test]
async fn find_tax_rules_by_city_when_tax_rules_cannot_be_read_then_responds_that_the_service_is_not_ready() {
    // Given a port no database listens on
    let finding = PostgresTaxRulesFinding::new(unreachable_database());

    // When the tax rules of Gothenburg are asked
    let (status, body) = get("/tax-rules/gothenburg", finding).await;

    // Then the response is service unavailable
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);

    // And the body hides the cause behind the standard not ready envelope
    assert_eq!(
        body,
        json!({"code": "SERVICE_UNAVAILABLE", "message": "The service is not ready."})
    );
}
