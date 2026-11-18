<!-- Vendored from payment (.claude/skills/php-writing-tests/references/exception-raising-behavior.md) on 2026-11-18. Adapted to this project: renamed from exception-raising-behavior.md, the PHP exception became a thiserror type answered through Result, and the correct shape is worked from tests/application/handlers/calculating_congestion_tax_handler.rs. -->

# Error raising behavior shape

Use this shape when testing a domain error, driving the call path that answers it rather than constructing the error
directly.

```rust
// Testing a domain error through the raising behavior. The error is exercised through the call
// path that produces it, never by constructing it directly and asserting on its message or its
// fields. A dedicated test file for an error is never justified.

// Prohibited: testing the error as a value object, no production code is exercised
#[test]
fn new_when_value_given_then_names_the_vehicle_type() {
    // Given an unsupported vehicle type
    let raw = "TRUCK";

    // When the error is constructed
    let failure = UnsupportedVehicleType::new(raw);

    // Then it names the vehicle type
    assert_eq!(failure.to_string(), "Vehicle type <TRUCK> is not supported.");
}

// Correct: driving the call path that answers the error
#[tokio::test]
async fn calculate_when_vehicle_type_is_unknown_then_fails_without_searching_the_catalog() {
    // Given a catalog holding the Gothenburg tax rules
    let catalog = TaxRulesCatalogSpy::holding(TaxRulesBuilder::gothenburg().build());

    // When the tax is calculated for a vehicle type the domain does not know
    let tax = CalculatingCongestionTaxHandler::new(&catalog)
        .calculate(command("gothenburg", "TRUCK", &["2013-02-05 07:00:00"]))
        .await;

    // Then the calculation fails because the vehicle type is not supported
    assert!(matches!(tax, Err(CalculationFailure::UnsupportedVehicleType(_))));

    // And the catalog is never searched
    assert_eq!(catalog.searched_cities(), Vec::<City>::new());
}
```
