---
description: Database adapters that load an aggregate through its outbound port, and the cache decorator that wraps one. The structural residue only, every database and decoding failure translates to the error declared beside the port at the load boundary, and a missing aggregate is a value rather than an error. The folder layout and the how-to live in rust-architecture and the rust-creating-driven skill.
paths:
    - "src/driven/postgres/**/*.rs"
    - "src/driven/cache/**/*.rs"
---
<!-- Vendored from payment (.claude/rules/php-hex-driven-repository.md) on 2026-11-18. Adapted to this project: the write side became the load side, because tax rules are content seeded by migrations and nothing saves an aggregate, so the constraint translation, the RecordWriter, optimistic concurrency, and the outbox were dropped. The failure translation now covers the load (sqlx::Error, decoding failures included, to TaxRulesUnavailable), and a section on the CachedTaxRulesCatalog decorator was added. The structure owner follows account (rust-architecture) because the skeleton skill is not ported. -->

# Repository (Driven, load side)

A database adapter rehydrates an aggregate. It implements the outbound port defined in
`src/application/ports/outbound/`: `PostgresTaxRulesCatalog` implements `TaxRulesCatalog` and loads the `TaxRules` of
one `City`. This service has no write path. Tax rules are content seeded by Flyway migrations, so no adapter saves an
aggregate, and this rule keeps only the failure translation invariant at the load boundary.

## Pre-output checklist

1. The adapter maps every `sqlx::Error`, a decoding failure included, to the error declared beside its port, and no
   `sqlx` type crosses the port. See § Failure translation.
2. A missing aggregate is `Ok(None)`, never an error. The not-found error belongs to the inbound side
   (`TaxRulesNotFound`), and the handler raises it. See § Failure translation.
3. The stored row decodes into the aggregate through the domain constructors, so stored content that breaks an
   invariant fails the load instead of building an invalid aggregate. See § Rehydration.
4. A cache decorator implements the same port, keeps only found aggregates, and passes a failure through unchanged. See
   § Cache decorator.

## Structure

The folder layout (one folder per technology under `src/driven/`, the adapter file, and the decoding module beside it)
is owned by `rust-architecture` and the rust-creating-driven skill. The `postgres_<port>.rs` file name pattern is owned
by `rust-hex-driven` (§ Adapter kinds).

## Failure translation

Infrastructure failures are translated by the adapter that sees them, never by the handler. The adapter holds the
`PgPool` the composition root built and is the one place a `sqlx::Error` is seen. It maps the failure with the port
error's constructor, which keeps the cause as its `#[source]`:

```rust
let decoded = sqlx::query_scalar::<_, Json<DecodedTaxRules>>(FIND_TAX_RULES)
    .bind(city.name())
    .fetch_optional(&self.pool)
    .await
    .map_err(TaxRulesUnavailable::new)?;

Ok(decoded.map(|Json(DecodedTaxRules(rules))| Arc::new(rules)))
```

A decoding failure of the `Json<...>` column surfaces as a `sqlx::Error` too, so the same `map_err` covers stored
content that breaks a domain invariant. The port error carries no HTTP status and no driver type in its signature
(`TaxRulesUnavailable` boxes the cause as `dyn Error`), and the driver logs that cause without importing `sqlx`.

Absence is not a failure. `fetch_optional` returns `None` when no row matches, the adapter returns `Ok(None)`, and the
handler turns it into `TaxRulesNotFound`, which is declared beside the inbound port.

## Rehydration

The adapter never builds the aggregate field by field from a raw row. The row decodes into `Decoded*` newtypes whose
`TryFrom` calls the domain constructor (`TaxRules::new`, `Currency::new`), so every invariant the domain enforces on
new input also guards stored content. The decoding shapes are owned by `rust-hex-sql-queries` § Row decoding.

## Cache decorator

`CachedTaxRulesCatalog<C, K>` implements `TaxRulesCatalog` around an origin `C: TaxRulesCatalog`, and the composition
root wires it around `PostgresTaxRulesCatalog`. It answers from memory while an entry is younger than its time to live,
measured with the `MonotonicClock` it receives (`K`), and asks the origin otherwise. It keeps found rules only: an
absence and a failure are returned as the origin gave them and never stored, so the next call asks the origin again. The
aggregate is shared as `Arc<TaxRules>`, so a hit hands out the same rules without copying them. The decorator never
reads the system clock directly, which lets a test move time with `MonotonicClockSpy`.

## Read and write boundary

The shapes and how-to are in the rust-creating-driven skill. The read side never reuses this adapter or its port
(`rust-hex-query` § Read-write isolation).
