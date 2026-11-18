---
name: rust-creating-query
description: Builds the read-side slice (query trait, read model, database adapter, query constants, read endpoint, response) and owns the read error mapping. Use when creating a read endpoint, a find, a query, or a read-side adapter.
---
<!-- Vendored from payment (.claude/skills/php-creating-query/SKILL.md) on 2026-11-18. Adapted to this project: the PHP slice became a module under src/query/<context>/ worked from the health slice, the SQL is PostgreSQL through sqlx, and the keyset, offset, masking, and MySQL collation references were dropped (no pagination, no sensitive read data, no MySQL here). -->

# Add query

Builds the read-side slice. Routes to the query rules, and carries the canonical read-side shapes that used to live
inside the rules.

Target: the query named in the request (e.g. `ReadinessCheck`).

Build the thinnest read that satisfies the contract. Start from a single-table SELECT, and add a JOIN or a JSON
projection only when the contract demands it. This project pages no collection, so a list that grows unbounded is a
contract decision for the owner before it is a query.

## When to use

- Create a read endpoint, a find, or a read-side adapter under `src/query/<context>/`.
- Add or extend the read error mapping of a slice (its `http/` module).

## When NOT to use

- The write side (command, handler): use `rust-creating-application`.
- A full slice with writes: use `rust-implementing-features`.
- A write HTTP endpoint or route: use `rust-creating-driver`.
- Read or persist an aggregate behind an outbound port (the driven side): use `rust-creating-driven`.
- Change schema or add a migration: use `creating-migration`.

## Slice layout

A slice is one module per bounded read under `src/query/<context>/`, one type per file, worked here from
`src/query/health/`:

- The query trait (`dependency_probe.rs`, `DependencyProbe`), the read port, with a doc comment stating its contract.
- The read model (`dependency_status.rs`, `DependencyStatus`), plain public fields of primitives.
- The database adapter under `database/` (`postgres_database_probe.rs`, `PostgresDatabaseProbe`), implementing the
  query trait over the `PgPool` with its query constants at the top of the file.
- The read endpoints and their responses under `http/` (`readiness_endpoint.rs`, `dependency_status_response.rs`), the
  responses `#[derive(Serialize)]` with private fields.

`src/query/<context>.rs` re-exports the trait, the read model, and the adapter `pub`, and the endpoint functions
`pub(crate)`, and `src/routes.rs` mounts the endpoints with the adapter as state.

## Rules applied

Load and follow, in this priority:

- `rust-hex-query`: slice shape (query trait, read model, database adapter, endpoint, response, row mapping) and the
  read error mapping.
- `rust-hex-sql-queries`: bind parameters (no value by `format!` or interpolation, structural fragments only).
- the `writing-documentation` skill: the read endpoint enters the docs page and `openapi.yaml`. `web-documentation`
  owns the cross-file sync, and the prose style follows the global default in `AGENTS.md` § Global defaults.
- `rust-testing-integration`: test of the read endpoint adapter and of the database adapter, both against the real
  PostgreSQL test database (`rust-testing` § Test paths and modules).

## Shapes and how-to (read when scaffolding)

The literal read-side shapes that the rules used to inline live here, surfaced only when this skill runs:

- For GoF pattern selection, use the `rust-applying-design-patterns` skill (cross-cutting, all families). The naming
  invariant is in `rust-code-style` (§ Design-pattern names).
- `references/read-model.md`: the read model of primitives plus the `Serialize` response that is the single owner of
  the wire envelope.
- `references/mapper.md`: the row mapping in the database adapter, the only place that reads a raw row or a raw
  outcome.
- `references/select.md`: the read SQL shapes (SELECT single table, SELECT with JOIN, multi-column ORDER BY, and the
  JSON document projection) in the PostgreSQL spelling.

## Assembly order

1. Read the contract if the query carries a business rule: the spec at `<spec-root>` where it is set, and otherwise
   `openapi.yaml` and the docs page.
2. Query trait (read port) and read model, then the response with its `From<ReadModel>`. Start the read model, the
   response, and the row mapping from `references/read-model.md` and `references/mapper.md`.
3. Database adapter with its query constants, the SQL per the sql-queries rules. Start the SQL from
   `references/select.md`.
4. Read endpoint, generic over the query trait it takes as `State<Arc<D>>`, with input parsing.
5. Read error mapping in the slice's `http/` module, if new. Map the slice's own failures there per `rust-hex-query` §
   Read-write isolation, never through `src/driver/http/error_response.rs`.
6. Route in `src/routes.rs`.
7. Tests: integration of the endpoint and of the adapter, via `rust-writing-tests`.

## Completeness gate

- [ ] Query trait named per the query rules, without leaking the persistence type.
- [ ] Every read over a tenant-scoped table is filtered by that table's tenant discriminator (the multi-tenant isolation
      invariant, not an optional filter). Look at the table you are querying, it either carries a tenant scope column or
      it does not, and where the schema alone does not settle which tables are tenant-scoped, the spec at `<spec-root>`
      does. In a single-tenant service, or on a table with no tenant discriminator, there is no tenant to isolate and
      this item is skipped rather than failed.
- [ ] Read endpoint parses input and translates errors, exhaustively.
- [ ] Read error mapping produced in the slice's own `http/` module, with no import from `src/driver/`, per
      `rust-hex-query`.
- [ ] The slice imports nothing from `crate::application` handlers or `crate::driven`.
- [ ] Database adapter covered by an integration test.
- [ ] Runnable check: `make tests` passes and `make review` is clean.

## Does not do

- Does not build the write side: the command, the handler, or a write endpoint or route.
- Does not model the application inner hexagon or read an aggregate on the driven side.
- Does not change schema or add a migration.
- Does not author the read tests by hand. The read slice delegates them to `rust-writing-tests`.
