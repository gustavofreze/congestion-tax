---
description: DDD building blocks of the domain layer. Context grouping, the aggregate method contract, the placement of every failure type, and the naming forms of domain errors.
paths:
    - "src/application/domain/**/*.rs"
---
<!-- Vendored from payment (.claude/rules/php-hex-application-domain.md) on 2026-11-18. Adapted to this project: exceptions mapped to thiserror error types and their placement mapped to `domain/errors/` and the two port folders, the state machine contract reduced to the premises that hold without domain events, outbox, or outbound side effects (none exist here), the error-code paragraph pointed at `src/driver/http/error_response.rs`, and examples taken from `src/application/`. -->

# Domain layer (DDD)

This rule keeps the domain layer's always-on structural invariants: the aggregate method contract, the error
layer-placement decision tree, the error naming forms, and the 15-file context-grouping threshold. Every other concern
is delegated to its owner, listed at the end.

## Pre-output checklist

Verify before producing any file under `src/application/domain/`.

1. `domain/models/` threshold: when more than 15 files sit at its root (excluding `commons/`), group by aggregate in the
   same change. See § Context-based grouping.
2. A failure type lands in the right place: domain-invariant violations and value-object self-validation in
   `domain/errors/`, a not-found and the use case's failure enum beside the inbound port in `ports/inbound/`, an
   outbound failure beside its port in `ports/outbound/`. Not-found is application, never domain. See § Error layer
   placement.
3. A type under `domain/` exists only when the domain itself consumes it. Never create a domain model, value object, or
   wrapper to serve another layer's convenience (commands carry primitives, adapters own their own decoded and stored
   shapes, as `driven/postgres/tax_rules_document/` does).
4. An aggregate method follows § Aggregate method contract: the guard lives inside it, only values cross into it, and it
   never reads the clock or an outbound port.
5. A failure type is named for what failed, not by taste: `<Thing>FormatNotValid` for a format parse, `Invalid<Thing>`
   for a broken invariant, `Unsupported<Thing>` for an enum promotion. See § Error naming.

## Context-based grouping

When `domain/models/` (excluding `commons/`) holds more than 15 files at its root, files MUST be grouped into
subfolders named after the aggregate or bounded concept they belong to. This is a hard threshold enforced on every
change. Below it, keep the flat structure and do NOT pre-emptively group. At or above it, grouping is REQUIRED in the
same change that crosses it. Group by aggregate boundary, never by technical type (no `enums/`, `value_objects/`, or
`ids/`). This service already groups: the aggregate root `TaxRules` is declared by `models/tax_rules.rs`, and its value
objects live in `models/tax_rules/`.

## Aggregate method contract

`TaxRules` has no state transition today. It is built valid by `TaxRules::new`, which refuses overlapping periods with
`TaxPeriodsOverlap`, and it answers `tax_for`. Three premises hold for every aggregate method, and they bind the first
transition the day one is added:

1. The guard lives inside the method. An illegal origin state or input raises the domain error from inside the
   aggregate (`tax_for` raises `PassageOutsideTaxPeriods` for a day no period covers), re-application of a transition's
   own resulting state returns silently, and no caller decides either case from outside the aggregate.
2. A transition takes `self` and returns the new value or a domain error (`Result<Self, {Error}>`), never a flow signal
   such as a `bool` telling the handler what to do next. One transition, one outcome.
3. Only values cross into the aggregate: value objects, enums, identifiers, and primitives promoted at the boundary
   (`tax_for(vehicle, &passages)`). No port, no service, no closure enters an aggregate method as an argument or lives
   as a field. A pure domain service is invoked by the handler with the loaded aggregates, never handed to the
   aggregate. The aggregate never reads the clock: a method that needs an instant receives it as an argument, which the
   handler reads from an outbound port.

## Error layer placement

This decision tree is the always-on residue: it decides where a failure type lives. The error type shapes are in the
rust-creating-application skill.

`src/application/domain/errors/` holds ONLY domain-invariant violations and value-object self-validation (the format,
membership, required-field, range, cross-field, and state rules a value object or aggregate enforces on itself). A
domain error is a struct deriving `Debug` and `thiserror::Error`, one per file, with a message in domain terms and no
HTTP status or code. It MAY carry domain context typed with domain types (value objects, identifiers, enums, and the
`jiff` dates and times the domain models time with, as `TaxPeriodsOverlap` carries two `Date` values), never other
primitives.

Every other failure is NOT a domain error and lives beside the port it belongs to:

1. A not-found (a lookup miss) is declared beside the inbound port (`ports/inbound/tax_rules_not_found.rs`).
2. An outbound-port failure (unavailable, conflict) is declared beside its port
   (`ports/outbound/tax_rules_unavailable.rs`), carrying the infrastructure cause as a boxed `#[source]`.
3. The failure of a use case is one enum beside its inbound port (`CalculationFailure`), with one variant per failure
   the use case can return, each a `#[from]` conversion so the handler propagates with `?`.

Database failures, decoding failures, and raw transport failures remain infrastructure: the driven adapter translates
them at its boundary, and they reach the application only as the outbound error the port declares.

A not-found is an application failure, never a domain one: a lookup miss is not a domain invariant. It carries no
context (the caller already holds the identifier it searched for), so it is a unit struct built with no arguments.

One carve-out applies to the primitive prohibition. An input-rejection error, raised when a raw input cannot be promoted
to a value object or enum, MAY carry the raw offending value as a primitive, because no value object exists yet to carry
it. The `Unsupported*` and `*FormatNotValid` errors that hold a `raw: String` (`UnsupportedVehicleType`,
`CurrencyFormatNotValid`) are the conforming shape under this carve-out.

## Error naming

§ Error layer placement decides where a failure type lives. This section decides what it is called. Three forms
partition the domain errors by what actually failed, and a fourth covers the rest. The form is not a style choice, it is
what tells the reader which kind of failure they are looking at.

| What failed                                               | Form                    | Examples                                                    |
|:----------------------------------------------------------|:------------------------|:------------------------------------------------------------|
| A string fails promotion to a value object on format      | `<Thing>FormatNotValid` | `PassageFormatNotValid`, `CurrencyFormatNotValid`           |
| A domain invariant breaks (transition, range, arithmetic) | `Invalid<Thing>`        | `InvalidPaymentStatusTransition`, `InvalidPositive`         |
| A raw value fails promotion to a closed-set enum          | `Unsupported<Thing>`    | `UnsupportedVehicleType`, `UnsupportedMonth`                |
| A business rule refuses a well-formed operation           | `<Thing><Condition>`    | `PassageOutsideTaxPeriods`, `PaymentNotRefundable`          |

`Invalid*` and `*NotValid` are not two spellings of one idea, and they never compete for the same type. `Invalid*` says
an invariant broke, `*NotValid` says a string did not parse. A `< 1` check on an integer is an invariant, so
`InvalidTrialDurationDays` and never `TrialDurationDaysNotValid`. A parse of a string is a format check, so
`PassageFormatNotValid` and never `InvalidPassageFormat`.

The `Format` token is required in the first form. It is what separates a parse failure from an invariant violation when
the subject name alone is ambiguous.

**Renaming a mapped error moves its HTTP error code.** The `code` of each failure is written in
`src/driver/http/error_response.rs` and defaults to the SCREAMING_SNAKE_CASE of the type name
(`PassageFormatNotValid` to `PASSAGE_FORMAT_NOT_VALID`), so a rename reaches that file, `openapi.yaml`, and the `docs/`
page in the same change (synchronization owned by web-documentation). The default is deliberately breakable: a code MAY
diverge when the wire term is a contract fact that differs from the internal name, or when several internal shapes
collapse into one public bucket (`TaxRulesUnavailable` answers `SERVICE_UNAVAILABLE`, a rejected request body answers
`INVALID_REQUEST`). Divergence is a decision, never an oversight.

## Delegated to owners

- Folder skeleton (`models/`, `errors/`, and the `events/` and `services/` folders created on first need): owned by the
  rust-architecture rule (§ Folder skeleton).
- Business nomenclature and DDD building blocks: The shapes and how-to are in the rust-applying-ddd skill.
- Aggregate root, entities, value objects, typed identifiers and collections, and every code skeleton: The shapes and
  how-to are in the rust-creating-application skill.
- Domain behavior: Owned by the spec at `<spec-root>`. With `<spec-root>` unset there is no spec tier: the contract
  facts come from `openapi.yaml` and the `docs/` pages, and domain behavior that neither states is asked for, never
  invented.
