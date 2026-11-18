---
description: Testing conventions. The structural residue only, Given/When/Then one-When discipline, tests mirror src under the single tests/main.rs target, the real-objects and {Port}Spy double policy, and the 100 percent line coverage and caught-mutant gate. The Given/When/Then, builder, fixture, and HTTP-boundary shapes and the coverage read procedure live in the rust-writing-tests skill.
paths:
    - "tests/**/*.rs"
---
<!-- Vendored from payment (.claude/rules/php-testing.md) on 2026-11-18. Adapted to this project: PHPUnit became cargo test with rstest and tokio::test, the doc-comment annotations became // Given, // And, // When, // Then comments, the tests/Unit and tests/Integration trees and the Test\ namespace became one tests/main.rs target whose modules mirror src/, the expectException ordering became a captured Result asserted after the When, Spy-suffixed classes became {Port}Spy structs under tests/support/spies/, and Infection with branch coverage became cargo llvm-cov (lines only) and cargo-mutants. The autoload-dev paragraph became the second-target case. -->

# Testing conventions

Framework: the built-in harness (`cargo test`), with `rstest` for case matrices and `#[tokio::test]` for async tests.
The worked examples, the Given/When/Then shapes, the builder and fixture templates, the `#[case]` shape, and the
coverage read procedure live in the `rust-writing-tests` skill, surfaced when that skill runs. This path-injected rule
keeps the always-on invariants only.

## Pre-output checklist

1. Body uses `// Given`, `// And`, `// When`, `// Then` comments with exactly one `// When` per test. A failure is
   captured in the `// When` and asserted in the `// Then`. See § One When per test.
2. Test file lives at the path mirroring its production counterpart under `tests/`, with the same file name, declared
   as a module of `tests/main.rs` nested like the `src/` tree. The function is named
   `<action>_when_<condition>_then_<outcome>`. The one admitted exception is a cross-cutting test at the root of
   `tests/`. See § Test paths and modules.
3. Real objects for collaborators inside the domain. Hand-written doubles are named `{Port}Spy` only, never `Mock`,
   `Stub`, `Fake`, or `Dummy`, and no mocking crate is used. See § Doubles.
4. 100 percent line coverage and every mutant caught are required. See § Coverage and mutation.

## One When per test

Every test uses `// Given`, `// And`, `// When`, `// Then` comments, each followed by a short phrase in domain terms,
with exactly one `// When` per test. Two actions require two tests. The single exception is a repeated-invocation test
(idempotence, caching, memoization): when the purpose is asserting that the same operation produces the same outcome
across N invocations, the `// When` block may contain N identical invocations captured in numbered bindings (`first`,
`second`) under a composite phrase (`// When the rules of Gothenburg are searched twice`). Rust has no expectation that
must precede the action, so a failure test binds the `Result` in the `// When` and asserts it in the `// Then` with
`assert!(matches!(rules, Err(TaxRulesUnavailable { .. })))`. Use `// And` for complementary preconditions or
assertions to avoid consecutive `// Given` or `// Then` blocks.

The happy-path, failure, `#[case]`, builder, and HTTP-boundary Given/When/Then shapes are in the rust-writing-tests
skill.

## Test paths and modules

Tests mirror the production layout under `src/` with the same relative path and file name under `tests/`:

- Production: `src/driven/postgres/postgres_tax_rules_catalog.rs`
- Test: `tests/driven/postgres/postgres_tax_rules_catalog.rs`

`tests/main.rs` is the only test target (`autotests = false`, one `[[test]]` in `Cargo.toml`). It declares every test
file as a module nested exactly like the `src/` tree
(`mod driven { mod postgres { mod postgres_tax_rules_catalog; } }`), so a file that is not declared there never
compiles. The support code lives in `tests/support/builders/`,
`tests/support/spies/`, and `tests/support/fixtures/`, declared `pub(crate)` at the top of `tests/main.rs`, one type per
file (the enum a spy answers with gets its own `tax_rules_catalog_outcome.rs`). A test reaches production code through
the crate's public API (`congestion_tax::...`) only, and no `#[cfg(test)]` module sits inside `src/`.

There is no unit and integration directory split. Whether a test needs the test database decides which of
`rust-testing-unit` and `rust-testing-integration` governs it, and both keep the mirror path.

The one admitted home that mirrors no `src/` file is a module at the root of `tests/`, declared at the root of
`tests/main.rs`, for a cross-cutting test no single production file owns. The file opens with a one-line comment naming
the behavior it guards and why no `src/` file mirrors it. None exists today.

The endpoint tests drive `routes::router` with `tower::ServiceExt::oneshot`. A write endpoint validates, builds a
command, and delegates through an inbound port, so substituting that port with a `{Port}Spy` leaves the whole behavior
under test. A read endpoint takes a spy of its read port only when it carries behavior of its own beyond the query (the
readiness endpoint maps a status to a code and rounds the duration), and the database adapter behind that port then has
its own test against the test database. A read endpoint whose behavior IS the query is exercised against the test
database instead, because a double in its place returns whatever it was handed and the test asserts nothing.

A second test target, or a test file outside the mirror path, is non-conformant, and the fix belongs to the layout, not
to the new file. Report it as a finding, and never split the suite across two conventions to satisfy this rule.

The per-target rulings on what gets a test are in the rust-writing-tests skill.

## Doubles

Real objects for collaborators inside the domain. An internal domain collaborator is never substituted with a test
double. Spies appear only at ports and system boundaries (database, HTTP, clock): `TaxRulesCatalogSpy`,
`CalculatingCongestionTaxSpy`, `DependencyProbeSpy`, `MonotonicClockSpy`. Every double is a hand-written struct in
`tests/support/spies/` that implements the port trait, is named `{Port}Spy` exclusively, answers through named
constructors (`holding`, `empty`, `unavailable`, `failing`), and records calls through inspection methods
(`searched_cities()`, `received_commands()`). When the test keeps the spy to inspect it after handing it over, the spy
implements the port for `&{Port}Spy`. The suffixes `Mock`, `Stub`, `Fake`, and `Dummy` are prohibited, and any type
still ending in one of them is renamed to end in `Spy` during any review or update pass. Binding names use the business
concept the object represents (`catalog`, `origin`, `clock`), never `spy` or `mock`.

The builder, fixture, and HTTP-boundary double shapes are in the rust-writing-tests skill.

## Coverage and mutation

`make tests` fails below 100 percent of lines (`cargo llvm-cov` with `--fail-under-lines 100`), then runs
`cargo mutants`, and every mutant must be caught. Stable `cargo llvm-cov` measures no branches, so lines are the
coverage gate. A line or mutant that cannot be covered or caught signals a design problem in the production code:
refactor the code, never silence the tool with an attribute or a new exclusion. The composition root files are the one
exclusion, carried in the `Makefile` `COVERAGE_FILTER` and in `.cargo/mutants.toml` `exclude_globs` (owned by
`rust-skeleton-tooling`). The procedure for reading `reports/coverage/html/index.html` and
`reports/mutants.out/outcomes.json` is in the rust-writing-tests skill.

Every code-style rule applies to test code without exception, exactly as in production. `clippy.toml` lifts only the
panic family for the test target, so a test calls `expect` with the assumption it relies on
(`expect("the catalog is available")`).
