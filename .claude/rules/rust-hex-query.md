---
description: Read side (CQRS). The structural residue only, read code lives at the narrowest scope containing all its consumers, the read pipeline returns a typed read model, and read-write isolation. The slice shapes and read-pipeline templates live in the rust-creating-query skill.
paths:
    - "src/query/**/*.rs"
---
<!-- Vendored from payment (.claude/rules/php-hex-query.md) on 2026-11-18. Adapted to this project: the bounded-context level was dropped because the service has one context and no layer subdivides by it, so src/query/ holds slices directly (the health slice with its DependencyProbe port, DependencyStatus read model, database/ adapter, and http/ module), the Shared/ homes became a module named after what it holds, and the Page, pagination, and toArray/toResponse material was dropped because no slice returns a collection. -->

# Query (read side)

The `src/query/` layer is the segregated read side. Each read use case is a self-contained vertical slice: the health
slice in `src/query/health/` and the tax rules slice in `src/query/tax_rules/` (`TaxRulesFinding`, its
`PostgresTaxRulesFinding`, and the `find_tax_rules_by_city` endpoint, which maps its own not-found and unavailable
outcomes). This rule keeps three structural invariants: placement by consumer scope, the read-pipeline boundary, and
read-write isolation.

## Pre-output checklist

1. Read code lives at the narrowest scope containing all its consumers: one slice in `src/query/<slice>/`, and code two
   or more slices need in a module named after what it holds beside them. Nothing cross-cutting lives in
   `src/query.rs`, which only declares the slices. See § Structure.
2. The adapter returns the slice's typed read model, never a raw row. The endpoint maps that read model into its
   response and serializes it. See § The read pipeline.
3. Nothing from `src/query/` is used by `application`, `driver`, or `driven`, and no outbound port or driven adapter is
   reused for reads. See § Read-write isolation.

## Structure

The service has one bounded context and no layer subdivides by it, so `src/query/` subdivides by slice directly. Inside
a slice, code lives at the narrowest scope that contains all of its consumers:

1. **`src/query/<slice>/`** holds the slice's read port, a trait with doc comments (`dependency_probe.rs`), and its read
   model (`dependency_status.rs`) at the slice root, the outbound database adapter under `database/`
   (`postgres_database_probe.rs`), and the inbound HTTP adapter under `http/` (the `liveness` and `readiness` endpoints
   and their responses). The slice module (`src/query/health.rs`) re-exports with `pub` what the composition
   root and the tests name (the port, the read model, the adapter) and with `pub(crate)` the endpoints `src/routes.rs`
   mounts.
2. **Code two or more slices need** moves to a module named after what it holds, beside those slices, the moment a
   second slice needs it, never copied per slice. None exists today.

Placement follows consumer scope. Cross-cutting query infrastructure NEVER lives in `src/query.rs`.

The per-home artifact layout and the read-pipeline templates are in the rust-creating-query skill. The HTTP wire
contract of the slice is owned by the spec at `<spec-root>` where one exists, and with `<spec-root>` unset by
`openapi.yaml` and `docs/HEALTH.md`.

## The read pipeline

The adapter never returns a raw row. It returns the slice's read model (`DependencyStatus`), built inside the adapter
from what the query answered. The endpoint only maps and serializes: `DependencyStatusResponse::from(DependencyStatus)`
shapes the wire fields, and `ReadinessResponse` wraps them and answers itself with its status code. A row or a driver
error that reaches the endpoint is the boundary leak this rule forbids, so the adapter turns a failed query into the
read model's `detail` text rather than returning the `sqlx::Error`. The read model is strongly typed (`Option<String>`,
`Duration`, `String`), and no untyped row escapes the adapter.

The literal read-model and response shapes are in the rust-creating-query skill.

## Read-write isolation

1. Each slice's adapter lives within `src/query/<slice>/database/` and never resides in `src/driven/`.
   `PostgresDatabaseProbe` lives beside its slice, not beside `PostgresTaxRulesCatalog`, and never imports from a
   sibling slice.
2. Never reuse an outbound port or a driven adapter for reads. The readiness probe runs its own query on its own clone
   of the pool and never calls `TaxRulesCatalog`, even though that adapter reaches the same database.
3. Nothing from `src/query/` is used in `application`, `driver`, or `driven`. Only `src/routes.rs`, which mounts the
   endpoints, and the composition root, which builds the adapter, name it.
4. The read side maps its own outcomes to HTTP in its own `http/` module (`readiness_response.rs`), never through the
   write-side `src/driver/http/error_response.rs`.
