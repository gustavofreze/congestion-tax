<!-- Vendored from payment (.claude/skills/php-writing-documentation/references/naming-derivation.md) on 2026-11-18. Adapted to this project: the php-reading-spec route now reads the spec at <spec-root> directly, and this project's own endpoint names join the examples. -->

# Endpoint naming derivation (reference)

Read this while naming an endpoint across the docs page and `openapi.yaml`. The Endpoint Name has one source and the
anchor slug and `operationId` derive from it mechanically. This file is narrative how-to, the synchronization invariant
that keeps the two renderings aligned stays in `web-documentation`. The actual endpoints and their summaries are
contract facts owned by the spec at `<spec-root>`, and with it unset by `openapi.yaml`.

## Endpoint name pattern per file

The `<Endpoint name>` follows a fixed pattern per file. Each docs page exists only when it has an endpoint to carry.

| File                | Pattern              | Examples                                                         |
|:--------------------|:---------------------|:-----------------------------------------------------------------|
| `docs/USE_CASES.md` | `<Noun> <gerund>`    | `Congestion tax calculating`, `Payment creating`                 |
| `docs/QUERIES.md`   | `Find <what>`        | `Find payment by id`, `Find payments by organization`            |
| `docs/HEALTH.md`    | `<Probe> check`      | `Liveness check`, `Readiness check`, `Startup check`             |
| `docs/WEBHOOKS.md`  | `<Provider> webhook` | `Asaas webhook`                                                  |

## Mechanical derivation

Three derived names share a single source, the Endpoint Name. The Endpoint Name in the docs page MUST match the
`summary` of the same endpoint in `openapi.yaml`. The anchor slug is what Markdown auto-generates from the heading
(lowercase, spaces replaced with hyphens). The `operationId` replaces the spaces with case boundaries and lowercases the
first letter.

| Endpoint Name                   | Anchor slug (`kebab-case`)      | OpenAPI `operationId` (`camelCase`) |
|:--------------------------------|:--------------------------------|:------------------------------------|
| `Congestion tax calculating`    | `congestion-tax-calculating`    | `congestionTaxCalculating`          |
| `Payment creating`              | `payment-creating`              | `paymentCreating`                   |
| `Payment refunding`             | `payment-refunding`             | `paymentRefunding`                  |
| `Find payment by id`            | `find-payment-by-id`            | `findPaymentById`                   |
| `Find payments by organization` | `find-payments-by-organization` | `findPaymentsByOrganization`        |
| `Liveness check`                | `liveness-check`                | `livenessCheck`                     |
| `Readiness check`               | `readiness-check`               | `readinessCheck`                    |
| `Asaas webhook`                 | `asaas-webhook`                 | `asaasWebhook`                      |

When any of the three names changes, all three change together in the same commit, per the synchronization invariant
owned by `web-documentation`.
