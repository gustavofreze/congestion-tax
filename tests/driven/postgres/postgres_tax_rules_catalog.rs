use std::sync::Arc;

use congestion_tax::application::domain::models::tax_rules::City;
use congestion_tax::application::ports::outbound::{TaxRulesCatalog, TaxRulesUnavailable};
use congestion_tax::driven::postgres::PostgresTaxRulesCatalog;

use crate::support::builders::tax_rules_builder::TaxRulesBuilder;
use crate::support::fixtures::database_fixtures::{
    migrated_database, store_tax_rules_with_overlapping_bands, unreachable_database,
};

#[tokio::test]
async fn find_when_city_has_tax_rules_then_returns_them() {
    // Given the catalog seeded by the migrations
    let catalog = PostgresTaxRulesCatalog::new(migrated_database());

    // When the tax rules of Gothenburg are searched
    let rules = catalog
        .find(&City::new("gothenburg").expect("gothenburg is a city"))
        .await;

    // Then the Gothenburg tax rules are returned
    assert_eq!(
        rules.expect("the catalog is available"),
        Some(Arc::new(TaxRulesBuilder::gothenburg().build()))
    );
}

#[tokio::test]
async fn find_when_city_has_no_tax_rules_then_returns_none() {
    // Given the catalog seeded by the migrations
    let catalog = PostgresTaxRulesCatalog::new(migrated_database());

    // When the tax rules of a city with no rules are searched
    let rules = catalog
        .find(&City::new("stockholm").expect("stockholm is a city"))
        .await;

    // Then no tax rules are returned
    assert_eq!(rules.expect("the catalog is available"), None);
}

#[tokio::test]
async fn find_when_stored_rules_break_an_invariant_then_fails_with_tax_rules_unavailable() {
    // Given a city whose stored tariff bands overlap
    let database = migrated_database();
    store_tax_rules_with_overlapping_bands(&database).await;

    // When the tax rules of that city are searched
    let rules = PostgresTaxRulesCatalog::new(database)
        .find(&City::new("overlapping-bands").expect("overlapping-bands is a city"))
        .await;

    // Then the search fails because the tax rules are unavailable
    assert!(matches!(rules, Err(TaxRulesUnavailable { .. })));
}

#[tokio::test]
async fn find_when_database_is_unreachable_then_fails_with_tax_rules_unavailable() {
    // Given a catalog pointing at a port no database listens on
    let catalog = PostgresTaxRulesCatalog::new(unreachable_database());

    // When the tax rules of Gothenburg are searched
    let rules = catalog
        .find(&City::new("gothenburg").expect("gothenburg is a city"))
        .await;

    // Then the search fails because the tax rules are unavailable
    assert!(matches!(rules, Err(TaxRulesUnavailable { .. })));
}
