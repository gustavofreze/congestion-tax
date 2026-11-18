<!-- Vendored from payment (.claude/skills/php-writing-documentation/references/docs-page.md) on 2026-11-18. Adapted to this project: the php-reading-spec route now reads the spec at <spec-root> directly, the URL template carries no version prefix, the paginated, rate-limited, and idempotent variants and the Authorization header were dropped (none exists here), and the health page reads what axum answers instead of a reverse proxy. -->

# Documentation page structure

Use this shape when writing a per-endpoint entry in `docs/USE_CASES.md`, `docs/QUERIES.md`, `docs/HEALTH.md`, or
`docs/WEBHOOKS.md`.

## Contents

- Table of contents
- Per-endpoint section (header, method and URL, headers, request, response)
- Response blocks (with body, without body, with notable headers)
- Health check page

`````text
# Endpoint docs page structure (template)

`docs/USE_CASES.md` (write operations), `docs/QUERIES.md` (read operations), `docs/HEALTH.md`
(health and observability), and `docs/WEBHOOKS.md` (inbound webhook handlers) all share one strict
per-endpoint structure, and each exists only when it has an endpoint to carry. The format is
identical across files. Which endpoints exist, their paths, methods, fields, status codes, headers,
and error codes are contract facts owned by the spec at `<spec-root>`, and with it unset by
`openapi.yaml`. Do not invent them. The naming derivation and the fixed-wording tables live under
`references/`.

**Omit empty sections.** Render a section or table only when it has content. If an endpoint has no
body parameters, no path parameters, no query parameters, or no notable response headers, omit the
subsection and table entirely. Never render an empty table or a table with a single placeholder row.
Render the Headers table whenever the endpoint requires at least one header.

## Table of contents

`docs/USE_CASES.md`, `docs/QUERIES.md`, and `docs/HEALTH.md` start with an anchor-linked bullet list
at the top of the file, with no heading and no preamble. Each bullet points to the auto-generated
anchor of an endpoint heading further down the file.

```
* [<Endpoint name>](#<anchor-slug>)
* [<Endpoint name>](#<anchor-slug>)
```

## Per-endpoint section

### Endpoint header

The `## <Endpoint name>` heading IS the anchor target. Markdown auto-generates `#endpoint-name` from
it. No `<div id='...'></div>` HTML anchor is added. When the endpoint is restricted to internal
callers, add the blockquote note directly below the purpose line. The note states the fact and stops
there, it does not name the caller or cite a spec section (whether an endpoint is internal is a
contract fact owned by the spec).

```
## <Endpoint name>

#### <One-line purpose description>.

> **Note:** Internal endpoint.
```

### HTTP method and URL

The path is the one the router mounts, with no version prefix unless the router carries one.

```
**<METHOD>** `{{<service-name>-dns}}/<path>`
```

### Headers

| Header | Type | Description | Constraints | Required |
|:-------|:----:|:------------|:------------|:--------:|

Use the fixed wording from `references/fixed-wording.md` for `Content-Type`. `Content-Type` is
mandatory whenever the endpoint accepts a request body. Omit the entire `### Headers` section when
the endpoint has no headers.

### Request

**Path and query parameters**:

| Parameter | Type | Description | Constraints | Required |
|:----------|:----:|:------------|:------------|:--------:|

**Body parameters**:

| Parameter | Type | Description | Constraints | Required |
|:----------|:----:|:------------|:------------|:--------:|

Followed by a complete JSON example containing every field from the table, including optional fields
with realistic values. Parameters in the table are ordered by name length ascending, except that a start and
end pair (`start_time` and `end_time`) stays adjacent, start first.

### Response section

Every possible status code is documented with the same block structure. All applicable lines
(`Description`, `Content-Type`, `Body`, notable `Headers`) appear on every response, success and
error alike. Shorthand or omission is prohibited. Status codes are listed in ascending numerical
order: `200`, `204`, `404`, `405`, `409`, `422`, `500`, `503`.

**Responses WITH a body** (any status code that returns content):

````
- `<STATUS_CODE> <STATUS_TEXT>`

  **Description**: <What this status means in the context of this endpoint.>

  **Content-Type**: application/json

  **Body**:
  ```json
  { ... }
  ```
````

When a status code has multiple possible error codes, list each variant in its own fenced block with
an `or` separator, one JSON body per block. Never stack multiple JSON objects inside one fenced block
and never wrap them in a JSON array, because each documented body is a single top-level object.

````
  ```json
  {
      "code": "FIRST_ERROR_CODE",
      "message": "First error description."
  }
  ```

  or when <condition>:

  ```json
  {
      "code": "SECOND_ERROR_CODE",
      "message": "Second error description."
  }
  ```
````

**Responses WITHOUT a body** (`204 No Content`, or a `405` the router answers empty):

````
- `204 No Content`

  **Description**: <What this status means in the context of this endpoint.>

  **Content-Type**: _(no content)_
````

**Responses with notable headers** (e.g., `Location`, `Retry-After`):

````
- `<STATUS_CODE> <STATUS_TEXT>`

  **Description**: <What this status means in the context of this endpoint.>

  **Headers**:
  | Header     |  Type  | Description                        | Constraints                                   | Required |
  |:-----------|:------:|:-----------------------------------|:----------------------------------------------|:--------:|
  | `Location` | String | URI of the newly created resource. | Must be an absolute URI. E.g., `/users/1`.    |   Yes    |

  **Content-Type**: application/json

  **Body**:
  ```json
  { ... }
  ```
````

## Health check page

Health and observability endpoints (liveness, readiness, metrics) live in `docs/HEALTH.md`. They
describe operational contracts, not domain operations, and never go in `USE_CASES.md` or
`QUERIES.md`. `docs/HEALTH.md` follows the same per-endpoint structure above.

**Document what answers the route, not what the code would answer.** A reverse proxy in front of the
application can hold an exact `location` for a probe and reply itself, and then the handler you read
in the source is never reached. With `<local-proxy>` absent nothing sits in front of the application,
so the axum handler answers every probe. Read the compose stack, and call the route, before writing
the response block. Ten services once published a JSON envelope for liveness because the handler
returns one, and all ten were wrong: the proxy answered first, in plain text.

Health endpoints:

- Do not require authentication.
- **Liveness.** `200 OK` with the body the responder actually writes. Where a proxy answers it, that
  is `Content-Type: text/plain` and the literal marker the proxy returns, never a JSON object. Where
  the application answers it, that is the JSON object its handler writes.
- **Readiness, `200 OK`.** `Content-Type: application/json`, an object carrying the per-dependency
  check list and the aggregate status, both required. The status enum holds only the values reachable
  with a `200`.
- **Readiness, `503 Service Unavailable`.** `Content-Type: application/json`, an object whose only
  required member is the aggregate status. The check list is optional there, because a draining task
  answers before any check has run.
- **The check entries.** Their members belong to the code that builds them (the response of the
  health slice). Read them from that code, never from a sibling service: a sibling that copied the
  shape without measuring it propagates the same error, and a contract that seven services agree on
  is not a contract anyone verified.
- **No error envelope, in either body.** `{ "code": "...", "message": "..." }` is the domain error
  shape. A probe reporting itself unready is not a domain error, and offering the envelope as an
  option here is what licensed three services to publish the wrong `503`.
- Both readiness bodies are named schemas under `components/schemas/`, referenced identically by the
  docs page and `openapi.yaml`.
`````
