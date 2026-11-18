<!-- Vendored from payment (.claude/skills/php-writing-tests/references/data-provider.md) on 2026-11-18. Adapted to this project: renamed from data-provider.md, the PHPUnit DataProvider with keyed datasets became an rstest matrix with named #[case::...] rows, worked from tests/driver/http/calculate_congestion_tax_endpoint.rs. -->

# rstest case matrix shape

Use this shape when a single behavior runs over a table of cases, with named rows and parameter-named arguments.

```rust
// rstest matrix shape. Two mandatory naming levels:
//  - case name: #[case::<snake_case_description>], a description of the row, never an unnamed
//    #[case(...)] that rstest numbers.
//  - arguments: one #[case] parameter per column, named after what it carries (failure,
//    expected_status, expected_code), never a tuple.
// The // Given states the input shape in prose, with the case arguments bound by rstest.
// #[rstest] comes first, then the cases, then #[tokio::test] for an async test.

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
        calculating_congestion_tax,
        r#"{"city": "gothenburg", "vehicle": "CAR", "passages": ["2013-02-07 06:23:27"]}"#,
    )
    .await;

    // Then the response carries the status of the failure
    assert_eq!(status, expected_status);

    // And the body carries its code and message
    assert_eq!(body, json!({"code": expected_code, "message": message}));
}
```
