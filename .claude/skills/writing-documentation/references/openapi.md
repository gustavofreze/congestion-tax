<!-- Vendored from payment (.claude/skills/php-writing-documentation/references/openapi.md) on 2026-11-18. Adapted to this project: the php-reading-spec route now reads the spec at <spec-root> directly, the pagination, rate-limit, and idempotency invariants were dropped (none exists here), the shared status codes are 405 and 500, and the skeleton path carries no version prefix. -->

# OpenAPI structure

Use this shape when adding or updating an operation, schema, or tag in `openapi.yaml`.

## Contents

- Structural invariants
- Skeleton (paths, operations, components and schemas)

`````yaml
# openapi.yaml structure (template)

`openapi.yaml` is the inbound API contract in machine-readable form, placed at the service root and
versioned with the source code. It and the docs pages are two renderings of one contract, kept in
sync by the `web-documentation` rule. Which endpoints, schemas, fields, tags, and error codes exist
are contract facts owned by the spec at `<spec-root>`, and with it unset this file is their register.
Do not invent them. The Endpoint Name to `operationId` derivation lives in
`references/naming-derivation.md`. Prose inside `description` fields follows the prose punctuation
default in `AGENTS.md` § Global defaults.

## Structural invariants

1. Place `openapi.yaml` at the service root, versioned with source code.
2. Document all mandatory headers, input and output schemas, and every possible error code
   exhaustively. Every code the service code can produce is listed on the operations that can
   produce it, including the ones no handler raises: `405`, answered by the router for a known path
   under a method it does not route, and `UPSTREAM_SERVICE_UNRESOLVED` (`500`), answered by a gateway
   whose route names an upstream it cannot resolve, wherever the service produces them.
3. Follow the standardized error structure `{ "code": "...", "message": "..." }`. The envelope shape
   and the error-code values are contract facts owned by the spec.
4. Every endpoint has a unique `operationId` derived from the Endpoint Name in `camelCase`. See
   `references/naming-derivation.md` for the mechanical derivation.
5. Every endpoint has exactly one `tag`. A tag groups a cohesive set of endpoints within the bounded
   context. A service MAY define multiple tags when its endpoints split naturally into distinct
   subdomains. The intent is navigability in Swagger UI, not a strict 1:1 mapping between tags and
   services. Health check endpoints always use the `Health` tag, which is not a bounded context. The
   real tag set comes from the contract.
6. Every `422` response with multiple possible error codes uses `examples` (plural) with a named key
   per variant. Every `4xx` response with a single specific code uses `example` (singular) with the
   inline object.
7. Every property in request and response schemas includes `description` and `example`.
8. Properties and `required` lists in schemas are ordered by name length ascending, consistent with
   the codebase convention. A start and end pair (`start_time` and `end_time`, `starts_at` and
   `ends_at`) stays adjacent, start first, in properties, `required`, and examples alike.
9. Enum properties always include the `enum` keyword with the full list of allowed values.
10. Nullable properties declare `null` as a member of `type`, never the `nullable` keyword. OpenAPI 3.1 aligns
    the Schema Object with JSON Schema 2020-12, which removed `nullable` outright, so a 3.1 parser ignores it and
    the property reads as non-nullable. Use `type: [<type>, 'null']`. When the property is a `$ref`, use
    `oneOf: [- $ref, - type: 'null']` and keep the `description` as a sibling of `oneOf`.
11. Reusable schemas live in `components/schemas/`. Inline schemas are prohibited for request bodies
    and response objects.
12. `description` text for shared status codes (`405`, `500`) uses the exact wording from
    `references/fixed-wording.md`, the single source of wording across docs and OpenAPI.
13. Only a gateway (a service whose public hostname serves its API) declares `servers`. An internal
    service declares none, because its public hostname answers only `/health/*` and its API is
    reached only inside the private network, so any `servers` entry would point a reader at a host
    that refuses the operations it lists. An operation an external system calls on the public host may
    declare operation-level `servers`, and the document still declares no top-level `servers`.

## Skeleton

```yaml
openapi: 3.1.0
info:
  title: <Service name> API
  version: <version>
tags:
  - name: <Tag>
  - name: Health
paths:
  /<path>:
    <method>:
      operationId: <operationId>
      tags:
        - <Tag>
      summary: <Endpoint name>
      description: <One-line purpose>.
      parameters:
        - name: <param>
          in: <query|path|header>
          required: <true|false>
          schema:
            $ref: '#/components/schemas/<Schema>'
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/<RequestSchema>'
      responses:
        '<code>':
          description: <Fixed or domain-specific wording>.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/<ResponseSchema>'
              examples:
                <variantKey>:
                  value:
                    code: <ERROR_CODE>
                    message: <Error description>.
components:
  schemas:
    <Schema>:
      type: object
      required:
        - <field>
      properties:
        <field>:
          type: <type>
          enum: [<value>]
          description: <Field description>.
          example: <example>
        <nullable-field>:
          type: [<type>, 'null']
          description: <Field description>.
          example: <example>
        <nullable-ref-field>:
          description: <Field description>.
          oneOf:
            - $ref: '#/components/schemas/<Schema>'
            - type: 'null'
```
`````
