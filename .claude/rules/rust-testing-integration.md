---
description: Integration test infrastructure. The structural residue only, all raw SQL in tests/ goes through tests/support/fixtures/database_fixtures.rs, the single file executing sqlx primitives, and an aggregate decoded from a stored document has a load test against the seeded database. The fixture example, the usage shape, and the load-test how-to live in the rust-writing-tests skill.
paths:
    - "tests/driven/postgres/**/*.rs"
    - "tests/query/**/database/**/*.rs"
    - "tests/support/fixtures/**/*.rs"
---
<!-- Vendored from payment (.claude/rules/php-testing-integration.md) on 2026-11-18. Adapted to this project: the tests/Integration tree became the tests that reach the dedicated PostgreSQL test database, the Fixtures class over DBAL became the free functions of tests/support/fixtures/database_fixtures.rs over sqlx, and the reflection round-trip test became a load test against the seeded rules, with the save half and the outbox version check dropped because nothing saves an aggregate. The HTTP transport fake and the messaging transport were dropped because the service has neither. -->

# Integration tests

Integration tests exercise the adapters that reach `<database-engine>` against the dedicated test database:
`PostgresTaxRulesCatalog` (`tests/driven/postgres/`), `PostgresDatabaseProbe` (`tests/query/health/database/`), and
`PostgresTaxRulesFinding` (`tests/query/tax_rules/database/`), whose read endpoint test runs against the same database.
`make tests` and `make test-file` drop and recreate that database, apply the Flyway migrations to it, and hand its
coordinates to the test container as the `DATABASE_*` variables. They follow every general testing convention from
`rust-testing`. This rule keeps two invariants: the single SQL boundary in tests and the aggregate load test.

## Pre-output checklist

1. All database access in `tests/` goes through `tests/support/fixtures/database_fixtures.rs`. Test files call its
   functions, never `sqlx` primitives directly. See § The fixtures module.
2. The fixtures module is the only place that executes raw SQL. See § The fixtures module.
3. An aggregate decoded from a stored document has a load test that compares it with the aggregate its builder makes.
   See § Load test.

## The fixtures module

`tests/support/fixtures/database_fixtures.rs` holds `pub(crate)` free functions, exposed as test utilities. It is the
**only** place in `tests/` that builds a pool or executes raw SQL (`sqlx::raw_sql`, `sqlx::query`, `.execute`). Every
integration test consumes its functions: `migrated_database()` for a lazy pool on the test database,
`unreachable_database()` for a pool on a port no database listens on, and the store functions for extra rows. A test
file never calls an `sqlx` primitive itself.

Its functions are named after the business operation they perform on test data
(`store_tax_rules_with_overlapping_bands`, never `run_query` or `execute_sql`), grouped by aggregate, and subject to
every code-style rule without relaxation. A query needed by a single test still goes into the fixtures module: the cost
of one extra function is small, the cost of letting raw SQL spread across `tests/` is large.

The suite runs its tests in parallel against one database that is never truncated between them. The rows the
migrations seed (the rules of `gothenburg`) are the shared baseline and are never inserted again, and a store function
writes fixed UUIDv7 ids with `ON CONFLICT DO NOTHING` under a city no other test reads (`overlapping-bands`).

The fixture example, the test usage, and the prohibited inline-SQL shapes are in the rust-writing-tests skill.

## Load test

An aggregate rebuilt from a stored document (`TaxRules` from the JSON row of `FIND_TAX_RULES`, decoded through
`tax_rules_document/`) has a dedicated load test against the seeded database, because a swapped or misread field
decodes without error into a different valid aggregate. The test loads the aggregate through the adapter and asserts
it equals the one the builder makes (`TaxRulesBuilder::gothenburg().build()`), so every decoded field is compared at
once. A test of the aggregate alone does not exercise the decoding path and does not satisfy this invariant. It guards
the rehydration contract owned by `rust-hex-driven-repository` and `rust-hex-sql-queries` § Row decoding. The adapter
test also covers stored content that breaks an invariant and an unreachable database, both answering
`TaxRulesUnavailable`.

The load test steps and the worked shape are in the rust-writing-tests skill.
