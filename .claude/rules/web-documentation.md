---
description: The synchronization invariant across the documentation artifacts. openapi.yaml, the docs pages, and the README are renderings of one API contract and change together. Per-artifact structure, fixed wording, and naming live in the writing-documentation skill. Contract facts live in the spec.
paths:
    - "docs/**/*.md"
    - "README.md"
    - "openapi.yaml"
---
<!-- Vendored from payment (.claude/rules/web-documentation.md) on 2026-11-18. Adapted to this project: php-writing-documentation became the writing-documentation skill, the php-reading-spec route was dropped because no spec directory sits beside this repository, and each docs page is present only when it has an endpoint to carry. -->

# Documentation

This rule owns one always-on invariant: `openapi.yaml`, the docs pages (`docs/USE_CASES.md`, `docs/QUERIES.md`,
`docs/HEALTH.md`, `docs/WEBHOOKS.md`, each present only when it has an endpoint to carry), and the `README.md` are
renderings of one API contract, kept in sync. Everything else has another owner.

- Per-artifact structure (docs pages, `openapi.yaml`, README), the fixed-wording tables, and the naming derivation: the
  shapes and how-to are in the `writing-documentation` skill.
- The contract facts the documentation renders (which endpoints exist, paths, methods, fields, error codes, headers,
  pagination, idempotency, the internal-endpoint fact): owned by the spec at `<spec-root>` where it resolves, read
  directly. Not restated here. With `<spec-root>` unset the service's own `openapi.yaml` is the register of those facts,
  and where that file does not exist yet it is produced from the endpoint code, its tests, and the docs page before the
  synchronization invariant is enforced, never treated as a missing input that blocks the work. The synchronization
  invariant below holds either way.
- Prose punctuation follows the global default in `AGENTS.md` § Global defaults.

## Pre-output checklist

Verify every item before producing or modifying any documentation file.

1. Every endpoint exists in BOTH `openapi.yaml` AND the matching docs page. No endpoint exists in only one source. See §
   Synchronization (single source of truth).
2. `openapi.yaml`, the docs pages, and the README change in the same commit. JSON examples match `example`/`examples` in
   OpenAPI exactly. See § Synchronization (single source of truth).
3. Per-artifact structure, fixed wording, and naming follow the `writing-documentation` skill.
4. `openapi.yaml` declares `openapi: 3.1.0`. Every service renders the same contract family, so a document on another
   minor is a deviation and not a choice. The `writing-documentation` skill ships the skeleton that carries the version,
   and this rule owns the invariant, because the skill loads only when it runs while this rule is injected on every edit
   of an `openapi.yaml`.
5. The `nullable` keyword never appears. It belongs to OpenAPI 3.0, and 3.1 removed it when the Schema Object adopted
   JSON Schema 2020-12, so a 3.1 parser ignores it and the property reads as non-nullable, which is the opposite of what
   the author meant. Null is declared as a member of `type` (`type: [string, 'null']`), or through `oneOf` with a
   `type: 'null'` branch when the property is a `$ref`. The shapes are in the skill.
6. Only a gateway (a service whose public hostname serves its API) declares `servers` in `openapi.yaml`. An internal
   service declares none, because its public hostname answers only `/health/*`. An operation an external system
   calls on the public host may declare operation-level `servers`, and the document still declares no top-level
   `servers`.
7. Every error code the service code can produce is documented in `openapi.yaml`, in the docs pages, and in the README
   error table where the README carries one, including the codes no handler raises, such as `METHOD_NOT_ALLOWED`
   (`405`) and, on a gateway, `UPSTREAM_SERVICE_UNRESOLVED` (`500`), wherever the service produces them.

## Synchronization (single source of truth)

1. Every endpoint defined in `openapi.yaml` MUST have a corresponding entry in `docs/USE_CASES.md` (write operations),
   `docs/QUERIES.md` (read operations), `docs/HEALTH.md` (health check and observability endpoints), or
   `docs/WEBHOOKS.md` (inbound webhook handlers), and the reverse. No endpoint may exist in only one source.
2. `openapi.yaml` and the docs are two representations of the same contract. When adding a new endpoint, write the
   OpenAPI spec first and generate the docs from it. When modifying an existing endpoint, change both representations in
   the same commit. The README links to the docs page anchors, so its endpoint lists change in that same commit.
3. These elements must match exactly between the representations:
    - Endpoint path, method, and description.
    - Status codes and their descriptions.
    - Request parameters (names, types, constraints, required).
    - Response schemas (field names, types, nullable).
    - Error codes (`code` values) and their messages.
    - Examples (JSON in docs must match `example`/`examples` in OpenAPI).
4. When reviewing, verify that every `code` value in the docs appears in the OpenAPI `examples`, and the reverse.
