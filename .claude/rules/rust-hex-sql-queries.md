---
description: SQL lives only in a module-level const raw string in the adapter that runs it, and the row decodes into domain types through the domain constructors. The structural residue only, no value reaching the SQL by format! or concatenation (bind parameters), DELETE and UPDATE always carry a WHERE, and the connection negotiates a full Unicode encoding. Formatting, aliasing, and worked examples live in the rust-creating-driven and rust-creating-query skills. Every query over a tenant-scoped table filters by that table's tenant discriminator, and a table without one, like a single-tenant service, skips that gate rather than failing it.
paths:
    - "src/driven/postgres/**/*.rs"
    - "src/query/**/database/**/*.rs"
---
<!-- Vendored from payment (.claude/rules/php-hex-sql-queries.md) on 2026-11-18. Adapted to this project: the final readonly Queries class became a module-level const raw string in the adapter file (FIND_TAX_RULES), sprintf became format!, Doctrine bind parameters became sqlx $n placeholders with bind(), the MySQL utf8mb4 charset and collation material became the UTF8 client_encoding sqlx negotiates, and a Row decoding section was added for src/driven/postgres/tax_rules_document/. The read-side query builder paragraph was dropped because no slice paginates or filters, and the php-reading-spec references were dropped because the skill is not ported. -->

# SQL queries

All SQL lives exclusively in a module-level `const` in the adapter file that runs it. The placement of that adapter is
owned by `rust-hex-driven-repository` (§ Structure) on the load side and by `rust-hex-query` on the read side.

## Pre-output checklist

1. SQL lives in a `const <QUERY_NAME>: &str = r"...";` at the top of the adapter file, named in `SCREAMING_SNAKE_CASE`
   after what it does (`FIND_TAX_RULES`). A string literal passed inline to `sqlx::query` is prohibited. See § Query
   constants.
2. No value reaches the SQL text through `format!`, `concat!`, or string building. Dynamic values flow through `$n`
   placeholders bound with `.bind()`. See § Query constants.
3. `DELETE` and `UPDATE` always carry a `WHERE` clause. Unconditional writes are prohibited. See § DELETE and UPDATE.
4. The connection negotiates a full Unicode encoding that carries four-byte characters, and a partial encoding is
   prohibited. With `<database-engine>` = `postgresql` that encoding is `UTF8`. See § Connection encoding.
5. Every query over a tenant-scoped table filters by that table's tenant discriminator, the column carrying the tenant
   scope. A table that has no such column, and a single-tenant service, SKIP this gate rather than failing it. See §
   Tenant scoping.
6. The row decodes into domain types through the domain constructors, and the decoding types stay private to the
   adapter's module. See § Row decoding.

## Query constants

The query is a `const` of type `&str` holding a raw string literal (`r"..."`), so its single-quoted SQL literals and
its line breaks need no escape. Dynamic values flow through `$n` placeholders and `.bind()`, never concatenated or
formatted into the SQL text:

```rust
const FIND_TAX_RULES: &str = r"
SELECT json_build_object(...) AS tax_rules_document
FROM tax_rules AS rules
WHERE rules.city = $1
";

sqlx::query_scalar::<_, Json<DecodedTaxRules>>(FIND_TAX_RULES)
    .bind(city.name())
```

The constant ordering, the formatting, the aliasing, the `AS` rules, and the `SELECT` shapes are in the
rust-creating-driven and rust-creating-query skills.

## DELETE and UPDATE

`DELETE` and `UPDATE` always carry a `WHERE` clause. An unconditional delete or update is prohibited.

## Connection encoding

The portable invariant: the connection negotiates a full Unicode encoding once, and every query then traffics it. An
encoding that cannot carry four-byte characters is never negotiated, whatever the engine calls it, because a truncated
emoji or CJK extension character is a data-loss bug and not a display bug. The encoding declared on the database is the
same one, owned by `database-migration` § Column types and engine.

With `<database-engine>` = `postgresql` that encoding is `UTF8`. sqlx sends `client_encoding` = `UTF8` in the startup
message of every PostgreSQL connection, so the `PgConnectOptions` built in the composition root (`src/main.rs`) and in
the test fixtures set no encoding, and no query issues `SET client_encoding` or `SET NAMES` with another value.

## Tenant scoping

Every query over a tenant-scoped table filters by that table's tenant discriminator, and a read or a write that can
cross tenants is a defect, never a convenience. The discriminator is the column that carries the tenant scope on the
table being queried, and whoever writes the query is already looking at that table and can see whether it has one.
Scoping is decided per table, never assumed repository-wide: a discriminator on one table says nothing about the next.
Which tables are tenant-scoped, and any deliberate exception (a uniqueness constraint intentionally global, for
instance), are owned by the spec at `<spec-root>`. Where `<spec-root>` is absent, ask which tables are tenant-scoped
rather than guessing.

A table with no tenant discriminator has no column to scope by, and so does a service that is single-tenant throughout.
Either way this gate is skipped rather than failed.

## Row decoding

`FIND_TAX_RULES` loads the whole aggregate in one row: `json_build_object` and `json_agg` (each with its `ORDER BY`,
and `COALESCE(..., '[]'::json)` for an empty child list) assemble one JSON document per city, and the adapter reads it
as `Json<DecodedTaxRules>`. The document decodes in `src/driven/postgres/tax_rules_document/` through two kinds of
type, one per file:

- `Stored*` types (`StoredTaxRules`, `StoredTaxPeriod`) derive `Deserialize` and mirror the document's fields.
- `Decoded*` newtypes (`DecodedTaxRules`, `DecodedCurrency`) carry the domain value. Each declares
  `#[serde(try_from = "...")]` and builds the value in `TryFrom` through the domain constructor (`TaxRules::new`,
  `Currency::new`), whose error is the `TryFrom::Error`.

A stored value that breaks a domain invariant therefore fails the decoding, which the adapter translates as any other
`sqlx::Error` (`rust-hex-driven-repository` § Failure translation). The decoding types are visible only inside
`src/driven/postgres/` (`pub(super)`, `pub(in crate::driven::postgres)`), and the module re-exports only the root
`DecodedTaxRules` to its adapter.
