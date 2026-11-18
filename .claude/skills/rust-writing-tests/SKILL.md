---
name: rust-writing-tests
description: Creates tests for an artifact, routing between unit and integration by target type. Use when asked to write, add, or complete tests, raise coverage, kill surviving mutants, or do TDD.
---
<!-- Vendored from payment (.claude/skills/php-writing-tests/SKILL.md) on 2026-11-18. Adapted to this project: PHPUnit and Infection became cargo test with rstest, cargo llvm-cov, and cargo-mutants in the single tests/main.rs target, unit and integration became a database-free test versus one against the PostgreSQL test database, and the outbound HTTP boundary references were dropped (no gateway here). -->

# Write tests

Creates tests for an existing artifact, routing the scope by the target. Routes to the testing rules, and carries the
worked shapes and the generative how-to that used to live inside the rules.

Target: the artifact to test, named in the request (e.g. `TaxRules` or `PostgresTaxRulesCatalog`).

Start from the simplest test that drives the behavior. Add machinery only when the target earns it: a Test Data Builder
past five construction parameters, an rstest `#[case]` matrix for a table of cases, never a dedicated test for what the
decision tree rules out.

Every test lives in the one test target, `tests/main.rs` (`autotests = false`), in a file under `tests/` that mirrors
the path of its subject under `src/`, declared as a module in `tests/main.rs`. Shared helpers live in
`tests/support/builders/`, `tests/support/spies/`, and `tests/support/fixtures/`, one type per file. A unit test runs
with no database, with a hand-written `{Port}Spy` at the port it crosses. An integration test reaches the dedicated
PostgreSQL test database, migrated by `make tests`, through the pools in `tests/support/fixtures/`.

## When to use

- Write, add, or complete tests, raise coverage, kill surviving mutants, do TDD.

## When NOT to use

- Create the artifact under test: use the layer action, which already delegates here (`rust-creating-driver`,
  `rust-creating-driven`, `rust-creating-query`, `rust-creating-application`, or `rust-implementing-features`).
- Give a dedicated test to what the decision tree forbids (a value object, enum, error, or other non-root covered
  through behavior): walk `references/unit-test-decision-tree.md` first.
- A handler as a unit test: a handler is always integration, per the `rust-testing-unit` default stance.
- Documentation or configuration changes with no behavior to exercise.

## Rules applied

Load and follow, in this priority:

- `rust-testing`: BDD Given/When/Then, doubles, Test Data Builders, organization, and coverage.
- `rust-testing-unit`: what NOT to unit-test and the default stance before creating a test file.
- `rust-testing-integration`: the fixtures as the single SQL boundary and the round-trip.

## Shapes and how-to (read when scaffolding)

The literal shapes and the generative how-to that the testing rules used to inline live here, surfaced only when this
skill runs:

- `references/bdd-given-when-then.md`: the happy-path and failure Given/When/Then shapes.
- `references/error-assertion.md`: the prohibited conditional or message-substring assertion versus the correct
  `matches!` on the error variant.
- `references/test-data-builder.md`: builder versus inline construction, the command variant, `build()` placement,
  state shortcuts via the real constructors, cover-actions-not-accessors, the `TaxRulesBuilder` worked builder, and the
  two worked usages (creation as the action, creation as setup).
- `references/fixtures.md`: the database fixtures shape (the single SQL boundary in tests), the test usage, and the
  prohibited inline-SQL shape.
- `references/rstest-cases.md`: the `#[rstest]` matrix with named `#[case::...]` rows and parameter-named arguments.
- `references/error-raising-behavior.md`: the prohibited error-as-value-object test versus driving the call path that
  answers it.
- `references/coverage-and-mutation.md`: the read-order procedure over the cargo-mutants outcome files and the
  cargo-llvm-cov report, with the pass criterion.
- `references/unit-test-decision-tree.md`: the decision tree, the per-target rulings, the write-versus-read endpoint
  split, when a dedicated non-root test is justified, and testing a domain error through its raising behavior.

## Assembly order

1. Decide the scope by the target. Domain (aggregate root) and WRITE endpoint adapters (`src/driver/http/`) go to unit.
   READ endpoint adapters (`src/query/**/http/`) go to integration, because a read endpoint's behavior is the query and
   a doubled query trait asserts nothing. Handler, database adapter, and any end-to-end flow go to integration too. A
   handler is never a unit test, per the `rust-testing-unit` default stance and `references/unit-test-decision-tree.md`
   item 4.
2. Before creating a test file, walk the decision tree (`references/unit-test-decision-tree.md`).
3. BDD Given/When/Then comments, with a single `// When` (`references/bdd-given-when-then.md`).
4. A Test Data Builder when an aggregate or a command has more than five construction parameters
   (`references/test-data-builder.md`).
5. Integration always through `tests/support/fixtures/`, the single SQL boundary in the tests.
6. Declare the new file in `tests/main.rs`, then run the suite and iterate until it closes
   (`references/coverage-and-mutation.md`).

## Completeness gate

- [ ] Right path: the file mirrors its subject under `tests/` and is declared in `tests/main.rs`. A read endpoint
      adapter test reaches the test database, never a doubled query trait.
- [ ] Nothing the decision tree forbids got a dedicated test.
- [ ] Test names read `<action>_when_<condition>_then_<outcome>`, and doubles are hand-written `{Port}Spy` types.
- [ ] BDD with one `// When`, covering action and invariant, never accessors.
- [ ] No `if`, `match`, loop, or early return inside a test body. A matrix of inputs is one `#[rstest]`.
- [ ] Runnable check: the suite passes, coverage and mutation per the rules. Run it through the project's own suite
      target, whatever it is called. Where the canonical `make tests` is absent, use the equivalent the `Makefile` does
      define and report the missing target as a tooling finding, never skip the verification.

## Does not do

- Does not create the artifact under test. The layer action owns that and delegates here.
- Does not give a dedicated test to what the decision tree forbids (value object, enum, error, or other non-root).
- Does not unit-test a handler, which is always integration per the `rust-testing-unit` default stance.
- Does not exercise documentation or configuration changes that carry no behavior.
