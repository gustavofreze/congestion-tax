<!-- Vendored from payment (.claude/skills/php-writing-tests/references/test-data-builder.md) on 2026-11-18. Adapted to this project: the PHP PaymentBuilder became the TaxRulesBuilder of tests/support/builders/tax_rules_builder.rs (a named seeded constructor, with_ methods that consume and return the builder, build(self)), the transition shortcuts became named constructors over the real domain constructors, and the event-count usage was dropped because this project has no domain events. -->

# Test Data Builder

Read this while constructing a test subject. None of it is a per-file invariant, so it is NOT path-injected. The
threshold invariant and the `build()`-placement invariant are defined here (the `>5`-parameter threshold and the
`build()`-chaining placement). This file holds the construction how-to, the builder shape, and the two worked usages.

Aggregate roots with large construction signatures cannot be built inline in every `// Given` without drowning the
scenario in noise. For those, a **Test Data Builder** under `tests/support/builders/` is the single sanctioned
construction helper. It is the construction analog of the single-SQL fixtures boundary in `rust-testing-integration`:
one builder per aggregate, consumed by every test of that aggregate.

## Contents

- Builder versus inline construction
- Commands with large signatures
- Builder shape
- Using the builder in a test
- Cover actions and invariants, not accessors
- Grouping assertions
- The TaxRulesBuilder worked builder
- Usage 1, creation as the action under test
- Usage 2, a domain action with creation as setup

## Builder versus inline construction

- **Inline via the production constructor** when construction takes **5 parameters or fewer** (value objects, result
  types, small entities). For example `Money::new(Amount::new(amount), swedish_kronor())` with two parameters. No
  builder.
- **Builder** when the aggregate, or a child it cannot exist without, takes more than 5 parameters. `TaxRules::new`
  takes two, but each of its `TaxPeriod` children takes six, so the aggregate gets `TaxRulesBuilder`. Inlining all of
  them in each test is prohibited noise.

A builder wrapping a constructor of 5 parameters or fewer is surplus. Never write one.

## Commands with large signatures

A `src/application/commands/<command>.rs` data carrier with more than 5 fields is built through a Test Data Builder
under `tests/support/builders/<command>_builder.rs`, on the same terms as the aggregate builder above (a named
constructor seeded with valid defaults, `with_<field>` methods, `build()` producing the real command). This is the only
sanctioned way to construct a large command in a test. Inlining twenty-plus fields in every `// Given` is wrong. A
command of 5 fields or fewer is built inline, or through a private helper function in the test file
(`command(city, vehicle, passages)` in the handler test). A command builder carries no state shortcuts, because a
command has no states.

## Builder shape

- Path `tests/support/builders/<aggregate>_builder.rs`, declared in `tests/main.rs` under `support::builders`.
- A `pub(crate)` struct named `<Aggregate>Builder`, subject to **every** code-style rule (member and parameter ordering
  by name length ascending, no abbreviations, no generic verbs).
- A named constructor seeded with valid defaults that build a coherent aggregate with no further configuration, named
  after the state it seeds (`gothenburg()`), a `const fn` when it can be.
- `with_<field>(mut self, ...) -> Self`. Chainable, each call consuming the builder and returning it, so no caller can
  observe a half-configured builder.
- `build(self) -> <Aggregate>`. Calls the production constructor, with `expect` naming why the seeded values are valid.
- State shortcuts. Named constructors returning a pre-configured builder that reach a state through the **real domain
  constructors and transitions** in `build()`, never by writing private fields. They carry **fixed internal defaults**
  for the transition inputs and expose **no** `with_` method for those inputs.
- Helpers the tests of the aggregate share (`swedish_kronor()`, `gothenburg_period(...)`) live as `pub(crate)` free
  functions in the builder file, and a helper the builder alone needs stays private.

## Using the builder in a test

1. **Business name, never the helper's.** The binding is `rules`, never `builder`, even while it still holds the
   builder. The test describes the domain, not the scaffolding. The same holds for the comments. A `// Given`, `// And`,
   `// When`, or `// Then` never names the builder. Write `// Given the Gothenburg tax rules`, never
   `// Given a tax rules builder for gothenburg`.
2. **`build()` chains onto the last link of the step where creation happens.** It is never a standalone
   `// When ... build()`, which narrates the helper rather than the domain. Creation lands on:
    - `// When` when **creation is the action under test** (invariant checks: normalization, validation, boundary
      acceptance or rejection).
    - `// Given` when creation is **setup** for a later domain action (the `// When` is then that action).
3. **Values that matter, one per `// And`.** When a value drives the assertion (a daily maximum, a boundary, a value a
   transition injects into the result), declare it in its own step calling the relevant `with_` method or domain
   method with the real value. Values irrelevant to the test stay in the seeded defaults. Do not spell them out.
4. **State shortcuts only when the path is irrelevant.** If the assertion checks only the direct effect of the
   `// When` without depending on how the aggregate reached the prior state, the shortcut is fine. If the assertion
   depends on the intermediate steps, spell those steps out as `// And` steps instead of hiding them in a shortcut.

## Cover actions and invariants, not accessors

A test exercises a **domain action**, either a constructor that validates or a behavior. It never asserts that a
constructor stored what it was given. A test of the form "create with value X then the getter returns X" is surplus: it
exercises the accessor, not behavior. Remove it. Cover invariants by repeating the **same action with different values**
that drive each one: a daily maximum below, at, and above the charges of the day, a band that ends before it starts and
one that ends when it starts, a boundary that is accepted or rejected.

## Grouping assertions

One logical concept per `// Then` block. When the verification spans distinct logical groups, separate them with
`// And` and a blank line. Never use a single block stacking every assertion.

## The TaxRulesBuilder worked builder

This builder is the real one. A builder written from it resolves its imports from the crate (`<RootNamespace>::`,
read from `Cargo.toml`) before saving.

```rust
// tests/support/builders/tax_rules_builder.rs

pub(crate) struct TaxRulesBuilder {
    daily_maximum: u32,
}

impl TaxRulesBuilder {
    pub(crate) const fn gothenburg() -> Self {
        Self { daily_maximum: 60 }
    }

    pub(crate) fn build(self) -> TaxRules {
        TaxRules::new(
            swedish_kronor(),
            [gothenburg_period(
                date(2013, 1, 1),
                date(2013, 12, 31),
                self.daily_maximum,
            )],
        )
        .expect("a single period overlaps no other")
    }

    pub(crate) const fn with_daily_maximum(mut self, daily_maximum: u32) -> Self {
        self.daily_maximum = daily_maximum;
        self
    }
}

pub(crate) fn swedish_kronor() -> Currency {
    Currency::new("SEK").expect("SEK is an ISO 4217 code")
}
```

## Usage 1, creation as the action under test

Creation as the action under test (an invariant), with the constructor on the `// When`.

```rust
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

## Usage 2, a domain action with creation as setup

A domain action with creation as setup, the value that drives the assertion declared through `with_`, and grouped
assertions.

```rust
#[rstest]
#[case::below_the_charges_of_the_day(25, 25)]
#[case::equal_to_the_charges_of_the_day(26, 26)]
#[case::above_the_charges_of_the_day(27, 26)]
fn tax_for_when_daily_maximum_is_around_the_charges_of_the_day_then_owes_at_most_the_daily_maximum(
    #[case] daily_maximum: u32,
    #[case] amount: u32,
) {
    // Given tax rules with a daily maximum around 26 kronor
    let rules = TaxRulesBuilder::gothenburg().with_daily_maximum(daily_maximum).build();

    // When the tax of a car charged 8 kronor at 06:00 and 18 kronor at 07:30 is calculated
    let assessment = rules
        .tax_for(
            VehicleType::Car,
            &passages(&["2013-02-05 06:00:00", "2013-02-05 07:30:00"]),
        )
        .expect("every passage falls within a tax period");

    // Then the car owes the charges of the day, capped at the daily maximum
    assert_eq!(assessment.total(), kronor(amount));
}
```
