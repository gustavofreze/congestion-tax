<!-- Vendored from payment (.claude/skills/php-writing-documentation/references/readme.md) on 2026-11-18. Adapted to this project: the php-reading-spec route now reads the spec at <spec-root> directly, the Composer and PHPUnit wording and the tests-full target of the worked example became Cargo and this Makefile's targets, the outbox paragraph was dropped (no events here), and the worked example names no domain. -->

# README structure

Use this shape when building or restoring the repository `README.md`.

## Contents

- Fixed section order
- Worked example (the section shapes filled in)

`````text
# README structure (template)

The repository `README.md` follows a fixed section order. The README never inlines endpoint
documentation: each endpoint lives in its docs page (`docs/USE_CASES.md`, `docs/QUERIES.md`,
`docs/HEALTH.md`, or `docs/WEBHOOKS.md`). Anchor links use Markdown auto-generated heading anchors
(`lowercase-hyphenated`). The `<div id='...'></div>` HTML anchor pattern is not used. Markdown
generates `#section-name` automatically from `## Section name`.

## Fixed section order

1. **Service name**. The H1 heading, the service name in title case (each hyphen-separated word of
   `<service-name>` capitalized, so `client-gateway` reads `Client Gateway`). Nothing sits between
   the H1 and the table of contents: no badge, no tagline, no paragraph.
2. **Top-level table of contents**. Nested bullet list with anchor links to every section and
   subsection below. Use `*` for top-level bullets and `-` for nested bullets to mirror the
   hierarchy visually.
3. **`## Overview`**. One or two paragraphs describing what the service centralizes and the problem
   it solves. Include cross-cutting concerns the consumer must know about (published events,
   transactional invariants, delivery semantics) where the service has them. The prose closes with
   `### Flow`, ahead of the endpoint lists.
4. **`### Flow`**. Under Overview, before `### Use cases`. Embeds
   `docs/diagrams/<service-name>-flow.svg`, the communication view: the services that call this one
   on the left, each with what it asks for, the service at the center with its use-case groups, the
   peers it calls during a request on the right in call order, the database below, and, when the
   service publishes events, the asynchronous chain from the outbox to each consumer, every step
   numbered in execution order. When the service publishes or consumes domain events it also embeds
   `docs/diagrams/<service-name>-events.svg`, the EventStorming view (the command, the aggregate,
   and the event it raises, then the policy and the command each consumer reacts with), followed by
   a one-line caption saying what that view shows. The subsection closes with the sources line, as
   real Markdown links to the HTML files, one per embedded diagram:
   `Sources: [<service-name>-flow.html](docs/diagrams/<service-name>-flow.html), [<service-name>-events.html](docs/diagrams/<service-name>-events.html).`
   The HTML file is the source of each diagram. The SVG is exported from it with the workspace
   export helper, which outlines every text as glyph paths because the repository host serves
   README images without loading fonts, and it is never edited by hand: change the HTML and export
   again. There is no `### Events` subsection. The events, the commands that raise them, and the
   consumers that react to them live in the diagrams.
5. **`### Use cases`**. Under Overview. Bulleted list. Each item is a Markdown link to the
   corresponding section anchor inside `docs/USE_CASES.md`. One bullet per write-side endpoint.
6. **`### Queries`**. Under Overview. Bulleted list. Each item is a Markdown link to the
   corresponding section anchor inside `docs/QUERIES.md`. One bullet per domain read-side endpoint.
   Omit this subsection entirely when the service has no domain read endpoint.
7. **`### Health checks`**. Under Overview. Bulleted list. Each item is a Markdown link to the
   corresponding section anchor inside `docs/HEALTH.md`. Covers liveness, readiness, startup probes,
   and observability endpoints. Kept separate from `### Queries` because these are operational
   contracts, not domain reads.
8. **`### Webhooks`**. Under Overview. Bulleted list. Each item is a Markdown link to the
   corresponding section anchor inside `docs/WEBHOOKS.md`. One bullet per provider-event pair the
   service consumes. Omit this subsection entirely when the service has no webhook endpoints.
9. **`## Installation`**. `### Repository` (the clone command), then one subsection per `make help`
   group, in this order: `### Configuration` (the setup group), `### Tests` (the test group),
   `### Review` (the review group), `### Reports` (the reports group). Each subsection lists every
   target its group shows in `make help`, never a subset, each with a one-line sentence and a Bash
   snippet carrying the `make` command. Read the targets from `make help` (the `## @<group>`
   annotations in the `Makefile`), not from this template. Close the section with the blockquote
   note pointing readers to `make help` for the full target list, which covers the maintenance and
   help groups.
10. **`## Environment setup`**. Tabular reference. Two subsections. `### Access URLs` (`Environment`
    and `DNS` columns), one row per environment (`Local`, `Development`, `Production` as
    applicable). The `Local` row mirrors the compose host rule. Each cloud row carries the hostname
    the infrastructure stack under `<deploy-target>` gives the service: its public record, or, when
    the stack publishes none, the private name the sibling services resolve (the service name under
    the DNS zone of the stack). With `<deploy-target>` absent there is no cloud row. When the public
    hostname answers only `/health/*` (an internal service, whose API is reached only inside the
    private network), a note under the table says so and says how the sibling services reach the API.
    `### Environment variables`, a table of the variables the service reads with exactly these
    columns in this order: `Variable`, `Description`, `Development value`. One row per variable. The
    `Development value` column carries only non-sensitive local defaults (a value like `local`,
    `true`, a docker service name, a `localhost` URL, or a port). For anything sensitive (real
    credentials, API keys, tokens, private keys, certificates, production hosts, or connection
    strings with real endpoints) the cell carries a dash placeholder and the `Description` says how
    to obtain the value.
    When unsure whether a value is sensitive, treat it as sensitive. Source the rows only from the
    tracked configuration (this repository ships no `.env.example`, which `skeleton-layout` forbids,
    so read the committed `docker-compose.yml` and the variable surface the service loads from
    `.env.local`). Never infer a variable from framework defaults. Leave out any variable that no
    tracked file confirms. State where values live (locally in `.env.local`, which a proof of
    concept commits per `skeleton-layout` § `.env` strategy, and in the cloud in the platform secret
    store where there is one) without pasting any secret.
11. **`## HTTP contract`**. Optional, and only for a gateway (a service whose public hostname serves
    its API). It states what the gateway adds over the contracts of the services behind it, such as
    the client channels, the trusted edge, and the request and response headers it reads and writes,
    and closes with `### Error codes`, a table (`Code`, `Status`, `Meaning`) of every code the
    gateway produces by itself. The endpoint contract itself stays in `openapi.yaml` and the docs
    pages. An internal service has no such section.

Every error code the service code can produce is documented, in `openapi.yaml`, in the docs page
response blocks, and in the README error table where the README carries one. That includes the
codes no handler raises: the router answers a known path under another method with `405`, and a
gateway whose route names an upstream it cannot resolve answers `500` with
`UPSTREAM_SERVICE_UNRESOLVED`. Each one is listed wherever the service produces it.

## Worked example (the section shapes filled in)

Everything inside the block below is illustrative, not canonical: the H1, the endpoint names, the
Overview prose, the Access URLs table, and the Environment variables table. Only the section order,
the Flow shape, and the table columns are fixed. The real content comes from the contract (the spec
at `<spec-root>` where it is set), from the contract rendered in the docs pages, and from this
repository's own tracked configuration. With `<local-proxy>` present, the `Local` DNS row mirrors
the proxy host rule in the committed `docker-compose.yml` (`<service-name>.<local-domain>`), and
with `<local-domain>` unset the host rule falls back to `<service-name>.localhost`. With
`<local-proxy>` absent there is no proxy at all, so the row carries the port the app publishes
directly.

````Markdown
# <Service name>

* [Overview](#overview)
    - [Flow](#flow)
    - [Use cases](#use-cases)
    - [Health checks](#health-checks)
* [Installation](#installation)
    - [Repository](#repository)
    - [Configuration](#configuration)
    - [Tests](#tests)
    - [Review](#review)
    - [Reports](#reports)
* [Environment setup](#environment-setup)
    - [Access URLs](#access-urls)
    - [Environment variables](#environment-variables)

## Overview

<One or two paragraphs saying what the service centralizes and the problem it solves.>

### Flow

![<Service name> communication](docs/diagrams/<service-name>-flow.svg)

Sources: [<service-name>-flow.html](docs/diagrams/<service-name>-flow.html).

### Use cases

- [<Endpoint name>](docs/USE_CASES.md#<anchor-slug>)

### Health checks

- [Liveness check](docs/HEALTH.md#liveness-check)
- [Readiness check](docs/HEALTH.md#readiness-check)

## Installation

### Repository

To clone the repository using the command line, run:

```bash
git clone https://github.com/<github-org>/<service-name>.git
```

### Configuration

To install project dependencies locally, run:

```bash
make configure
```

To update dependencies to their latest compatible release, run:

```bash
make configure-and-update
```

To start the application containers, run:

```bash
make start
```

To stop the application containers and drop the data volume, run:

```bash
make stop
```

### Tests

Run all tests with coverage and mutation testing:

```bash
make tests
```

Run a specific test file without coverage:

```bash
make test-file FILE=<name>
```

### Review

Run static code analysis:

```bash
make review
```

Fix what static code analysis can correct:

```bash
make fix-review
```

### Reports

Open coverage and mutation reports in the browser:

```bash
make show-reports
```

Show outdated dependencies:

```bash
make show-outdated
```

> You can check other available commands by running `make help`.

## Environment setup

### Access URLs

| Environment | DNS                                    |
|:------------|:---------------------------------------|
| `Local`     | http://<service-name>.localhost:<port> |

### Environment variables

Every variable the service reads. Values live in `.env.local` locally, and a deployed environment supplies its own
through the container environment. Sensitive values are never committed here: the `Development value` column carries a
dash placeholder for them and the `Description` says how to obtain each.

| Variable            | Description                                                | Development value         |
|:--------------------|:-----------------------------------------------------------|:--------------------------|
| `HTTP_ADDRESS`      | Address and port the HTTP server binds to                  | `0.0.0.0:8080`            |
| `DATABASE_HOST`     | Database host (docker service name)                        | `<service-name>-adm`      |
| `DATABASE_PORT`     | Database port inside the compose network                   | `5432`                    |
| `DATABASE_NAME`     | Database name                                              | `<service_name>_adm`      |
| `DATABASE_USER`     | Database user, local value provisioned in `.env.local`     | `—`                       |
| `DATABASE_PASSWORD` | Database password, local value provisioned in `.env.local` | `—`                       |
````
`````
