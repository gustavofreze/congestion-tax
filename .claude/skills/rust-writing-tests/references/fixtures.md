<!-- Vendored from payment (.claude/skills/php-writing-tests/references/fixtures.md) on 2026-11-18. Adapted to this project: the Fixtures class over a DBAL Connection became the functions of tests/support/fixtures/database_fixtures.rs over a sqlx PgPool, and the tearDown truncation became idempotent seeds, because the test database is recreated and migrated by every make tests run. -->

# Fixtures (the single SQL boundary in tests)

Use this when an integration test needs a database, seeds rows, or reads raw state. All database access in `tests/`
goes through `tests/support/fixtures/database_fixtures.rs`. The invariant (the fixtures module is the only place that
executes raw SQL, tests call its public functions and never a `sqlx` primitive directly) is owned by
`rust-testing-integration` (§ The Fixtures helper). This reference carries the shape and the usage.

## The fixtures shape

A module of `pub(crate)` functions. The pools come from here: `migrated_database()` points at the test database that
`make tests` recreates and migrates, reading its connection from the variables the `Makefile` passes into the test
container, and `unreachable_database()` points at a port no database listens on, for the unavailable path. Each seed or
read is named after the business operation on test data (`store_tax_rules_with_overlapping_bands`), never a technical
verb, takes the `&PgPool`, and runs its SQL through `sqlx`. A seed is idempotent (`ON CONFLICT DO NOTHING` on the
literal ids it inserts), because the tests of one run share the database.

```rust
// tests/support/fixtures/database_fixtures.rs

pub(crate) fn migrated_database() -> PgPool {
    let options = PgConnectOptions::new()
        .host(&variable("DATABASE_HOST"))
        .port(
            variable("DATABASE_PORT")
                .parse()
                .expect("DATABASE_PORT is a port number"),
        )
        .database(&variable("DATABASE_NAME"))
        .username(&variable("DATABASE_USER"))
        .password(&variable("DATABASE_PASSWORD"));

    PgPoolOptions::new()
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect_lazy_with(options)
}

pub(crate) async fn store_tax_rules_with_overlapping_bands(database: &PgPool) {
    sqlx::raw_sql(
        r"
        INSERT INTO tax_rules (id, city, currency)
        VALUES ('01a1e6cd-a800-7000-8000-0000000000f1', 'overlapping-bands', 'SEK')
        ON CONFLICT DO NOTHING;
        ",
    )
    .execute(database)
    .await
    .expect("the overlapping bands are stored");
}
```

The real `store_tax_rules_with_overlapping_bands` also inserts the tax period and the two overlapping bands. The excerpt
keeps the shape.

## The test usage

An integration test takes its pool from the fixtures, calls the seed it needs, and hands the pool to the adapter under
test. It never reaches for a `sqlx` primitive of its own.

```rust
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
```

## The prohibited inline-SQL shape

A test that runs SQL directly is prohibited. The moment a test calls `sqlx::query(...)`, `sqlx::raw_sql(...)`, or any
`sqlx` primitive, the statement belongs in a named fixtures function instead. A statement needed by a single test still
goes into the fixtures, the cost of one extra function is small and the cost of raw SQL spread across `tests/` is large.

```rust
// PROHIBITED. Raw SQL inside a test, never through the fixtures.
sqlx::raw_sql("DELETE FROM tax_rules").execute(&database).await.expect("rows deleted");
```
