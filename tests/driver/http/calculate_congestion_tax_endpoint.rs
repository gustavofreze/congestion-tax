use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use congestion_tax::application::commands::CalculateCongestionTax;
use congestion_tax::application::domain::models::tax_rules::{City, Passage, Passages, VehicleType};
use congestion_tax::application::ports::inbound::{
    CalculationFailure, CongestionTax, DailyCongestionTax, TaxRulesNotFound,
};
use congestion_tax::application::ports::outbound::TaxRulesUnavailable;
use congestion_tax::query::tax_rules::PostgresTaxRulesFinding;
use congestion_tax::routes::router;
use http_body_util::BodyExt;
use rstest::rstest;
use serde_json::{Value, json};
use tower::ServiceExt;
use tracing::instrument::WithSubscriber;

use crate::support::builders::tax_rules_builder::TaxRulesBuilder;
use crate::support::fixtures::database_fixtures::unreachable_database;
use crate::support::spies::calculating_congestion_tax_spy::CalculatingCongestionTaxSpy;
use crate::support::spies::dependency_probe_spy::DependencyProbeSpy;
use crate::support::spies::log_spy::LogSpy;

fn tax_of_89_kronor() -> CongestionTax {
    CongestionTax {
        days: vec![
            DailyCongestionTax {
                date: "2013-02-07".to_owned(),
                amount: 29,
            },
            DailyCongestionTax {
                date: "2013-02-08".to_owned(),
                amount: 60,
            },
        ],
        amount: 89,
        currency: "SEK".to_owned(),
    }
}

fn passage_outside_tax_periods() -> CalculationFailure {
    let passages: Passages = ["2014-02-04 07:00:00"
        .parse::<Passage>()
        .expect("the passage is a local date and time")]
    .into_iter()
    .collect();

    TaxRulesBuilder::gothenburg()
        .build()
        .tax_for(VehicleType::Car, &passages)
        .expect_err("no 2013 period covers 2014")
        .into()
}

async fn post(body: &str, calculating_congestion_tax: Arc<CalculatingCongestionTaxSpy>) -> (StatusCode, Value) {
    let request = Request::post("/congestion-taxes")
        .header("content-type", "application/json")
        .body(Body::from(body.to_owned()))
        .expect("the request is well formed");

    let response = router(
        Arc::new(DependencyProbeSpy::up(Duration::ZERO)),
        Arc::new(PostgresTaxRulesFinding::new(unreachable_database())),
        calculating_congestion_tax,
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
async fn calculate_when_request_is_valid_then_responds_with_the_tax_due_per_day() {
    // Given a calculation that answers 89 kronor over two days
    let calculating_congestion_tax = Arc::new(CalculatingCongestionTaxSpy::calculating(tax_of_89_kronor()));

    // When a car with two passages in Gothenburg is posted
    let (status, body) = post(
        r#"{"city": "gothenburg", "vehicle": "CAR", "passages": ["2013-02-07 06:23:27", "2013-02-08 15:27:00"]}"#,
        Arc::clone(&calculating_congestion_tax),
    )
    .await;

    // Then the response is OK
    assert_eq!(status, StatusCode::OK);

    // And the body carries the total and the tax of each day
    assert_eq!(
        body,
        json!({
            "days": [{"date": "2013-02-07", "amount": 29}, {"date": "2013-02-08", "amount": 60}],
            "amount": 89,
            "currency": "SEK"
        })
    );

    // And the calculation received the request as a command
    assert_eq!(
        calculating_congestion_tax.received_commands(),
        vec![CalculateCongestionTax {
            city: "gothenburg".to_owned(),
            vehicle: "CAR".to_owned(),
            passages: vec!["2013-02-07 06:23:27".to_owned(), "2013-02-08 15:27:00".to_owned()],
        }]
    );
}

#[rstest]
#[case::city_is_empty(
    City::new(" ").expect_err("a blank city is empty").into(),
    StatusCode::UNPROCESSABLE_ENTITY,
    "CITY_CANNOT_BE_EMPTY"
)]
#[case::vehicle_type_is_unsupported(
    "TRUCK".parse::<VehicleType>().expect_err("TRUCK is not a vehicle type").into(),
    StatusCode::UNPROCESSABLE_ENTITY,
    "UNSUPPORTED_VEHICLE_TYPE"
)]
#[case::passage_format_is_not_valid(
    "yesterday".parse::<Passage>().expect_err("yesterday is not a date and time").into(),
    StatusCode::UNPROCESSABLE_ENTITY,
    "PASSAGE_FORMAT_NOT_VALID"
)]
#[case::passage_is_outside_tax_periods(
    passage_outside_tax_periods(),
    StatusCode::UNPROCESSABLE_ENTITY,
    "PASSAGE_OUTSIDE_TAX_PERIODS"
)]
#[case::city_has_no_tax_rules(
    TaxRulesNotFound.into(),
    StatusCode::NOT_FOUND,
    "TAX_RULES_NOT_FOUND"
)]
#[tokio::test]
async fn calculate_when_calculation_fails_then_responds_with_the_matching_error(
    #[case] failure: CalculationFailure,
    #[case] expected_status: StatusCode,
    #[case] expected_code: &str,
) {
    // Given a calculation that fails with the case failure
    let message = failure.to_string();
    let calculating_congestion_tax = Arc::new(CalculatingCongestionTaxSpy::failing(failure));

    // When a car is posted
    let (status, body) = post(
        r#"{"city": "gothenburg", "vehicle": "CAR", "passages": ["2013-02-07 06:23:27"]}"#,
        calculating_congestion_tax,
    )
    .await;

    // Then the response carries the status of the failure
    assert_eq!(status, expected_status);

    // And the body carries its code and message
    assert_eq!(body, json!({"code": expected_code, "message": message}));
}

#[tokio::test]
async fn calculate_when_tax_rules_are_unavailable_then_responds_that_the_service_is_not_ready() {
    // Given a calculation that fails because the tax rules cannot be read
    let calculating_congestion_tax = Arc::new(CalculatingCongestionTaxSpy::failing(
        TaxRulesUnavailable::new("connection refused").into(),
    ));

    // And a log listening to the service
    let log = LogSpy::listening();

    // When a car is posted
    let (status, body) = post(
        r#"{"city": "gothenburg", "vehicle": "CAR", "passages": ["2013-02-07 06:23:27"]}"#,
        calculating_congestion_tax,
    )
    .with_subscriber(log.subscriber())
    .await;

    // Then the response is service unavailable
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);

    // And the body hides the cause behind the standard not ready envelope
    assert_eq!(
        body,
        json!({"code": "SERVICE_UNAVAILABLE", "message": "The service is not ready."})
    );

    // And the log keeps the cause as an error
    assert!(log.logged_lines().iter().any(|line| {
        line.contains("ERROR")
            && line.contains("The tax rules catalog is unavailable.")
            && line.contains("connection refused")
    }));
}

#[rstest]
#[case::body_is_not_json(r#"{"city": "gothenburg", "vehicle":"#)]
#[case::city_is_missing(r#"{"vehicle": "CAR", "passages": []}"#)]
#[case::passages_are_not_a_list(r#"{"city": "gothenburg", "vehicle": "CAR", "passages": "2013-02-07 06:23:27"}"#)]
#[tokio::test]
async fn calculate_when_body_does_not_match_the_contract_then_responds_with_invalid_request(#[case] request: &str) {
    // Given a calculation that would answer 89 kronor
    let calculating_congestion_tax = Arc::new(CalculatingCongestionTaxSpy::calculating(tax_of_89_kronor()));

    // When a body that does not match the contract is posted
    let (status, body) = post(request, Arc::clone(&calculating_congestion_tax)).await;

    // Then the response is unprocessable
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // And the body names the invalid request
    assert_eq!(body["code"], "INVALID_REQUEST");

    // And the calculation is never reached
    assert_eq!(
        calculating_congestion_tax.received_commands(),
        Vec::<CalculateCongestionTax>::new()
    );
}
