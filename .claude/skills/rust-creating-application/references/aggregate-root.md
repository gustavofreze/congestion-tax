<!-- Vendored from payment (.claude/skills/php-creating-application/references/aggregate-root.md) on 2026-11-18. Adapted to this project: the eventual, event-recording Payment became the immutable TaxRules aggregate of src/application/domain/models/tax_rules.rs, because this project has no domain events and no generated identity. -->

# Aggregate root (immutable)

Use this when modeling an aggregate root whose constructor receives its child entities, enforces the invariant that
spans them, and answers a domain error instead of an instance that breaks it.

This project's aggregates are immutable. A change returns a new value, and nothing records state through domain events,
because the project has none. The eventual shape (state recorded through events, an outbox written in the same
transaction) has no subject here and is not carried. An aggregate that one day needs it starts from
`rust-hex-application-domain`, not from this file.

```rust
// src/application/domain/models/tax_rules.rs
//
// Immutable aggregate root worked example. The fields are private, so the only way in is new(),
// which validates the invariant spanning the children (no two tax periods overlap) and answers a
// domain error instead of building a broken instance. The constructor takes the children already
// built as value objects, because a child (TaxPeriod) validates its own invariants in its own new().
// The aggregate never reads the clock, the environment, or random identity. An instant it needs
// arrives as an argument. The adapter that reads the rules rebuilds the aggregate through this same
// new(), so stored content that breaks the invariant is refused at the boundary.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxRules {
    periods: Vec<TaxPeriod>,
    currency: Currency,
}

impl TaxRules {
    pub fn new(currency: Currency, periods: impl IntoIterator<Item = TaxPeriod>) -> Result<Self, TaxPeriodsOverlap> {
        let mut chronological: Vec<TaxPeriod> = periods.into_iter().collect();
        chronological.sort_by_key(|period| period.validity().starts_on());

        let overlap = chronological.windows(2).find_map(|pair| match pair {
            [earlier, later] if earlier.validity().overlaps_next(later.validity()) => Some(TaxPeriodsOverlap::new(
                earlier.validity().starts_on(),
                later.validity().starts_on(),
            )),
            _ => None,
        });

        overlap.map_or_else(
            || {
                Ok(Self {
                    periods: chronological,
                    currency,
                })
            },
            Err,
        )
    }

    pub fn tax_for(
        &self,
        vehicle: VehicleType,
        passages: &Passages,
    ) -> Result<TaxAssessment, PassageOutsideTaxPeriods> {
        let days = passages
            .by_day()
            .map(|(date, day_passages)| {
                let period = self
                    .period_on(date)
                    .ok_or_else(|| PassageOutsideTaxPeriods::new(date))?;
                Ok(DailyTax::new(date, period.daily_tax(vehicle, date, day_passages)))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(TaxAssessment::new(self.currency.clone(), days))
    }
}
```
