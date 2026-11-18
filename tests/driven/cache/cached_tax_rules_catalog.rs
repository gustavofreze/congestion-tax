use std::sync::Arc;
use std::time::Duration;

use congestion_tax::application::domain::models::tax_rules::City;
use congestion_tax::application::ports::outbound::{TaxRulesCatalog, TaxRulesUnavailable};
use congestion_tax::driven::cache::{CachedTaxRulesCatalog, SystemMonotonicClock};
use rstest::rstest;

use crate::support::builders::tax_rules_builder::TaxRulesBuilder;
use crate::support::spies::monotonic_clock_spy::MonotonicClockSpy;
use crate::support::spies::tax_rules_catalog_spy::TaxRulesCatalogSpy;

const AN_HOUR: Duration = Duration::from_secs(3600);
const THIRTY_SECONDS: Duration = Duration::from_secs(30);

fn gothenburg() -> City {
    City::new("gothenburg").expect("gothenburg is a city")
}

#[tokio::test]
async fn find_when_rules_were_found_within_the_time_to_live_then_answers_without_searching_the_origin_again() {
    // Given an origin holding the Gothenburg tax rules
    let origin = TaxRulesCatalogSpy::holding(TaxRulesBuilder::gothenburg().build());

    // And a cache keeping found rules for an hour
    let catalog = CachedTaxRulesCatalog::new(SystemMonotonicClock, &origin, AN_HOUR);

    // When the rules of Gothenburg are searched twice
    let first = catalog.find(&gothenburg()).await;
    let second = catalog.find(&gothenburg()).await;

    // Then both searches answer the Gothenburg tax rules
    let expected = Some(Arc::new(TaxRulesBuilder::gothenburg().build()));
    assert_eq!(first.expect("the origin is available"), expected);
    assert_eq!(second.expect("the cache answers"), expected);

    // And the origin is searched once
    assert_eq!(origin.searched_cities(), vec![gothenburg()]);
}

#[rstest]
#[case::one_millisecond_before_the_time_to_live(Duration::from_millis(29_999), 1)]
#[case::exactly_the_time_to_live(THIRTY_SECONDS, 2)]
#[case::one_millisecond_after_the_time_to_live(Duration::from_millis(30_001), 2)]
#[tokio::test]
async fn find_when_time_passes_around_the_time_to_live_then_searches_the_origin_again_once_it_has_elapsed(
    #[case] elapsed: Duration,
    #[case] searches: usize,
) {
    // Given an origin holding the Gothenburg tax rules
    let origin = TaxRulesCatalogSpy::holding(TaxRulesBuilder::gothenburg().build());

    // And a clock the test moves forward
    let clock = MonotonicClockSpy::starting_now();

    // And a cache keeping found rules for thirty seconds
    let catalog = CachedTaxRulesCatalog::new(&clock, &origin, THIRTY_SECONDS);

    // And the rules of Gothenburg were searched once
    catalog.find(&gothenburg()).await.expect("the origin answers");

    // And time passed around the time to live
    clock.advance(elapsed);

    // When the rules of Gothenburg are searched again
    let rules = catalog.find(&gothenburg()).await;

    // Then the origin is searched again only once the time to live has fully elapsed
    assert_eq!(origin.searched_cities().len(), searches);

    // And the rules of Gothenburg are found
    assert!(matches!(rules, Ok(Some(_))));
}

#[tokio::test]
async fn find_when_origin_has_no_rules_then_keeps_nothing_and_searches_the_origin_again() {
    // Given an origin with no tax rules
    let origin = TaxRulesCatalogSpy::empty();

    // And a cache keeping found rules for an hour
    let catalog = CachedTaxRulesCatalog::new(SystemMonotonicClock, &origin, AN_HOUR);

    // When the rules of a city with no rules are searched twice
    let first = catalog.find(&gothenburg()).await;
    let second = catalog.find(&gothenburg()).await;

    // Then both searches answer no rules
    assert_eq!(first.expect("the origin is available"), None);
    assert_eq!(second.expect("the origin is available"), None);

    // And the origin is searched for each of them
    assert_eq!(origin.searched_cities(), vec![gothenburg(), gothenburg()]);
}

#[tokio::test]
async fn find_when_origin_is_unavailable_then_fails_with_tax_rules_unavailable() {
    // Given an origin that cannot be read
    let origin = TaxRulesCatalogSpy::unavailable();

    // And a cache keeping found rules for an hour
    let catalog = CachedTaxRulesCatalog::new(SystemMonotonicClock, &origin, AN_HOUR);

    // When the rules of Gothenburg are searched
    let rules = catalog.find(&gothenburg()).await;

    // Then the search fails because the tax rules are unavailable
    assert!(matches!(rules, Err(TaxRulesUnavailable { .. })));
}
