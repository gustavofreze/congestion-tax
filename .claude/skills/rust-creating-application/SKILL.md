---
name: rust-creating-application
description: Models the Application inner hexagon (aggregate, value object, enum, domain error, domain service) and the write use case (command, port, handler). Use when adding any of these in a Rust service.
---
<!-- Vendored from payment (.claude/skills/php-creating-application/SKILL.md) on 2026-11-18. Adapted to this project: PHP classes, interfaces, and exceptions became Rust structs, traits, and thiserror errors under src/application/, the domain events, typed identifier, and php-reading-spec routes were dropped, and the inbound port method is named after the intention instead of handle(). -->

# Add application

Builds the Application layer inner hexagon. Routes between the domain artifacts and the write use case, carries the
canonical shapes and the worked examples that used to live inside the rules, and delegates outward to the adapter skills
when persistence or HTTP enter.

Target: the aggregate, concept, or use case named in the request (e.g. `TaxRules`, `CalculateCongestionTax`).

Build only the artifact the request names. Start with the simplest shape that satisfies the rule (an enum before
per-state types, a thin handler before a domain service) and add structure when the behavior demands it, not before.
Do not pre-build the outbound adapter, the endpoint, or domain not yet needed: those are sibling skills, reached only
when the request actually crosses into them.

The write side is the command side of CQRS. A use case on it takes a command and may persist nothing at all: this
project's only use case, `CalculateCongestionTax`, reads the tax rules through an outbound port and computes the tax.

## When to use

- Create a domain artifact: aggregate, value object, enum, domain error, or domain service.
- Introduce a write intention (calculate, create, refund, and similar) wiring a command, an inbound port, a handler,
  and an outbound port.
- The trigger is not HTTP (CLI, job), or the endpoint and the driven adapter already exist.

## When NOT to use

This skill owns the inner hexagon only (domain plus the write use-case ring). It stops at the outbound port. The moment
a real adapter, an HTTP entry, or a schema change is what the request needs, the work belongs to a sibling. Reach for
the sibling instead of widening this skill.

- A full vertical slice from HTTP request to the database: use `rust-implementing-features`, which orchestrates this
  skill and the adapters in dependency order.
- Only the write HTTP entry (endpoint, request, response): use `rust-creating-driver`.
- Read or persist the aggregate, or build any adapter behind a port: use `rust-creating-driven`.
- A read-side slice (a query, a read endpoint): use `rust-creating-query`.
- Selecting or applying a GoF pattern in any layer: use `rust-applying-design-patterns`.

## Sub-flows

Pick the path by what the request introduces:

- **Domain artifact** (an aggregate, value object, enum, domain error, or domain service) with no surrounding write
  flow. Route to § Domain artifact.
- **Use case** (a write intention that wires a command, an inbound port, a handler, and an outbound port). Route to §
  Use case. The use-case path delegates to § Domain artifact for any new domain it needs, never hand-rolling domain
  inside the handler.

When the request is both (a new write flow that also introduces new domain), run § Domain artifact first for the new
concept, then § Use case to wire it.

## Rules applied

Load and follow, in this priority (domain first, then the use-case ring, then patterns and tests):

- `rust-hex-application-domain`: aggregate, constructors, transitions, value objects, enums, errors.
- `rust-hex-application-handlers`: Handler shape (A, B or C) and the single trait method.
- `rust-testing-unit`: what to unit-test and what to cover through the aggregate.

The command, inbound port, outbound port, and domain-service shapes (the command struct of public primitive fields,
the gerund inbound port trait with one method, the plural-noun outbound port trait answering `Option` where the row
may be absent, a domain service only when the rule crosses aggregates) are carried by this skill's references and
completeness gate, the naming by `rust-code-style`, the CQRS fact by `rust-architecture`.

## Domain artifact

Models the domain artifacts: the aggregate or concept named in the request.

### Assembly order

1. Read the contract for the aggregate boundary, invariants, and transitions: the spec at `<spec-root>` where it is
   set, and otherwise the domain vocabulary in `AGENTS.md`, `openapi.yaml`, and the docs pages.
2. Value objects and enums under `src/application/domain/models/<aggregate>/`, validating in `new` (or `FromStr` for
   a value parsed from text) and answering `Result<Self, <Error>>`.
3. Domain errors under `src/application/domain/errors/`, one file per violated invariant. Start from
   `references/domain-error.md`.
4. Aggregate: constructor, behavior, and transitions. Start from `references/aggregate-root.md` and
   `references/state-transition.md`.
5. Domain service, only if the rule crosses aggregates.
6. Test of the behavior through the aggregate, via `rust-writing-tests`.

### Completeness gate

- [ ] The aggregate drives the behavior, with no anemic setter and no public field.
- [ ] A value object validates in its constructor, an invalid transition answers a domain error, and nothing panics.
- [ ] The aggregate never reads the clock: a constructor or transition receives its instant as an argument.
- [ ] Domain service only where the rule does not fit in an aggregate.
- [ ] The domain imports only `std`, itself, `jiff`, and `thiserror` (`make review` runs the domain-dependency check).
- [ ] Test covers action and invariant, never accessors.
- [ ] Runnable check: `make tests` passes and `make review` is clean.

## Use case

Builds the inner hexagon of a write flow: the use case named in the request.

### Assembly order

1. Read the contract before generating: the spec at `<spec-root>` where it is set, and otherwise `openapi.yaml`, the
   docs pages, and the endpoint tests. Fix here the Handler shape, whether a new outbound port is needed, and whether a
   domain service is needed. Do not guess. If the contract does not decide, ask before generating.
2. Required domain, via § Domain artifact above.
3. Command under `src/application/commands/` (write intention, public primitive fields), then the inbound port trait
   under `src/application/ports/inbound/` (gerund, one method named after the intention), then its failure enum and
   its not-found error beside it, then an outbound port trait under `src/application/ports/outbound/` with its error if
   a new collaborator enters. The command never reuses a domain model, value object, enum, or wrapper: primitives only.
   It carries every field of its use case, `Option` when optional: a handler never hardcodes `None` for a field the
   transport can supply, and a domain constructor never multiplies variants to encode call-site absences. For a
   PATCH-style partial update, the command carries `Option` primitives plus a `provided_fields` list naming the use
   case's changes, and the handler dispatches each provided change to an intention-revealing aggregate method
   (`with_priority`, `with_country`). No layer diffs state: an absent field is simply never dispatched, a provided
   `None` clears.
4. Handler under `src/application/handlers/` implementing the inbound port, in the shape decided in step 1. Start from
   the matching `references/handler-shape-*.md`.
5. Integration test of the handler, delegated to `rust-writing-tests`, never a unit test (`rust-testing-unit`). It runs
   once the adapter behind each outbound port exists (`rust-creating-driven`).

### Completeness gate

- [ ] Inbound port trait in the gerund (`CalculatingCongestionTax`) with a single method named after the intention
      (`calculate`), returning `impl Future<Output = Result<..., <Failure>>> + Send`, with a doc comment stating the
      contract.
- [ ] The inbound port's failure is one enum beside it (`CalculationFailure`) with a `#[from]` variant per error the
      driver must map, so the handler propagates with `?` and the driver matches exhaustively.
- [ ] Command is a struct of public primitive fields (`String`, integers, `Vec` of them, `Option` of them), never a
      value object or a domain type.
- [ ] Outbound port trait is a plural noun, answers `Option` where the row may be absent (the Handler returns the
      not-found), and declares its error beside it in `ports/outbound/`.
- [ ] Handler is generic over the outbound ports it takes in `new`, implements the one trait method, and holds no
      business rule that belongs to the aggregate.
- [ ] Handler promotes the command primitives to value objects before calling the port, so a rejected input never
      reaches it.
- [ ] Handler reads the instant from an outbound clock port it takes through `new`, then passes it in, whenever the
      domain needs one.
- [ ] Domain came from § Domain artifact, not hand-rolled here.
- [ ] Test covers action and invariant, never accessors.
- [ ] Runnable check: `make tests` passes (100 percent of lines, every mutant caught) and `make review` is clean.

## Shapes and how-to (read when scaffolding)

The literal shapes that the rules used to inline live here, surfaced only when this skill runs.

### Design patterns

- For selecting or applying a GoF pattern, use the `rust-applying-design-patterns` skill (one cross-cutting catalog,
  usable in any layer). The naming invariant is enforced by `rust-code-style` (§ Design-pattern names).

### Domain artifact shapes

- `references/aggregate-root.md`: the immutable aggregate worked example, `TaxRules::new` validating its invariant
  over the child entities it receives.
- `references/state-transition.md`: the absorb-guard-move shape of a transition that consumes the aggregate and
  answers the next one.
- `references/typed-collection.md`: the `Passages` newtype over a `Vec`, with `FromIterator`.
- `references/domain-error.md`: the four error shapes (prohibited ad hoc failure, dedicated type, application
  `<Aggregate>NotFound`, input-rejection carrying the raw value).
- `references/aggregate-walkthrough.md`: the narrative behind the worked examples, immutable construction and the
  transition shape.

### Use case shapes

- `references/handler-shape-a-create.md`: Shape A, single aggregate create, with the prohibited build-up-front body kept
  as a paired anti-pattern.
- `references/handler-shape-b-transition.md`: Shape B, single aggregate load then apply, with the one allowed not-found
  guard, worked from `CalculatingCongestionTaxHandler`.
- `references/handler-shape-c-orchestration.md`: Shape C, multi-aggregate or compensating body with the one allowed
  fallback `match` on a port outcome.
- `references/handler-branching-antipattern.md`: the create-or-mutate lookup branch, the prohibited body next to the
  constructor-decides correction.

## Does not do

- Does not build the write HTTP entry (endpoint, request, response): that is `rust-creating-driver`.
- Does not build any driven adapter (persistence or cache behind a port): that is `rust-creating-driven`.
- Does not build the read-side slice (a query, a read endpoint): that is `rust-creating-query`.
- Does not author the tests: it delegates coverage to `rust-writing-tests`.
