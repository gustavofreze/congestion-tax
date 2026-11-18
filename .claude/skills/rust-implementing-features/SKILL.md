---
name: rust-implementing-features
description: Orchestrates the vertical slice in dependency order with one contract read and a completeness gate. Use when implementing a feature or endpoint end to end, or any change crossing more than one layer.
---
<!-- Vendored from payment (.claude/skills/php-implementing-features/SKILL.md) on 2026-11-18. Adapted to this project: the spec read became one read of openapi.yaml, the docs pages, and AGENTS.md (no spec skill, <spec-root> is absent), webhooks, outbound gateways, and the outbox were dropped from the steps, the style hook and MSI became make review and the cargo-mutants run of make tests, and every route names the rust-* skills and rules. -->

# Implement feature

Orchestrates the vertical slice. Does not apply rules directly: each step delegates to the layer action, which carries
its own rules. The role here is assembly order, a single contract read, and the gate.

Start with the thinnest slice the contract demands. Add a layer only when the use case reaches it, never scaffold a
layer the slice does not need. Complexity earns its place as the request grows. It is not poured in up front.

Target: the feature or use case named in the request.

## When to use

- The request goes from an entry point (an HTTP endpoint) to persistence, crossing more than one layer.
- Implement a use case or endpoint end to end.

## When NOT to use

- It touches a single layer: use the specific layer skill (`rust-creating-application`, `rust-creating-driven`,
  `rust-creating-driver`, `rust-creating-query`).
- The change is read-only (a list, a find, or a health check with no use case): go straight to `rust-creating-query`.
- The work is a behavior-preserving refactor or rename inside one layer: the layer skill owns it, no orchestration is
  warranted.
- The question is only what the contract says: read `openapi.yaml` and the docs pages and stop there.

## Rules applied

None directly. Each step delegates and the called action carries its rules. At the close, confirm the
`writing-documentation` skill (with `web-documentation` for sync, and `AGENTS.md` § Global defaults for the prose style)
and the testing rules (`rust-testing`, `rust-testing-unit`, `rust-testing-integration`).

## Assembly order

1. Read the contract once. With `<spec-root>` set, the spec there. With it unset, `openapi.yaml`, the docs pages under
   `docs/`, and the domain vocabulary and project-specific rules in `AGENTS.md`. Fix the scope and what is out of it.
2. Application inner hexagon (the domain plus the use-case command, inbound port, handler, and outbound port):
   `rust-creating-application`.
3. Migration, if there is new schema or new content: `creating-migration`.
4. Driven adapters (the persistence adapter over PostgreSQL, and any decorator over it such as a cache), each
   implementing an application outbound port: `rust-creating-driven`.
5. Driver HTTP entry (an axum endpoint): `rust-creating-driver`.
6. Read side, if the request includes a query: `rust-creating-query`.
7. Unit and integration tests: `rust-writing-tests`.

## Completeness gate

- [ ] Every layer within scope exists and passed its action gate.
- [ ] Nothing outside the defined scope was touched. The scope comes from the spec at `<spec-root>`, and with that
      unset it comes from `openapi.yaml`, the docs pages, and the request itself, so the item holds either way and is
      never skipped.
- [ ] Runnable check: `make tests` passes (unit and integration) and `make review` is clean.
- [ ] Coverage and mutation per the testing rules (`make tests` runs `cargo llvm-cov` and `cargo mutants`). On a large
      slice, close with `rust-reviewing-conformance`.

## Does not do

- Does not widen scope beyond what the contract defined.
- Does not skip a layer gate to check later.
