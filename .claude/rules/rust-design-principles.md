---
description: Design-principle residue only, the precedence ladder, the binding force (MUST vs SHOULD), and the this-service homes for shared code. SOLID, DRY teaching, Tell-Don't-Ask, complexity, and decision routing live in the rust-applying-design-principles skill.
paths:
    - "src/**/*.rs"
---
<!-- Vendored from payment (.claude/rules/php-design-principles.md) on 2026-11-18. Adapted to this project: the shared-code homes renamed to Rust module paths and reduced to the ones that can exist here (the event, outbound client, service token, and pagination homes dropped), the specification line resolved through `<spec-root>`, and the skill names mapped to their `rust-` counterparts. -->

# Design principles (precedence, binding force, shared-code homes)

This rule keeps the precedence ladder, the binding force that gives the principles teeth, and the this-service homes for
shared code. The per-principle teaching (SOLID, DRY, Tell-Don't-Ask, complexity, decision routing) is in the
rust-applying-design-principles skill. A textbook formulation never overrides an existing rule, and any disagreement is
flagged.

**Binding force.** The Pre-output checklist, Single responsibility, Open-closed, Liskov, Interface segregation,
Dependency inversion, Tell-Don't-Ask, DRY, and Precedence are MUST: a violation is a defect to fix before the change
lands. Complexity as a decision input and Decision routing are SHOULD: they steer the choice between admissible designs,
and a deviation ships only with its reason recorded.

## Pre-output checklist

1. Duplicated business knowledge has exactly one owner. Structural similarity alone is never unified. See § DRY.
2. Shared code lives only in the homes § DRY admits. Never a new technical-role module. See § DRY.
3. When principles collide, § Precedence decides. Novel trade-offs are flagged, not resolved silently. See § Precedence.

## DRY

The DRY principle proper (knowledge over text, the wrong abstraction over duplication, the rule of three) is in the
rust-applying-design-principles skill. Two this-service structural lists stay here, tied to the skeleton and CQRS:

1. **Admitted homes for shared code**, and no others: `application/domain/models/commons/` (value objects shared by
   more than one aggregate), `query/<context>/shared/` (read artifacts shared by slices of one context: the context
   read model and its row mapping), `query/shared/` (cross-context read side), the `driver/http/` root for cross-cutting
   inbound infrastructure (`error_response.rs`, which maps every `CalculationFailure` to its HTTP answer),
   `application/handlers/<aggregate>/` for the collaborators used only by that folder's handlers
   (`rust-hex-application-handlers` § Handler collaborators), and `driven/shared/` for the outbound infrastructure
   abstractions whose role IS the pattern (a connection wrapper, its failure translation), consumed by the driven
   adapters alone. Only the `driver/http/` root exists today, and each other home is created by the change that brings
   its second consumer. Each home is shared only inside its own layer, never across the hexagon: `driven/shared/` is
   never used by `driver/` (the dependency rule keeps every cross-layer coupling on `application/ports/`) and never by
   `query/` (the read side owns its access, per `rust-hex-query` § Read-write isolation). Creating a technical-role
   module (`util`, `helpers`, `common`) to host shared code is prohibited.
2. **Duplication that is correct by design**, never to be unified: models across bounded contexts (one context per
   service), read-side code relative to `driven/` (CQRS, as the separate pool access of
   `query/health/database/postgres_database_probe.rs` shows), the double value-object construction in handlers (the
   handler promotes the command, and the domain constructor still validates), and the per-slice query scaffolding (the
   adapter and its endpoint and response types). This protection covers the scaffolding that genuinely differs by
   slice, not the read model or row mapping the slices of a context share: those have a single owner under
   `query/<context>/shared/` per item 1.

## Precedence

When two principles point in opposite directions, the first applicable line wins:

1. Specification semantics over any rule. The spec lives at `<spec-root>`, and with `<spec-root>` unset the contract
   facts in `openapi.yaml` and the `docs/` pages take that place.
2. An existing rule in `.claude/rules/` over a textbook principle.
3. Context boundary over DRY: never share to avoid duplicating across contexts.
4. Explicit duplication over a wrong abstraction.
5. Composition over inheritance.
6. No preventive abstraction: a trait, a helper, or a method exists only when a present consumer or a port boundary
   requires it. Methods are on demand, never preventive.

When no line above resolves the collision, the trade-off is novel: choose the option that keeps the change reversible
and adds no preventive abstraction, record the decision in the change summary, and proceed without blocking on it.

## Delegated to owners

- Single responsibility, Open-closed (composition form), Liskov (contract form), Interface segregation (port
  granularity), Tell-Don't-Ask, the general DRY principle, complexity as a decision input, and the decision-routing
  table: The shapes and how-to are in the rust-applying-design-principles skill.
