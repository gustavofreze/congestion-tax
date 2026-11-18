<!-- Vendored from payment (.claude/skills/php-writing-tests/references/bdd-given-when-then.md) on 2026-11-18. Adapted to this project: the PHPUnit doc-comment annotations became // Given, // When, // Then comments, the expectException ordering became an assertion on the answered Result, and both shapes are worked from tests/. -->

# BDD Given-When-Then shapes

Use these shapes when writing a behavioral test, one happy path and one failure path, each with exactly one `// When`
step.

```rust
// Given/When/Then shapes. Every test uses // Given, // And, // When, // Then comments, each
// followed by a short phrase in domain terms, and exactly one // When per test. Every failure is a
// value, so a failure test keeps the same order as a happy path: the // When captures the Result,
// and the // Then asserts on it. The test name reads <action>_when_<condition>_then_<outcome>.

// Happy path (tests/application/domain/models/tax_rules.rs)
#[test]
fn tax_for_when_vehicle_made_no_passages_then_owes_nothing() {
    // Given the Gothenburg tax rules
    let rules = TaxRulesBuilder::gothenburg().build();

    // When the tax of a car with no passages is calculated
    let assessment = rules
        .tax_for(VehicleType::Car, &passages(&[]))
        .expect("no passage falls outside a tax period");

    // Then the car owes nothing
    assert_eq!(assessment.total(), kronor(0));
}

// Failure path. The // When keeps the Result, and the // Then matches the error variant.
#[test]
fn new_when_tax_periods_share_a_day_then_fails() {
    // Given a period ending on 2013-12-31
    let year_2013 = gothenburg_period(date(2013, 1, 1), date(2013, 12, 31), 60);

    // And a period starting on that same day
    let overlapping = gothenburg_period(date(2013, 12, 31), date(2014, 12, 31), 60);

    // When the tax rules are created from both periods
    let rules = TaxRules::new(swedish_kronor(), [overlapping, year_2013]);

    // Then the tax rules are refused because the periods overlap
    assert!(matches!(rules, Err(TaxPeriodsOverlap { .. })));
}
```
