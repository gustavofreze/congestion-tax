<!-- Vendored from payment (.claude/skills/php-creating-application/references/aggregate-walkthrough.md) on 2026-11-18. Adapted to this project: the eventual construction and the event emission were dropped because this project has no domain events, and the worked example is TaxRules. -->

# Aggregate walkthrough (worked examples)

Read this while modeling an aggregate. It explains the worked examples carried as templates. The invariants stay in
`rust-hex-application-domain`. This file is narrative how-to, so it is NOT path-injected.

## Immutable aggregate construction

`references/aggregate-root.md` shows an immutable aggregate, the only kind this project has. Its constructor does three
things:

1. Receives the child entities already built (`TaxPeriod`), each one having validated its own invariants in its own
   `new`.
2. Checks the invariant that spans the children (no two tax periods overlap) and answers the domain error that names it
   (`TaxPeriodsOverlap`) instead of an instance that breaks it.
3. Assembles the instance with private fields, so no caller can reach a state the constructor refused. The aggregate
   never reads the clock: an instant it needs is an argument, read once by the handler from an outbound clock port.

Reconstitution goes through the same constructor. The driven adapter decodes the stored row into value objects and
calls `TaxRules::new`, so stored content that breaks an invariant fails the read with the port's error
(`TaxRulesUnavailable`) instead of producing an aggregate the domain would have refused.

The behavior of the aggregate answers a new value and changes nothing: `tax_for` answers a `TaxAssessment`. A change to
the aggregate itself follows the same rule and returns a new instance.

## State transitions

`references/state-transition.md` shows the absorb-guard-move shape for an aggregate that has a lifecycle. Each
transition consumes the aggregate, absorbs a repeat of its own resulting state, guards itself by asking the status enum
a predicate (`cannot_be_paid()`), and answers the aggregate in the new status. It receives the instant it happened as an
argument. This project's `TaxRules` has no lifecycle, so it has no transition, and the shape applies only when an
aggregate with statuses enters.
