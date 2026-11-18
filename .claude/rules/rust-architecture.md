---
description: Hexagonal architecture structural residue only, the inward dependency rule, bounded-context subdirectory naming, CQRS, the infrastructure-files exception, the composition root and Settings contract, and one bounded context per service. Layer shapes and strategic DDD live with their owners.
paths:
    - "src/**/*.rs"
---
<!-- Vendored from payment (.claude/rules/php-architecture.md) on 2026-11-18. Adapted to this project: layer paths in Rust module casing, the Settings and Dependencies contract mapped to the binary composition root (`src/main.rs` and the settings modules it declares), the `src/` root list and the coverage and mutation exclusions read from this repository, the folder skeleton added here because the skeleton skill is not ported, and the BFF, webhook, messaging, and Driven/Shared carve-outs dropped. -->

# Hexagonal architecture (ports & adapters)

This rule keeps only the always-on structural residue of the hexagon: the inward dependency rule, the folder skeleton,
the bounded-context subdirectory-naming invariant, the CQRS one-liner, the infrastructure-files exception, the
composition root and Settings contract, and the one-bounded-context-per-service fact. The per-layer shapes and strategic
DDD how-to are delegated to their owners, listed at the end.

## Pre-output checklist

Verify every item before producing or relocating any file under `src/`.

1. Dependencies point inward only: `driver` to `application` to `domain`, and `driven` implements
   `application/ports/outbound/`. See § Dependency rule.
2. A new module lands in the tree of § Folder skeleton, declared from its parent module file (`application.rs`,
   `driven/cache.rs`), never from a `mod.rs`. See § Folder skeleton.
3. Subdirectory names under the layers represent **bounded contexts**, aggregates, transports, or technologies, never
   visibility, audience, technical role, or layering. See § Subdirectory naming.
4. CQRS holds: each query slice is self-contained, and a driven adapter is never reused for reads. See § CQRS.
5. The `src/` root holds only the closed list of § Infrastructure files exception.
6. One bounded context per service, translated at the `driven/` boundary. See § Bounded context.
7. `<Name>Settings` reads the environment via `EnvironmentVariable::from` only and declares no error of its own. See §
   Settings.
8. `src/main.rs` is declarative wiring only: no validation logic and no guard that fails on a value. See § Composition
   root.

## Dependency rule

Dependencies always point inward:

1. `driver` to `application` to `domain`.
2. `driven` implements the ports (traits defined in `application/ports/outbound/`).
3. `domain` has zero knowledge of `driver`, `driven`, `query`, or framework code. External crates are restricted to a
   small whitelist. The `check-domain-dependencies` step of `make review` enforces it: `src/application/domain/` may
   use only `std`, itself, and the crates the `Makefile` lists in `DOMAIN_CRATES` (`jiff`, `thiserror`).

All coupling between layers is trait-based injection through `application/ports/`. A handler receives its outbound port
as a generic parameter (`CalculatingCongestionTaxHandler<C: TaxRulesCatalog>`), and the driver receives the inbound port
the same way (`router<P: CalculatingCongestionTax, D: DependencyProbe>`). No concrete adapter is named inside domain or
application code.

## Folder skeleton

The library crate declares the tree from `src/lib.rs`, one module file per directory (`application.rs` beside
`application/`), because `Cargo.toml` turns on `mod_module_files`, which rejects `mod.rs`.

```
src/
├── main.rs, app_settings.rs, database_settings.rs, environment_variable.rs   # binary: the composition root
├── lib.rs, routes.rs                                                          # library: module tree and router
├── application/
│   ├── domain/
│   │   ├── models/tax_rules/   # aggregate root TaxRules and its value objects
│   │   └── errors/             # one error type per violated invariant
│   ├── ports/
│   │   ├── inbound/            # CalculatingCongestionTax, its read models, failure enum, and not-found
│   │   └── outbound/           # TaxRulesCatalog and its TaxRulesUnavailable
│   ├── commands/               # CalculateCongestionTax, primitives only
│   └── handlers/               # CalculatingCongestionTaxHandler
├── driver/http/                # axum endpoint, request, responses, failure mapping
├── driven/
│   ├── postgres/               # PostgresTaxRulesCatalog and the decoding of its stored document
│   └── cache/                  # CachedTaxRulesCatalog, its TaxRulesCache, and its MonotonicClock
└── query/
    ├── health/                 # read slice: DependencyProbe, its database probe, and the http endpoints
    └── tax_rules/              # read slice: TaxRulesFinding, its database finding, and the http endpoint
tests/                          # main.rs is the single target, modules mirror src/, plus support/
```

`application/domain/events/` and `application/domain/services/` do not exist, because no domain event and no rule
spanning aggregates exists. Each one is created by the change that brings its first member, never preventively.

## Subdirectory naming

Subdirectory names under `application/domain/models/`, `application/ports/`, `driven/`, `driver/`, and `query/` name a
concept the service owns: an aggregate (`tax_rules`), a read slice (`health`), a transport (`driver/http/`), or a
technology (`driven/postgres/`, `driven/cache/`). With one bounded context, the layers group by aggregate, transport,
and technology. When a service spans more than one context, the context name (a noun from the ubiquitous language)
takes the first level under each layer. `application/commands/` and `application/handlers/` stay flat, and they MAY
group their files under the same per-context folders once more than one context exists. `commons` and `shared` name no
context, and they are admitted only as the shared-code homes `rust-design-principles` § DRY lists.

They never represent:

1. **Visibility** (`internal`, `external`, `public`, `private`): a routing concern, expressed in the route path or in a
   layer that authorizes the caller.
2. **Audience** (`admin`, `backoffice`, `mobile`, `web`): a routing concern. The same slice serves whoever the route
   exposes it to.
3. **Technical role** (`sync`, `async`, `batch`, `realtime`): a deployment concern.
4. **Layering** (`service`, `manager`, `handler` as a context name): technical labels, not business concepts.

When a query or endpoint is restricted to a specific consumer, the restriction lives in the route, the OpenAPI tag, or
the documentation. Never in the folder name.

## CQRS

Commands mutate state and return nothing or a typed result. Queries are read-only and return typed read models. The
read side is fully segregated: a driven adapter is never reused for reads, and nothing from `query/` is reused elsewhere
in the hexagon. The readiness slice shows it: `query/health/database/postgres_database_probe.rs` owns its own access to
the pool and shares no code with `driven/postgres/`. The full read-write isolation contract is owned by the
`rust-hex-query` rule.

## Infrastructure files exception

The `src/` root (no subdirectory) admits exactly these files, a closed list:

1. `main.rs`, the binary and composition root, and the three modules only it declares: `app_settings.rs`,
   `database_settings.rs`, and `environment_variable.rs`.
2. `lib.rs`, which declares the library module tree, and `routes.rs`, which mounts the driver and the query slices on
   one axum router.
3. The module files of the top-level layers: `application.rs`, `driven.rs`, `driver.rs`, `query.rs`.

No other file is created at the `src/` root. The binary files are exempt from the private-function and type
decomposition rules.

Exclusion from coverage and mutation is a separate and equally closed list: `src/main.rs` and the three modules it
declares, named in `COVERAGE_FILTER` of the `Makefile` and in `exclude_globs` of `.cargo/mutants.toml`, and the two
lists stay identical. A Settings type is environment-only and carries no behavior to test, and the composition root is
declarative wiring. `routes.rs` is library code reached by the endpoint tests, so it earns its coverage like every other
file.

## Composition root

`src/main.rs` is declarative wiring only: it reads the settings once, builds the pool, the adapters, and the handler,
binds concrete adapters to ports by constructor injection, and serves the router until a shutdown signal. It carries no
conditional business or validation logic and no guard that inspects a value. A precondition belongs to the concrete type
it protects, never to the code that wires it: the composition root is excluded from coverage and mutation, so logic
placed there can never be tested or mutation-killed. `anyhow` lives on this side only (`main.rs` and the settings
modules), and the library never returns it.

## Settings

`<Name>Settings` is a struct with `pub(crate)` fields and one `from_environment()` associated constructor returning
`anyhow::Result<Self>` (`AppSettings`, `DatabaseSettings`). Settings live beside `main.rs`, and only those modules read
the environment (project rule in `AGENTS.md`). Three invariants:

1. Every environment read uses `EnvironmentVariable::from`, followed by `text()` or `parsed()`, never `std::env::var`
   directly and never a code-side fallback (`unwrap_or`, `unwrap_or_default`). A missing, renamed, or unparseable
   variable fails at startup, so the app only ever runs on valid, current configuration.
2. A Settings type declares no error and fails with nothing of its own. The failures `EnvironmentVariable` raises
   (absent, or not parseable into the field's type) are the only guard. Validation beyond that (scheme, format, range)
   lives in the concrete implementation that consumes the value (an adapter, a value object), never in Settings and
   never in `main.rs`.
3. No methods beyond `from_environment()`.

## Bounded context

One bounded context per service. This service owns `congestion taxation`, its domain model is the authority for that
context, and it never references another service's domain objects directly. Integration crosses the process boundary
over HTTP. Foreign shapes are translated to this service's ubiquitous language at the `driven/` boundary (the
anti-corruption layer): `driven/postgres/tax_rules_document/` decodes the stored JSON row into `TaxRules` and its value
objects, and a row that breaks a domain invariant surfaces as `TaxRulesUnavailable`, never as a domain error leaking
past the adapter.

## Delegated to owners

- Driven adapter structure, the `driver/http/` layout, and the read-side query structure: The shapes and how-to are in
  the rust-creating-driven, rust-creating-driver, and rust-creating-query skills.
- Outbound port parameter types, handler shapes, and the persistence (sqlx) contract: The shapes and how-to are in the
  rust-creating-application and rust-creating-driven skills.
- Strategic DDD beyond the one-context fact (context map, ubiquitous-language modeling): The shapes and how-to are in
  the rust-applying-ddd skill. Domain behavior is owned by the spec at `<spec-root>`. With `<spec-root>` unset there is
  no spec tier: the contract facts come from `openapi.yaml` and the `docs/` pages, and domain behavior that neither
  states is asked for, never invented.
- SOLID, DRY, principle precedence, and pattern selection: owned by the rust-design-principles rule and the
  rust-applying-design-principles skill.
