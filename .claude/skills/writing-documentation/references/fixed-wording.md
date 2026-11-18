<!-- Vendored from payment (.claude/skills/php-writing-documentation/references/fixed-wording.md) on 2026-11-18. Adapted to this project: the php-reading-spec route now reads the spec at <spec-root> directly, the Authorization, Idempotency-Key, 400, and 401 wording was dropped (no authentication, idempotency, reverse proxy, or bootstrap guard here), and the 405 and 500 rows state what the axum router answers. -->

# Fixed documentation wording (reference)

Read this while filling header tables and response blocks. These tables fix the documentation wording so the docs pages
and `openapi.yaml` do not drift apart. They are the single source of wording across both renderings.

The contract facts these tables render are owned by the spec at `<spec-root>`, not by this skill: the error-code
values and the `{ code, message }` envelope shape. Read those from the spec where it is set, and from `openapi.yaml`
where it is not. Do not invent them. What lives here is only the fixed phrasing that keeps the two documentation
renderings identical.

These tables are copied verbatim into every docs page and into `openapi.yaml`, so they never name a sibling service. A
proper name written here is published in the API contract of every service that renders it. Where a name matters to
the consumer, the contract supplies it and the docs page carries it, the table does not.

This project has no authentication, no idempotency key, no rate limit, and no reverse proxy in front of the
application, so the owner's fixed wording for `Authorization`, `Idempotency-Key`, `400`, and `401` has no subject here
and is not carried. Adding one of them is a contract change, and its fixed wording comes back with it, never from a
guess.

## Standard headers (fixed wording)

When `Content-Type` appears in the Headers table, use the exact wording below to avoid drift between endpoints.

| Header         |  Type  | Description               | Constraints               | Required |
|:---------------|:------:|:--------------------------|:--------------------------|:--------:|
| `Content-Type` | String | The request content type. | Must be application/json. |   Yes    |

## Standard responses (fixed wording)

These responses apply where the service produces them. Their `Description` text is fixed: use the exact wording across
all docs and OpenAPI. A row applies to an operation only when the service actually answers that status on it, so read
what answers the route before listing one (`references/openapi.md` structural invariant 2).

`405` applies to every path the router mounts, because axum answers a known path under a method it does not route
before any handler runs. Without a method-not-allowed fallback on the router, that answer carries an empty body, never
the `{ code, message }` envelope, so the docs page renders it as a response without a body. `500` applies only where a
layer of the router turns an unexpected failure into a response. A handler that panics with no such layer answers
nothing at all, so an operation documents no `500` until the router carries that layer.

| Status | Fixed description                                                                       |
|:-------|:----------------------------------------------------------------------------------------|
| `405`  | Indicates that the path exists but does not accept the request method.                  |
| `500`  | Indicates that an unexpected error occurred on the server while processing the request. |

The `code` and `message` values for these, where the service answers them with a body, mirror the contract-owned error
envelope.

- `405`: `{ "code": "METHOD_NOT_ALLOWED", "message": "Method Not Allowed." }`, only once a router fallback writes it.
- `500`: `{ "code": "INTERNAL_ERROR", "message": "An unexpected error occurred." }`

**`503` has no fixed wording, and it is not a standard response.** Two different endpoints reach it and neither is
covered by the table above. A readiness probe answers `503` with its own diagnostic body, never with the
`{ code, message }` envelope, and the § Health check page of `references/docs-page.md` owns that shape. A service that
depends on another service or a store answers `503` with the envelope when that dependency is unavailable, and the
`code` comes from its own error mapping rather than from a fixed catalog. Do not list `503` on endpoints that can return
neither.

## Domain-specific status codes (carve-out)

A status code carries domain semantics when its emission is tied to a specific business or session-lifecycle condition
rather than the generic protocol-level meaning. In those cases the fixed wording above is replaced by a custom,
consumer-actionable description. The same custom wording MUST appear identically across the docs page and
`openapi.yaml`. Whether a status code carries domain semantics is a contract decision owned by the spec at
`<spec-root>`, and with it unset by the owner.

`503` has no fixed wording to carve out from, and the § Standard responses section above says where each of its two
shapes comes from.

Constraints on the carve-out:

- The custom description must be precise and consumer-actionable. Vague descriptions ("an error occurred") are
  prohibited.
- The `code` value follows the standard `SCREAMING_SNAKE_CASE` convention. The carve-out is for the `description` text,
  not for the error-code naming.
- When a `503` returns a diagnostic body, the schema MUST be defined under `components/schemas/` in `openapi.yaml` and
  referenced from the `503` response. Inline schemas remain prohibited. An endpoint whose `503` reports an unavailable
  dependency instead carries the `{ code, message }` envelope, with the `code` its own error mapping emits.
- When in doubt about whether a status code is generic-protocol or domain-specific, default to the standard fixed
  wording. The carve-out is the exception, not the rule.
