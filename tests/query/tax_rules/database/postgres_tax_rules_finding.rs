use congestion_tax::query::tax_rules::{PostgresTaxRulesFinding, TaxRulesFinding, TaxRulesFindingUnavailable};

use crate::support::fixtures::database_fixtures::{migrated_database, unreachable_database};

#[tokio::test]
async fn find_when_city_keeps_tax_rules_then_answers_them() {
    // Given the migrated database, seeded with the tax rules of Gothenburg
    let finding = PostgresTaxRulesFinding::new(migrated_database());

    // When the tax rules of Gothenburg are found
    let rules = finding.find("gothenburg").await;

    // Then the rules of Gothenburg are answered in Swedish kronor
    let rules = rules
        .expect("the tax rules are readable")
        .expect("Gothenburg keeps tax rules");
    assert_eq!(rules.city, "gothenburg");
    assert_eq!(rules.currency, "SEK");

    // And they hold the one tax period of 2013
    assert_eq!(rules.periods.len(), 1);
    let period = &rules.periods[0];
    assert_eq!(period.starts_on, "2013-01-01");
    assert_eq!(period.ends_on, "2013-12-31");

    // And the period carries its nine tariff bands in the order of the day
    assert_eq!(period.tariff_bands.len(), 9);
    assert_eq!(period.tariff_bands[0].amount, 8);
    assert_eq!(period.tariff_bands[0].starts_at, "06:00:00");
    assert_eq!(period.tariff_bands[0].ends_at, "06:30:00");
    assert_eq!(period.tariff_bands[8].starts_at, "18:00:00");

    // And the period carries its toll free calendar, exempt vehicles, and limits
    assert_eq!(period.public_holidays.len(), 14);
    assert_eq!(period.public_holidays[0], "2013-01-01");
    assert_eq!(
        period.exempt_vehicles,
        ["BUS", "DIPLOMAT", "EMERGENCY", "FOREIGN", "MILITARY", "MOTORCYCLE"]
    );
    assert_eq!(period.toll_free_months, [7]);
    assert_eq!(period.toll_free_weekdays, [6, 7]);
    assert!(period.toll_free_day_before_holiday);
    assert_eq!(period.daily_maximum_amount, 60);
    assert_eq!(period.single_charge_window_minutes, 60);
}

#[tokio::test]
async fn find_when_city_is_named_in_any_case_then_answers_its_rules() {
    // Given the migrated database, seeded with the tax rules of Gothenburg
    let finding = PostgresTaxRulesFinding::new(migrated_database());

    // When the tax rules are found for the city written with capitals and surrounding spaces
    let rules = finding.find("  Gothenburg ").await;

    // Then the rules of Gothenburg are answered
    let rules = rules
        .expect("the tax rules are readable")
        .expect("Gothenburg keeps tax rules");
    assert_eq!(rules.city, "gothenburg");
}

#[tokio::test]
async fn find_when_city_keeps_no_tax_rules_then_answers_none() {
    // Given the migrated database, which keeps no tax rules for Stockholm
    let finding = PostgresTaxRulesFinding::new(migrated_database());

    // When the tax rules of Stockholm are found
    let rules = finding.find("stockholm").await;

    // Then no tax rules are answered
    assert!(matches!(rules, Ok(None)));
}

#[tokio::test]
async fn find_when_database_is_unreachable_then_fails_as_unavailable() {
    // Given a port no database listens on
    let finding = PostgresTaxRulesFinding::new(unreachable_database());

    // When the tax rules of Gothenburg are found
    let rules = finding.find("gothenburg").await;

    // Then the finding fails because the tax rules cannot be read
    assert!(matches!(rules, Err(TaxRulesFindingUnavailable { .. })));
}
