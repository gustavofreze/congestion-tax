---
description: Driven layer (secondary/outbound adapters). The structural residue only, the adapter-kind file name pattern table. Driver-vs-Driven naming and the anti-corruption boundary live in their owners, named inline.
paths:
    - "src/driven/**/*.rs"
---
<!-- Vendored from payment (.claude/rules/php-hex-driven.md) on 2026-11-18. Adapted to this project: the adapter kinds became the Rust modules under src/driven/ (one folder per technology, PostgresTaxRulesCatalog, the tax_rules_document decoding, the cache decorator and its clock), the Gateway, Client, Publisher, Consumer, and log sink rows, the src/Driven/Shared/ paragraph, and the whole Outbox section were dropped because this service has no outbound HTTP, no messaging, and no outbox. -->

# Driven layer (secondary adapters)

The `src/driven/` layer holds secondary (outbound) adapters, one folder per technology: `postgres/` loads the tax rules
from the database and `cache/` keeps them in memory. This rule keeps the cross-Driven structural residue (the adapter
kind file name pattern). The Driver-vs-Driven naming criterion and the anti-corruption boundary are taught by the skills
named inline below.

## Pre-output checklist

1. Adapter file name matches its kind. A bare `adapter.rs`, and an `Adapter` suffix on the type, are prohibited. See §
   Adapter kinds.

## Adapter kinds

Adapter file names follow a strict pattern that signals the adapter kind and lets the matching path-scoped rule load.
The file name is the `snake_case` of the one type it holds.

| Adapter kind             | File name pattern (type)                                               | Placement                                              |
|:-------------------------|:-----------------------------------------------------------------------|:-------------------------------------------------------|
| Database persistence     | `postgres_<port>.rs` (`Postgres<Port>`)                                | `src/driven/postgres/`                                 |
| Stored document decoding | `stored_<concept>.rs` and `decoded_<concept>.rs`                       | `src/driven/postgres/<document>/`, beside its adapter  |
| Caching decorator        | `cached_<port>.rs` (`Cached<Port>`) and its `<port>_cache.rs` store    | `src/driven/cache/`                                    |
| Clock                    | `<kind>_clock.rs` (trait) and `system_<kind>_clock.rs`                 | beside the adapter that reads it (`src/driven/cache/`) |
| Probe                    | `<technology>_<dependency>_probe.rs` (`<Technology><Dependency>Probe`) | the readiness slice, `src/query/health/database/`      |

The real instances: `PostgresTaxRulesCatalog` implements the outbound port `TaxRulesCatalog`, its JSON row decodes
through `src/driven/postgres/tax_rules_document/` (`StoredTaxRules`, `DecodedTaxRules`), `CachedTaxRulesCatalog`
implements the same port around it, and `MonotonicClock` with `SystemMonotonicClock` gives the cache its instants. The
probe sits on the read side, owned by `rust-hex-query`.

Every placeholder resolves against the service's own ubiquitous language, never against a sibling service's. `<port>`
is the outbound port the adapter implements (`tax_rules_catalog`), so the adapter reads as `{Technology}{Capability}`.
`<concept>` is the domain type a decoded shape carries. Read the sibling modules under `src/driven/` before adding one:
the service has already chosen its terms, and a new adapter joins them rather than introducing a parallel vocabulary.

The bare `adapter.rs` file and the `Adapter` type suffix are prohibited. They are ambiguous (the database adapter, the
decorator, and the probe are all adapters in the hexagonal sense). Pick the specific pattern matching the adapter's
responsibility.

The homes admitted for code two adapters share are owned by `rust-design-principles` § DRY. A collaborator only one
adapter reads lives beside it, private to that technology's module.

## Driver vs Driven naming

The shapes and how-to are in the rust-creating-driven and rust-creating-driver skills.

## Anti-corruption layer

The shapes and how-to are in the rust-applying-hexagonal-architecture skill. In this service the anti-corruption layer
of the database is `src/driven/postgres/tax_rules_document/`, whose decoding is owned by `rust-hex-sql-queries` § Row
decoding.
