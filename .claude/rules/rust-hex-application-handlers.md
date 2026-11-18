---
description: Rules for thin handlers in application/handlers. Existence and uniqueness handling, the one aggregate call per path, the thin structure and its prohibitions, and handler collaborators. The canonical handler bodies live in the rust-creating-application skill.
paths:
    - "src/application/handlers/**/*.rs"
---
<!-- Vendored from payment (.claude/rules/php-hex-application-handlers.md) on 2026-11-18. Adapted to this project: `handle()` mapped to the domain-verb method of the inbound port trait, try/catch and loops mapped to their Rust forms, the calculation handler `CalculatingCongestionTaxHandler` placed in the thin structure, and the facts-drive-the-flow section dropped because this service has no domain events, outbox, or messaging. -->

# Handlers

Handlers orchestrate one use case as thin orchestration over the domain, translating a command into calls on the domain
and the outbound ports. The teaching that a handler contains no business rules (those live on aggregates and domain
services) is in the rust-applying-hexagonal-architecture skill. This rule keeps the structural invariants: the
no-database-pre-check rule, the thin Shape A or B structural prohibitions, and the one-trigger contract.

A handler is a struct named `{InboundPort}Handler` that holds its outbound ports as generic fields and implements the
inbound port trait (`CalculatingCongestionTaxHandler<C: TaxRulesCatalog>` implements `CalculatingCongestionTax`). The
port method carries the domain verb (`calculate`), never a generic `handle`.

## Pre-output checklist

Verify before producing a handler.

1. The handler never pre-checks existence or uniqueness before a mutation. The schema enforces the invariant and the
   adapter translates the violation. No `exists(...)` style port. See § No database pre-check.
2. A Shape A or B body contains no `match` or `if let` on an `Err` used for control flow, no nested `if` (one guard that
   returns the not-found aside), no `for`, `while`, or `loop`, and no business calculation. When it needs any of these,
   escalate to Shape C. See § Thin handler structure.
3. The handler invokes no other handler. Shared logic is a domain service or an aggregate method, never a second
   handler. See § No handler-to-handler calls.
4. The handler calls the aggregate at most once per execution path, and never reads aggregate state to decide whether to
   call a transition. See § One aggregate trigger per handler.

## One aggregate trigger per handler

Four premises, each a hard invariant:

1. A handler triggers the aggregate at most once: one call per execution path (`rules.tax_for(vehicle, &passages)`).
   The state guard lives inside that method, never before it. Guard-then-apply (reading aggregate state in the handler
   to decide whether to call a transition) is prohibited: it duplicates the state rule outside the domain and opens a
   check-then-act race.
2. The domain returns no flow signal. A transition returns the new value or a domain error. A `bool` telling the handler
   what to do next moves a business decision out of the model.
3. Repeated or out-of-order input is absorbed by the aggregate itself (owned by `rust-hex-application-domain` §
   Aggregate method contract), never by handler branching.
4. When a use case seems to need a second aggregate trigger, the second trigger is a separate use case behind its own
   inbound port.

## No database pre-check

A handler never asks the database whether something exists, is unique, or is already taken before acting. The pre-check
has two problems, whether the question is positive (uniqueness) or negative (existence):

1. **Race condition.** Between the pre-check and the write, another request can change the answer. The handler thinks
   it is safe, the database disagrees.
2. **Domain leakage.** The handler ends up with a prohibited verb (`assert`, `ensure`, `verify`, `validate`).

The invariant is declared in the schema. The driven adapter catches the constraint violation and translates it into the
matching error. The handler stays pure and just calls the port. This applies to every "exists, already taken, already
booked, missing reference" invariant on a write. The handler never asks, the database answers. An `exists(...)` style
port is prohibited.

A lookup is not a pre-check. A use case that needs an aggregate loads it through the port and turns an absent result
into the not-found (`let Some(rules) = self.catalog.find(&city).await? else { ... }` returning `TaxRulesNotFound`),
which is the one guard Shape B admits.

CONTEXTUAL on the invariant being enforceable as a schema constraint. This no-pre-check rule presupposes that the
persistence layer translates a constraint (a unique index, a foreign key, a not-null) into the matching error. When an
invariant cannot be expressed as a constraint, the handler still never queries the database to pre-check: model the
rule in the aggregate constructor or a domain service and let the domain raise the error.

## Thin handler structure

A handler matches one shape: A (single aggregate, create), B (single aggregate, one call on the aggregate it loads), or
C (multi-aggregate or compensating). A and B are the thin default. `CalculatingCongestionTaxHandler` is Shape B: it
promotes the command fields, loads `TaxRules` through `TaxRulesCatalog`, guards the not-found, calls `tax_for` once, and
maps the assessment into the `CongestionTax` read model. The literal body for each shape is in the
rust-creating-application skill.

The structural distinction below is the residue this rule owns. A Shape A or B body MUST NOT contain:

1. a `match`, `if let`, or `or_else` on an `Err` used for control flow (propagate with `?` and let the `#[from]`
   conversions of the failure enum carry each error),
2. a nested `if` (a single guard returning the not-found, as above, is allowed in Shape B),
3. a `for`, `while`, or `loop`,
4. a calculation or conditional on business state, a state transition, or value-object construction that duplicates
   the aggregate constructor.

An iterator chain that only promotes command primitives (`command.passages.iter().map(...)` collected into
`Passages`) or maps the aggregate's answer into the read model is promotion and mapping, not a loop on business state.

When the use case genuinely needs branching, an error fallback, or multi-aggregate coordination, escalate to Shape C,
with the orchestration directly in the port method. A cross-aggregate rule that has no input or output is a domain
service the Shape C handler invokes, never logic inlined in the handler.

## No handler-to-handler calls

A handler orchestrates one use case and never invokes another handler, and no driving adapter sequences two inbound
ports for a single request. Depending on a second handler couples two use cases and opens a cycle, the exact hazard a
general in-process command bus would hide (this service has none, by design). Shared logic that two use cases need is a
domain service or an aggregate method, called by each handler over its own ports, never one handler reaching through
another.

## Handler collaborators

A type used only by handlers that is not itself a handler (a policy-like collaborator) lives in
`application/handlers/<aggregate>/` next to the handlers that use it (in the `handlers/` root when the layout is flat,
as it is today), never in a technical-role module. It implements no inbound port and is never reached through one.

## Delegated to owners

- The body for Shape A, B, and C, the port and aggregate call-path parameter rules, and the prohibited, correct, and
  branching examples: The shapes and how-to are in the rust-creating-application skill.
- The teaching that handlers carry no business rules and the application-as-orchestration framing: owned by the
  rust-applying-hexagonal-architecture skill.
