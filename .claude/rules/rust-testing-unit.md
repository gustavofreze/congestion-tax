---
description: When to write a test that runs without the test database, and when not. The structural residue only, the default stance (no dedicated test for a non-aggregate-root domain type) and the domain-services exception. The decision tree, per-target rulings, and the error-raising technique live in the rust-writing-tests skill.
paths:
    - "tests/application/**/*.rs"
    - "tests/driver/**/*.rs"
    - "tests/driven/cache/**/*.rs"
    - "tests/query/**/http/**/*.rs"
---
<!-- Vendored from payment (.claude/rules/php-testing-unit.md) on 2026-11-18. Adapted to this project: the tests/Unit tree became the tests that run without the test database (the aggregate, the handler, the cache decorator, and the endpoints over {Port}Spy doubles), because this suite has one target and no unit directory. The Redaction carve-out was dropped because the service installs no log redaction, and the phpunit.xml exclude block became the Makefile COVERAGE_FILTER and .cargo/mutants.toml exclude_globs. -->

# Unit tests

Unit tests run without the test database. They are the pure domain tests driven through the aggregate root, and the
tests of the handler, the cache decorator, and the endpoints, whose collaborators across a port are `{Port}Spy`
doubles. They follow every general testing convention from `rust-testing`. This rule keeps only the default stance on
what gets a dedicated test. The decision tree to walk before creating a test file, the per-target rulings, the narrow
cases where a dedicated non-root test is justified, and the technique for testing a domain error through its raising
behavior are in the `rust-writing-tests` skill.

## Pre-output checklist

1. Before creating a test file, walk the decision tree carried by the `rust-writing-tests` skill. Default to NOT
   creating the file and covering the behavior through the aggregate root. See § Default stance.
2. Domain services are the only non-aggregate-root domain type that gets a dedicated test by rule. Every other non-root
   domain type is covered through its aggregate root. See § Default stance.
3. A dedicated test for a non-root domain type requires explicit written justification naming the state or path
   unreachable through the aggregate root. See the rust-writing-tests skill.

## Default stance

By default, do NOT create a dedicated test file for any non-aggregate-root domain type. Coverage for these types
arrives through aggregate-root tests, handler tests, adapter tests, or endpoint tests. The value objects of
`src/application/domain/models/tax_rules/` are covered by `tests/application/domain/models/tax_rules.rs`, the test of
the `TaxRules` aggregate. Creating a dedicated test file requires a written justification (a comment at the top of the
file, or the change description) stating why the condition cannot be reached through any of those paths.

The single by-rule exception is **domain services** (`src/application/domain/services/`), which get a dedicated test
by rule, not by exception. The service has none today.

A read model (`DependencyStatus`) is covered through its endpoint test and its adapter test. A dedicated test is
admitted only when the JSON cannot distinguish a mapping a mutant changes (two fields that serialize to the same value
in every case), and the justification names that mapping.

Aggregate roots (`tests/application/domain/models/`), the handler (`tests/application/handlers/`), the cache decorator
(`tests/driven/cache/`), and the endpoints (`tests/driver/http/`, `tests/query/health/http/`) get dedicated tests here.
The database adapters get theirs under `rust-testing-integration`, because their behavior is the query. Every other
target is routed by the decision tree in the rust-writing-tests skill (shared driver infrastructure such as
`error_response.rs` to the endpoint tests, and the composition root files to the coverage and mutation exclusions in the
`Makefile` `COVERAGE_FILTER` and `.cargo/mutants.toml` `exclude_globs`).
