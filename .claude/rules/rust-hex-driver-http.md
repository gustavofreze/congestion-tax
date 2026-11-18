---
description: HTTP rules for the inbound side (src/driver/http and src/query). The structural residue only, the src/driver/http layout and the error-to-HTTP boundary invariant (domain errors stay pure, translation only at the boundary through an exhaustive match, the {code,message}/SCREAMING_SNAKE envelope, the 422/409/404 decision test). Shapes and tables live in their owners, named inline. Rate limiting and content negotiation each carry a portable default that the spec overrides where it speaks.
paths:
    - "src/driver/http/**/*.rs"
    - "src/query/**/*.rs"
    - "src/routes.rs"
---
<!-- Vendored from payment (.claude/rules/php-hex-driver-http.md) on 2026-11-18. Adapted to this project: Slim and PSR-15 became axum, the Endpoints/Validations/Middlewares layout became the flat src/driver/http/ module (endpoint, request, response, and error_response files), ExceptionMapping became the exhaustive match in error_response.rs over the inbound failure enum, and the 503 SERVICE_UNAVAILABLE arm and the readiness report were recorded. The Webhook, event, Proxy, and Session entries, the BFF collaborators, the health-only mapping case, and the Authentication, Pagination, Idempotency, and CORS sections were dropped because this service has none of them, and the php-reading-spec references were dropped because the skill is not ported. -->

# HTTP (inbound)

Rules for the inbound side of the hexagon: the axum endpoints under `src/driver/http/` and the read side under
`src/query/`, both mounted on one router by `src/routes.rs`. This service calls no external HTTP API, so it has no
outbound HTTP adapter.

This rule keeps only the always-on structural residue: the `src/driver/http/` layout and the error-to-HTTP boundary
invariant. The mapping code shape lives in its owner. Rate limiting and content negotiation each carry a portable
default below, which the spec at `<spec-root>` overrides where it speaks, so each resolves to something with or without
a spec.

## Pre-output checklist

1. Domain errors stay pure. No HTTP status inside them. HTTP translation happens only at the `src/driver/http/`
   boundary, in `error_response.rs`, and on the read side in the slice's own `http/` module. See § Error to HTTP
   mapping.
2. The decision between `422`, `409`, and `404` follows the "would the same request fail again with no server-side
   changes?" test. See § Decision test.
3. Error responses use the `{"code": "...", "message": "..."}` envelope. `code` is `SCREAMING_SNAKE_CASE`.
   `INVALID_REQUEST` is reserved for a body that does not match the request contract. See § Error response structure.
4. The `src/driver/http/` layout (endpoint, request, and response files, and the root-level `error_response.rs`) is
   respected. See § Driver http folder structure.

## Driver http folder structure

The `src/driver/http/` module separates endpoint code from shared HTTP infrastructure:

1. **`<operation>_endpoint.rs`** holds one endpoint, an `async fn` named after the operation verb-noun with no suffix
   (`calculate_congestion_tax`), generic over the inbound port it receives as `State<Arc<P>>`. It extracts the body as
   `Result<Json<Request>, JsonRejection>`, converts the request into the command, calls the inbound port, and maps the
   outcome. It holds no business rule. The read side follows the same convention (`readiness` in
   `readiness_endpoint.rs`).
2. **`<operation>_request.rs`** holds the `Deserialize` request with private fields of primitives and its
   `From<Request> for Command`. **`<read_model>_response.rs`** holds one `Serialize` response with its
   `From<ReadModel>`. One type per file gives every nested response a file of its own
   (`daily_congestion_tax_response.rs`).
3. **`error_response.rs`** at the module root holds the cross-cutting translation consumed by every endpoint:
   `invalid_request` for a `JsonRejection` and `from_failure` for the inbound port's failure enum, both `pub(super)`,
   plus the router fallbacks `route_not_found` and `method_not_allowed`, `pub(crate)` for `src/routes.rs`. They answer
   the `ErrorBody` of `error_body.rs` as the envelope. It never lives inside an endpoint file.
4. **`src/driver/http.rs`** declares the modules privately and re-exports only the endpoints, as `pub(crate)`, for
   `src/routes.rs` to mount. The router binds each endpoint to its concrete port with `.with_state`, and the composition
   root in `src/main.rs` builds the concretes. No tower layer handles errors: each endpoint maps its own outcome.

The service carries only the entry kinds it has, and it has one (the API), so the endpoints sit flat in
`src/driver/http/` with no entry subdirectory. Request validation is the `Deserialize` shape of the request plus the
domain constructors the handler calls, so there is no shared validation module.

## Error to HTTP mapping

`src/driver/http/` and the read slice's `http/` module are the only places where the service's own errors are converted
to HTTP responses. Domain errors stay pure: they carry no HTTP status and no formatted message (owned by
`rust-hex-application-domain` § Error layer placement). Every variant of the inbound port's failure enum
(`CalculationFailure`) has an explicit arm in `error_response::from_failure`, and the `match` has no wildcard arm, so
the compiler refuses a new variant until it is mapped. A domain error raised only while decoding stored content
(`TaxPeriodsOverlap`, `TariffBandsOverlap`, `CurrencyFormatNotValid`) never reaches the driver: the database adapter
wraps it in `TaxRulesUnavailable`, so it is no variant of the enum and gets no arm, because the arm could never run and
would stand as code no test can cover. A library error never gets a variant of its own: the adapter that sees it wraps
it in the error declared beside its port. A handler never converts a domain error into a different one, it lifts it
into the failure enum through `#[from]` and `?`. A new domain or business error that reaches the enum adds its variant,
its mapping arm, its OpenAPI `examples`, and its docs entry in the same change.

An infrastructure failure answers `503` with the fixed code `SERVICE_UNAVAILABLE` and the message
`The service is not ready.`, and the arm logs the cause with `tracing::error!`. The cause never reaches the body.

The one carve-out is a domain guard that request validation fully shields, so it can never surface through any
endpoint. Giving it a mapping arm would add an endpoint-unreachable branch that no test can exercise (dedicated mapping
tests are prohibited), so it survives mutation and reads as dead code, both forbidden by the 100 percent mutation and
no-dead-code rules. Such a guard is covered by its domain test alone and stays out of the mapping. A guard qualifies
only when request validation provably shields every endpoint that can reach it, so the instances are read from the
service's own requests and domain tests, never from a list kept here.

The mapping code shape (the `(status, code)` arms and the shared `respond`) is in the rust-creating-driver skill.

### Decision test

Ask: **"If I sent the exact same request again with no server-side changes, would it still fail?"**

- **Yes**, the input itself is invalid, so `422`.
- **No**, the failure depends on server state, so `409` (or `404` when the resource is missing).

`TaxRulesNotFound` answers `404`: the same request succeeds once a migration seeds the rules of that city.
`PassageOutsideTaxPeriods` answers `422`: the passage falls outside every period the city holds.

### Error response structure

```json
{
    "code": "SCREAMING_SNAKE_CASE_ERROR_CODE",
    "message": "Human-readable description."
}
```

The `code` is the error type name in `SCREAMING_SNAKE_CASE` (`PassageOutsideTaxPeriods` gives
`PASSAGE_OUTSIDE_TAX_PERIODS`), written as a literal in its arm because Rust has no runtime type name to derive it from.
The `message` is the error's `Display`. `INVALID_REQUEST` is reserved for a body that does not match the request
contract, and its message is the rejection's `body_text()`. That envelope is the default and it holds on every error
response, 4xx and 5xx alike, with no field beyond `code` and `message`, so checklist item 3 always has a defined
envelope to check against. The readiness report is not an error response: its `503` carries the diagnostic body owned
by `docs/HEALTH.md`. The canonical code values are owned by the spec at `<spec-root>`, which overrides this default
where it speaks. With `<spec-root>` unset the envelope above is the whole contract, and `openapi.yaml` is the register
of its code values.

## Rate limiting

The default: throttling is an edge concern and no endpoint implements it. The application enforces a limit only when the
limit is itself a business rule (a per-tenant quota), and it then answers `429`. The limits, the buckets, and the layer
that enforces each one are owned by the spec at `<spec-root>`, which overrides this default where it speaks.

## Content negotiation and CORS

The default: the service speaks `application/json` on both request and response and negotiates nothing else. The
service is not browser-facing, so no endpoint and no layer emits an `Access-Control-*` or `Timing-Allow-Origin` header.
Any additional media type is owned by the spec at `<spec-root>`, which overrides this default where it speaks.

## Tests

Owned by the rust-writing-tests skill, the no-dedicated-mapping-test prohibition included. Every arm of
`error_response.rs` is reached through the endpoint test, which drives `routes::router` with
`tower::ServiceExt::oneshot` over a `CalculatingCongestionTaxSpy`.
