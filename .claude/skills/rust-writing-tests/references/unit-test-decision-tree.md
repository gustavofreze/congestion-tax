<!-- Vendored from payment (.claude/skills/php-writing-tests/references/unit-test-decision-tree.md) on 2026-11-18. Adapted to this project: the src/ and tests/Unit or tests/Integration paths became the Rust modules and the single tests/main.rs target, the phpunit.xml exclusion became the Makefile coverage filter and .cargo/mutants.toml, the transactional-wrapper and Redaction exceptions were dropped (neither exists here), and the settings item folded into the composition root item. -->

# Unit-test scope decision tree (read before creating a test file)

Read this while deciding whether a target gets a dedicated unit test. None of it is a per-file invariant, so it is NOT
path-injected. The default stance and the kept invariants live in `rust-testing-unit`. This file holds the decision
support that used to sit inside that rule.

Unit tests are pure domain tests driven through the aggregate root. By default, do NOT create a dedicated test file for
any non-aggregate-root domain type. Coverage arrives naturally through aggregate-root tests, integration tests, or
endpoint adapter tests. The single exception by rule (the closed set is owned by `rust-testing-unit`) is **domain
services** (`src/application/domain/services/`, absent until the first one exists).

Every test file lives in the one target, `tests/main.rs`. "Unit" below means a test that runs with no database, with a
`{Port}Spy` at the port it crosses. "Integration" means a test that reaches the PostgreSQL test database through
`tests/support/fixtures/`. Both sit at the path under `tests/` that mirrors their subject under `src/`.

## Contents

- Decision tree (consult before creating any test file)
- Details
- When a dedicated test IS justified
- Testing a domain error through the raising behavior

## Decision tree (consult before creating any test file)

Before writing a new test file, walk these questions in order. The first match dictates the answer.

1. **Is the target under `src/driver/http/`?**
    - If it is an endpoint function (or its request), write the endpoint adapter test, a **unit** test at the mirrored
      `tests/driver/http/` path, driving the real `router` from `src/routes.rs` with `tower::ServiceExt::oneshot`. A
      write endpoint delegates through an inbound port, so a `Spy` in that port's place
      (`CalculatingCongestionTaxSpy`) leaves the endpoint's whole behavior (parsing, command construction, status
      mapping) under test.
    - Otherwise, NO TEST. It is shared driver infrastructure (the error mapping, the responses). Covered by the endpoint
      adapter tests that exercise it. See "Shared driver infrastructure" below.
2. **Is the target under `src/query/`?**
    - If it is an endpoint function under an `http/` module, write the endpoint adapter test as an **integration** test
      at the mirrored `tests/query/` path, with the slice's real database adapter over `migrated_database()`. Never a
      unit test with a doubled query trait: a read endpoint's behavior IS the query (the projection, the filters, the
      scope predicate), so a double returns whatever it was handed and the test asserts nothing. See "Read endpoint
      adapters" below.
    - If it is the slice's database adapter, write an **integration** test against the test database, as
      `tests/query/health/database/postgres_database_probe.rs` does.
    - If it is a read model, a response, or a query constant, it carries no dedicated test. It is covered through the
      read endpoint adapter test that exercises it. A read-model test is admitted only when the JSON cannot distinguish
      a mapping a mutant changes (`rust-testing-unit` § Default stance).
3. **Is the target under `src/driven/`?**
    - If it is the unmapped-failure arm of an adapter that translates a constraint violation into a domain error, write
      a **unit** test at the mirrored path covering only the fallback arm, the one that maps a failure no translation
      matches, which the real database does not raise on demand. Every mapped arm goes to the integration test that
      provokes the real violation, and that test remains mandatory. See Exception A below.
    - Otherwise, NO unit test. Cover it with an integration test against the real external system, the PostgreSQL test
      database. A pure-logic `src/driven/` collaborator with no external system (a decorator over a port, a strategy
      selection) is still NOT a unit-test target. Its behavior is covered by the integration test of the handler or
      flow that runs the real port chain. The justification escape hatch in "When a dedicated test IS justified" is
      for domain types (value objects, enums) only, never for a `src/driven/` type. The sole dedicated unit test under
      `src/driven/` remains the unmapped-failure arm of a constraint translation (Exception A). See "Driven adapters"
      below.
4. **Is the target under `src/application/handlers/`?**
    - **NO unit test.** Cover it with an integration test that exercises the full domain plus database path.
5. **Is the target part of the composition root?** (`src/main.rs`, `src/app_settings.rs`, `src/database_settings.rs`,
   `src/environment_variable.rs`)
    - **NO TEST.** The composition root wires the graph and reads the typed configuration once at startup. It contains
      no business logic, and only these files may read the environment (`AGENTS.md` § Project-specific rules), so a
      settings type never lives anywhere else. They are excluded from coverage by `COVERAGE_FILTER` in the `Makefile`
      and from mutation by `exclude_globs` in `.cargo/mutants.toml`, and a new composition-root file joins both lists.
      `src/routes.rs` is NOT part of that exclusion: it is library code, and the endpoint tests that build `router`
      cover it. A guard a route carries is proven by a test that builds `router` with the inbound ports replaced by
      `Spy` doubles, asserting the refusal status and that the port was never reached.
6. **Is the target a domain type other than an aggregate root?** (value object, enum, internal entity, domain error,
   command)
    - **NO unit test by default.** Coverage arrives through the aggregate root that consumes it. See "Domain types
      covered through the aggregate" below for the narrow exceptions.
7. **Is the target a domain service under `src/application/domain/services/`?**
    - **WRITE A DEDICATED UNIT TEST** under `tests/application/domain/services/`. Domain services are pure functions
      over domain objects (no I/O, stateless) and have a public contract worth testing in isolation. This is an
      explicit exception to item 6: domain services are not internal domain types, they are domain operations. The
      structural contract is owned by the domain rules.
8. **The target is an aggregate root or an endpoint adapter**: write the test.

## Details

**Shared driver infrastructure**: every file under `src/driver/http/` that is NOT an endpoint function or its request.
That is the error mapping (`error_response.rs`) and every response (`*_response.rs`).

Their behavior is covered by the endpoint adapter tests that exercise each parsing branch and each error-to-HTTP
translation. A dedicated test for any of these files duplicates coverage and bypasses the real execution path. Any
existing dedicated test of the error mapping or of a response MUST be deleted. Its scenarios belong in the
corresponding endpoint tests. A service with no domain endpoint yet has no error mapping and no test for it, and the
arms are born with the first endpoint that reaches them, covered by that endpoint's test.

The error mapping reaches 100% line and mutation coverage exclusively this way. The endpoint test builds the `router`
over a `Spy` of the inbound port answering the failure under test, drives a request, and asserts the translated HTTP
status and the error code. Every arm of the mapping is exercised the moment an endpoint test reaches the failure it
maps. An arm with no endpoint test reaching its failure is uncovered code, closed by adding the missing endpoint
scenario, never by a test of the mapping itself. The prohibition is owned here, and the mapping contract it tests (one
arm per failure) by `rust-hex-driver-http` § Error to HTTP mapping.

The same applies to the request and the responses themselves. Their coverage lives in the endpoint test that
deserializes the HTTP payload into the request and serializes the response back. A dedicated request or response test
asserts only round-trip field behavior, which is a tautology.

**Read endpoint adapters** (`src/query/**/http/<name>_endpoint.rs`). The one endpoint kind that is an integration test,
and the reason is the shape of the thing rather than a preference. A write endpoint parses, builds a command, and hands
it to an inbound port, so the port is a real seam: put a `Spy` there and the endpoint's own behavior stays under test.
A read endpoint has a query trait too, but nothing worth testing sits on this side of it. The projection, the joins,
the filters, the scope predicate, the ordering are all in the SQL behind that trait, so a double returns whatever the
test handed it and the assertion is a tautology. Write the test under the mirrored `tests/query/` path, build the
slice's real database adapter over `migrated_database()`, and drive the endpoint over the real `router`.

Everything else under `src/query/` follows from that: read models, responses, query constants, and the slice's error
mapping carry no dedicated test, because the read endpoint test that exercises each branch already covers them. This
mirrors how shared driver infrastructure is covered on the write side.

**Driven adapters**: database adapters under `src/driven/<technology>/` (`PostgresTaxRulesCatalog`), their decoded
shapes, and the decorators over their ports (`CachedTaxRulesCatalog`). They are covered exclusively by integration
tests, with the database real. **Exception A**: an adapter that translates a constraint violation into a domain error
(a unique violation read with `is_unique_violation()` translated to `<Aggregate>AlreadyExists`) gets a dedicated unit
test covering only the fallback arm, the one that maps a failure no translation matches, which the real database does
not raise on demand. Every mapped arm is covered by the integration test that provokes the real violation, and that
test remains mandatory.

**Handlers** (`src/application/handlers/`). Thin orchestrators whose behavior is meaningful only with real
collaborators (domain aggregate plus database). Cover them with integration tests exclusively.

**Domain types covered through the aggregate**:

- **Value objects used exclusively by a single aggregate**: the value object's invariants (e.g., "a city cannot be
  empty") are exercised when the aggregate or the handler receives an invalid primitive and the value object's
  constructor answers its error. **Exception**: a value object may have a dedicated test when it exposes non-trivial
  public behavior that cannot be reached through the aggregate's public API. For example, a rich parsing method,
  arithmetic, or a comparison. A value object with only a `new` and an accessor does NOT qualify.
- **Enums in the domain layer**: enums with methods are covered through the aggregate that exercises each variant.
  **Exception**: a dedicated enum test is justified ONLY when the enum has non-trivial standalone methods (e.g.,
  `PaymentStatus::can_transition_to(self, other: Self) -> bool` with branching logic).
- **Internal entities** (non-root entities owned by an aggregate). Covered through the aggregate root. Dedicated tests
  only when a specific invariant is structurally unreachable through the root's public API. And at that point, the
  design is suspect: ask first whether the invariant belongs on the root.
- **Domain errors** (`src/application/domain/errors/`). They carry no behavior beyond their `#[error("...")]` message
  and the context their constructor receives. They are covered by the aggregate test that asserts the error is answered
  under the invariant-violating condition. The same test asserts the carried context when it matters. A dedicated test
  file for an error is NEVER justified.
- **Commands** (`src/application/commands/`). Data carriers with public fields only. Nothing to test.
- **Decoded shapes** (`src/driven/<technology>/<document>/`). Covered through the integration tests that exercise the
  full adapter path, because the decoding is consumed by the adapter under test.

## When a dedicated test IS justified

The justification comment must state one of:

- "Behavior X cannot be reached through aggregate Y because it requires state Z that the aggregate refuses to enter."
  (Design smell. Prefer refactoring to expose the path through the root.)
- "This value object is consumed across N aggregates and has standalone parsing logic worth isolating."
- "This enum has a branching method `<method>()` whose truth table is larger than the cases exercised by any single
  aggregate."

Absence of the justification means the dedicated test is surplus and must be removed.

## Testing a domain error through the raising behavior

Error types are pure invariant-violation signals owned by the domain. They are not the subject of behavior tests, and
the default stance already rules out a dedicated test file for one. A test constructs the conditions, invokes the public
method that is supposed to fail, and asserts the expected error variant is answered. Constructing the error directly and
asserting on its message or fields is prohibited: the error's structure is exercised through the call path that
produces it. Assert the whole message with `assert_eq!` only where the message is part of the contract, as in an
endpoint body.

The prohibited shape (testing the error as a value object, where no production code is exercised) versus the correct
shape (driving the call path that answers the error) lives in `references/error-raising-behavior.md`.

If no public method has a call path that answers the error, the error is dead code. Remove it instead of writing a test
against its constructor.
