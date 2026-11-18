use congestion_tax::application::commands::CalculateCongestionTax;
use congestion_tax::application::domain::models::tax_rules::City;
use congestion_tax::application::handlers::CalculatingCongestionTaxHandler;
use congestion_tax::application::ports::inbound::{
    CalculatingCongestionTax, CalculationFailure, CongestionTax, DailyCongestionTax,
};

use crate::support::builders::tax_rules_builder::TaxRulesBuilder;
use crate::support::spies::tax_rules_catalog_spy::TaxRulesCatalogSpy;

fn command(city: &str, vehicle: &str, passages: &[&str]) -> CalculateCongestionTax {
    CalculateCongestionTax {
        city: city.to_owned(),
        vehicle: vehicle.to_owned(),
        passages: passages.iter().map(|passage| (*passage).to_owned()).collect(),
    }
}

#[tokio::test]
async fn calculate_when_city_has_tax_rules_then_returns_the_tax_due_per_day() {
    // Given a catalog holding the Gothenburg tax rules
    let catalog = TaxRulesCatalogSpy::holding(TaxRulesBuilder::gothenburg().build());

    // When the tax of a car passing on a weekday and on a Saturday in Gothenburg is calculated
    let tax = CalculatingCongestionTaxHandler::new(&catalog)
        .calculate(command(
            "gothenburg",
            "CAR",
            &["2013-02-09 07:00:00", "2013-02-05 07:00:00"],
        ))
        .await;

    // Then the car owes 18 kronor, all of them on the weekday
    assert_eq!(
        tax.expect("the tax is calculated"),
        CongestionTax {
            days: vec![
                DailyCongestionTax {
                    date: "2013-02-05".to_owned(),
                    amount: 18,
                },
                DailyCongestionTax {
                    date: "2013-02-09".to_owned(),
                    amount: 0,
                },
            ],
            amount: 18,
            currency: "SEK".to_owned(),
        }
    );
}

#[tokio::test]
async fn calculate_when_city_is_written_in_any_case_then_searches_the_normalized_city() {
    // Given a catalog holding the Gothenburg tax rules
    let catalog = TaxRulesCatalogSpy::holding(TaxRulesBuilder::gothenburg().build());

    // When the tax is calculated for a city written with capitals and surrounding spaces
    let tax = CalculatingCongestionTaxHandler::new(&catalog)
        .calculate(command("  Gothenburg ", "CAR", &["2013-02-05 07:00:00"]))
        .await;

    // Then the catalog is searched for the city in lowercase, without the spaces
    assert_eq!(
        catalog.searched_cities(),
        vec![City::new("gothenburg").expect("gothenburg is a city")]
    );

    // And the tax is calculated for that city
    assert!(tax.is_ok());
}

#[tokio::test]
async fn calculate_when_city_has_no_tax_rules_then_fails_with_tax_rules_not_found() {
    // Given a catalog with no tax rules
    let catalog = TaxRulesCatalogSpy::empty();

    // When the tax is calculated for a city with no rules
    let tax = CalculatingCongestionTaxHandler::new(&catalog)
        .calculate(command("stockholm", "CAR", &["2013-02-05 07:00:00"]))
        .await;

    // Then the calculation fails because the city has no tax rules
    assert!(matches!(tax, Err(CalculationFailure::TaxRulesNotFound(_))));
}

#[tokio::test]
async fn calculate_when_catalog_is_unavailable_then_fails_with_tax_rules_unavailable() {
    // Given a catalog that cannot be read
    let catalog = TaxRulesCatalogSpy::unavailable();

    // When the tax is calculated
    let tax = CalculatingCongestionTaxHandler::new(&catalog)
        .calculate(command("gothenburg", "CAR", &["2013-02-05 07:00:00"]))
        .await;

    // Then the calculation fails because the tax rules are unavailable
    assert!(matches!(tax, Err(CalculationFailure::TaxRulesUnavailable(_))));
}

#[tokio::test]
async fn calculate_when_no_tax_period_covers_a_passage_then_fails_with_passage_outside_tax_periods() {
    // Given a catalog holding the Gothenburg tax rules of 2013
    let catalog = TaxRulesCatalogSpy::holding(TaxRulesBuilder::gothenburg().build());

    // When the tax of a car passing in 2014 is calculated
    let tax = CalculatingCongestionTaxHandler::new(&catalog)
        .calculate(command("gothenburg", "CAR", &["2014-02-04 07:00:00"]))
        .await;

    // Then the calculation fails because no tax period covers the passage
    assert!(matches!(tax, Err(CalculationFailure::PassageOutsideTaxPeriods(_))));
}

#[tokio::test]
async fn calculate_when_city_is_blank_then_fails_without_searching_the_catalog() {
    // Given a catalog holding the Gothenburg tax rules
    let catalog = TaxRulesCatalogSpy::holding(TaxRulesBuilder::gothenburg().build());

    // When the tax is calculated for a blank city
    let tax = CalculatingCongestionTaxHandler::new(&catalog)
        .calculate(command("   ", "CAR", &["2013-02-05 07:00:00"]))
        .await;

    // Then the calculation fails because the city cannot be empty
    assert!(matches!(tax, Err(CalculationFailure::CityCannotBeEmpty(_))));

    // And the catalog is never searched
    assert_eq!(catalog.searched_cities(), Vec::<City>::new());
}

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

#[tokio::test]
async fn calculate_when_passage_is_not_a_date_and_time_then_fails_without_searching_the_catalog() {
    // Given a catalog holding the Gothenburg tax rules
    let catalog = TaxRulesCatalogSpy::holding(TaxRulesBuilder::gothenburg().build());

    // When the tax is calculated for a passage on a day February does not have
    let tax = CalculatingCongestionTaxHandler::new(&catalog)
        .calculate(command("gothenburg", "CAR", &["2013-02-30 07:00:00"]))
        .await;

    // Then the calculation fails because the passage format is not valid
    assert!(matches!(tax, Err(CalculationFailure::PassageFormatNotValid(_))));

    // And the catalog is never searched
    assert_eq!(catalog.searched_cities(), Vec::<City>::new());
}
