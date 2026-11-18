---
name: rust-creating-driven
description: Builds a Driven (outbound) adapter via the repository sub-flow, persisting or reading an aggregate behind its outbound port, plus the decorators over that port. Use when reading or persisting an aggregate, writing persistence SQL, or caching a port.
---
<!-- Vendored from payment (.claude/skills/php-creating-driven/SKILL.md) on 2026-11-18. Adapted to this project: the gateway and outbox sub-flows and their references were dropped (no outbound HTTP and no messaging here), the repository sub-flow became sqlx over PostgreSQL in src/driven/postgres/, and the response cache decorator is worked from src/driven/cache/. -->

# Add driven adapter

Builds a secondary (outbound) adapter in `src/driven/<technology>/`: persistence, or a decorator over an outbound port.
The adapter always implements an outbound port and translates the foreign world into the ubiquitous language at the
boundary. This skill routes to the repository sub-flow and carries the canonical shapes that the driven rules used to
inline.

Build only the adapter the request needs, behind the port that already exists. Start with the simplest shape that works
(the adapter itself before a cache decorator) and add a layer only when a real concern actually arrives. The outbound
port comes from `rust-creating-application`, never invented here, and a cross-cutting concern lives outside the adapter
body: a response cache is a decorator that implements the same port over the adapter, as
`CachedTaxRulesCatalog` does over `PostgresTaxRulesCatalog`.

This project has no outbound HTTP integration and no messaging, so the gateway and outbox sub-flows of the owner's
services are not carried here. A request that needs one is a new decision for the owner, not a gap this skill fills.

## When to use

- Read or persist an aggregate, add a database adapter, write persistence SQL, decode a stored row.
- Add a decorator over an outbound port (a response cache, with the clock it needs behind its own trait).

## When NOT to use

This skill owns the outbound edge only: the adapter that implements an already-defined port and translates the foreign
world (a database row, a stored document) into the domain. It does not define the contract it implements, and it does
not own the inbound side. When the work sits on the other side of the port, reach for the sibling.

- Define the outbound port itself: it is born in `rust-creating-application`.
- Create the service HTTP entry (endpoint, request, response): use `rust-creating-driver`.
- Build a read-side adapter (query trait, read model, read endpoint): use `rust-creating-query`, which owns the read
  stack and shares the query-constant shape with this skill.
- Change schema (new table, column, index, seed): use `creating-migration`.

## Rules applied

Load and follow, in this priority. The first rule governs the layer. The middle rules are specific to persistence. The
last one applies across the layer.

- `rust-hex-driven`: general contract of the outbound adapter, the `{Technology}{Capability}` naming, and the
  anti-corruption layer at the boundary.
- `rust-hex-driven-repository`: the database adapter, its query constants, the decoded shape, the row mapping, and the
  constraint-violation to domain-error translation.
- `rust-hex-sql-queries`: bind parameters (no `format!` or interpolation), a mandatory WHERE on DELETE and UPDATE.
- `rust-testing-integration`: test against the real PostgreSQL test database.

## Repository (persistence) sub-flow

Target: the aggregate to read or persist, named in the request (e.g. `TaxRules`).

### Assembly order

1. Confirm the outbound port the adapter implements (comes from `rust-creating-application`).
2. Decoded shape: the stored row or document decoded into domain value objects, rebuilding the aggregate through its
   validating constructor. This project decodes one JSON document per city in `src/driven/postgres/tax_rules_document/`,
   a `Stored*` struct per raw shape and a `Decoded*` newtype per domain type, each `Decoded*` built with
   `#[serde(try_from = ...)]` over the domain constructor.
3. Query constants: the SQL per the sql-queries rules, as `const` raw string literals at the top of the adapter file.
   Start from `references/query-constants.md` and, for a write, `references/write-sql.md`.
4. Adapter implementing the port, named `{Technology}{Capability}` (`PostgresTaxRulesCatalog`), taking the `PgPool` in
   `new`, with `find` answering `Option` and every `sqlx::Error` mapped to the port's error.
5. Wiring in the composition root: `src/main.rs` builds the adapter over the pool and hands it to the handler, wrapped
   in any decorator the port has.
6. Integration test against the migrated database, via `rust-writing-tests`.

### Completeness gate

- [ ] Adapter implements the outbound port, with `find` answering `Option`.
- [ ] Every infrastructure failure maps to the port's error (`TaxRulesUnavailable::new`), never leaks a `sqlx::Error`
      through the port.
- [ ] `save` maps a violated constraint to its domain error per `rust-hex-driven-repository` (a unique violation, read
      with `is_unique_violation()` on the `sqlx::Error::Database` payload, to the domain error, a not-null or check
      violation to the not-supported error), and maps the failure to the port's error when no mapping applies.
- [ ] The decoded shape rebuilds the aggregate through its validating constructor, and stored content that breaks an
      invariant fails the read with the port's error instead of producing an aggregate the domain refuses.
- [ ] Every UPDATE and DELETE carries a WHERE clause, per the sql-queries rules. That part is unconditional. Every write
      over a tenant-scoped table also carries that table's tenant discriminator (in the INSERT column list and in the
      WHERE), the multi-tenant isolation invariant, not an optional filter. Look at the table you are writing to, it
      either carries a tenant scope column or it does not, and where the schema alone does not settle which tables are
      tenant-scoped, the spec at `<spec-root>` does. In a single-tenant service, or on a table with no tenant
      discriminator, there is no scope column to carry and this half of the item is skipped rather than failed.
- [ ] A decorator implements the same port over the adapter it wraps, never a method of the adapter itself.
- [ ] Runnable check: `make tests` passes and `make review` is clean.

## Shapes and how-to (read when scaffolding)

The literal shapes that the driven rules used to inline live here, surfaced only when this skill runs. Each shape below
is named by its path under `rust-creating-driven`.

### Design patterns

- For selecting or applying a GoF pattern, use the `rust-applying-design-patterns` skill (one cross-cutting catalog,
  usable in any layer). The naming invariant is enforced by `rust-code-style` (§ Design-pattern names).

### Repository shapes

The query-constant shape is shared with the read side, so it lives here as the single owner.

- `references/query-constants.md`: the `const` raw string literals holding the SQL (ordered by name length ascending,
  the SQL body flush with the literal, bind parameters `$1`, `$2`).
- `references/write-sql.md`: the write SQL shapes (INSERT, UPDATE, DELETE), each carrying the table's tenant
  discriminator when the table is tenant-scoped.

### Read-side cross-reference

For the read-side SQL shapes, see the `rust-creating-query` skill (`references/select.md`). Those stay in
`rust-creating-query`.

## Does not do

- Does not define the outbound port: that belongs to the application layer (`rust-creating-application`).
- Does not build the driver HTTP entry (endpoint, request, response): use `rust-creating-driver`.
- Does not build the read-side slice (query trait, read model, read endpoint): use `rust-creating-query`.
- Does not write the tests itself: it delegates each integration test to `rust-writing-tests`.
