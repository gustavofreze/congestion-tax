---
name: rust-creating-driver
description: Builds a Driver (inbound) HTTP adapter and owns the write error mapping, via the endpoint sub-flow. Use when creating a write endpoint, route, request, or response.
---
<!-- Vendored from payment (.claude/skills/php-creating-driver/SKILL.md) on 2026-11-18. Adapted to this project: the Slim endpoint classes became axum handler functions in src/driver/http/, the ExceptionMapping became error_response.rs, and the webhook sub-flow, authentication, pagination, and idempotency were dropped because this project has none of them. -->

# Add driver adapter

Builds the inbound (primary) HTTP entry under `src/driver/http/` and owns the write error mapping. This skill builds a
regular write endpoint, then points back to the driver HTTP rule for the endpoint, request, and response placement (§
Driver/Http folder structure).

Start with the thinnest entry that parses and delegates to the inbound port. Add an arm to the error mapping or a new
error only when a real failure enters, never preventively.

This project receives no provider callbacks, so the owner's webhook sub-flow (signature verification, fast
acknowledgement, replay idempotency) is not carried here. A request that needs one is a new decision for the owner.

## When to use

- Create a write endpoint, a route, a request, or a response.
- Add or extend the write error mapping (`src/driver/http/error_response.rs`).

## When NOT to use

- A full vertical slice from HTTP request to the database: use `rust-implementing-features`.
- A read endpoint: use `rust-creating-query`.
- Implement the Handler or the Command behind the endpoint: use `rust-creating-application`.
- Read, persist, or write SQL behind the outbound port: use `rust-creating-driven`.
- Change schema: use `creating-migration`.

## Endpoint placement

A write endpoint is one file per type under `src/driver/http/`: the handler function
(`<use_case>_endpoint.rs`), the request (`<use_case>_request.rs`), each response (`<name>_response.rs`), and the shared
`error_response.rs`. `src/driver/http.rs` declares the modules and re-exports the handler function `pub(crate)`, and
`src/routes.rs` mounts it on the router with the inbound port as state. The endpoint, request, and response naming are
owned by `rust-hex-driver-http` (§ Driver/Http folder structure), read it when scaffolding. The command an endpoint
dispatches comes from `rust-creating-application`. A read endpoint is not built here, it comes from
`rust-creating-query`.

## Rules applied

Load and follow, in this priority:

- `rust-hex-driver-http`: endpoint, request, response, the write error mapping, the endpoint and request placement (§
  Driver/Http folder structure), content negotiation, and CORS. The error mapping shape is in
  `references/error-mapping.md` here.
- the `writing-documentation` skill: the endpoint enters the docs page and `openapi.yaml`. `web-documentation` owns
  the cross-file sync, and the prose style follows the global default in `AGENTS.md` § Global defaults.
- `rust-testing-unit`: exhaustive test of the endpoint adapter, with no dedicated test of the error mapping.

## Endpoint (write) sub-flow

Target: the route or use case named in the request (e.g. `CalculateCongestionTax`).

### Assembly order

1. Read the contract of the route: the spec at `<spec-root>` where it is set, and otherwise `openapi.yaml` and the docs
   page.
2. Request: a `#[derive(Deserialize)]` struct with private fields, converted into the command with `From`. The
   endpoint takes the body as `Result<Json<Request>, JsonRejection>`, so a body that does not match the contract is
   answered by the error mapping instead of axum's default rejection.
3. Endpoint handler function, generic over the inbound port it takes as `State<Arc<P>>`, delegating to it with no
   business rule.
4. Write error mapping (`error_response.rs`), if new: one arm per variant of the inbound port's failure enum, plus the
   invalid-request answer for a rejected body. A write endpoint adds its arms to `src/driver/http/error_response.rs`,
   never to a query slice's mapping.
5. Route in `src/routes.rs`, with the inbound port as the router state.
6. Endpoint docs per the documentation rule, kept in sync with `openapi.yaml`.
7. Test covering parsing, success, and error translation, via `rust-writing-tests`.

### Completeness gate

- [ ] Endpoint only parses and delegates, with no business rule.
- [ ] Adapter imports only inward (`crate::application::ports::inbound`, `crate::application::commands`), never a
      `crate::driven` or a `crate::query` type.
- [ ] Every variant of the inbound port's failure enum gets its arm in the error mapping, plus an OpenAPI example,
      plus a docs entry, all in the same change. The `match` is exhaustive with no wildcard arm, so a new variant fails
      to compile until it is mapped.
- [ ] Status codes come from `axum::http::StatusCode` constants, never a magic number.
- [ ] Every parsing and error-translation branch covered by a test.
- [ ] Endpoint docs updated.
- [ ] Runnable check: `make tests` passes and `make review` is clean.

## Shapes and how-to (read when scaffolding)

- `references/error-mapping.md`: the error mapping code shape (the `{code, message}` body, the exhaustive `match`, the
  fixed message versus the error's own message).
- For GoF pattern selection, use the `rust-applying-design-patterns` skill (cross-cutting). The naming invariant is in
  `rust-code-style` (§ Design-pattern names).
- Endpoint, request, and response placement: `rust-hex-driver-http` (§ Driver/Http folder structure).
- The 422 versus 409 versus 404 choice and the `{code, message}` error envelope: `rust-hex-driver-http` (§ Decision
  test, § Error response structure).
- Content negotiation and CORS: the matching sections of the same rule. The endpoint and request code shapes are not
  extracted yet: when they are, they move under `references/` here and this section points to them.

## Does not do

- Does not orchestrate the full vertical slice across layers: use `rust-implementing-features`.
- Does not build the read side (a read endpoint): use `rust-creating-query`.
- Does not implement the Handler or the Command behind the endpoint: use `rust-creating-application`.
- Does not read, persist, or write SQL behind the outbound port: use `rust-creating-driven`.
- Does not change schema: use `creating-migration`.
