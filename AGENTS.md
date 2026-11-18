# Congestion tax

Rust microservice. Hexagonal architecture (ports & adapters), DDD, CQRS. One bounded context per repository (polyrepo).
Bounded context: congestion taxation (the tax a vehicle owes, per day and in total, for its passages through the
tolling stations of a city, under tax rules each city keeps as content in PostgreSQL).

The sections down to the divider belong to this project. Everything below the divider is the toolkit contract that the
rules, skills, and hooks in `.claude/` read, ported from the owner's PHP services to Rust. Nothing is configured,
because nothing is declared: every value the toolkit needs it reads from the project, per § Reading the project.

## Domain vocabulary

- **Passage**: the local date and time at which a vehicle passed a tolling station of the city.
- **Tax rules**: the currency of a city and its tax periods, which never overlap.
- **Tax period**: the rules in force between two dates, both included: tariff, calendar, single charge window, daily
  maximum, and exempt vehicles.
- **Tariff band**: the fee a passage costs between two local times of the day, start included and end excluded.
- **Single charge window**: the minutes after the first charged passage during which only the highest fee is charged. A
  free passage never opens a window.
- **Toll free calendar**: weekdays, months, public holidays and, optionally, the day before each holiday, on which no
  passage is charged.
- **Daily maximum**: the most a vehicle pays in one day, applied per day after the single charge rule.
- **Tax assessment**: the tax of each day of the passages, toll free days at zero, and their total.

## Bounded contexts

- `congestion taxation`: calculates the tax due from the rules of a city. Lives in `src/`.

## Project-specific rules

- Dependencies take their latest release, with no cooldown. See § Dependency policy.
- Tax rules are content, not code. A new city, tariff, or year of holidays is a set of Flyway migrations under
  `database/migrations/`, one insert file per table (tax rules, tax periods, tariff bands, public holidays, exempt
  vehicles), because the migration rule allows one seed per file. Never a constant in `src/`.
- Amounts are whole units of the currency of the city (`u32`), because every tariff the service knows charges whole
  kronor.
- One type per file. Every struct, enum, trait, and type alias lives in its own file named after it in `snake_case`,
  private helpers and test doubles included, so a request, a response, a decoded or stored shape, and a value a spy
  answers with each get a file. An associated type inside an `impl` (`type Error`) is not a separate type.
- No comments in Rust code beyond the `///` doc comments on port traits and their methods, and the `// Given`,
  `// And`, `// When`, and `// Then` markers every test carries (`rust-testing` § One When per test).
- Every line of a Rust file fits in 120 characters, the rustfmt `max_width`, and `make review` fails above it.
- The composition root is the binary: `src/main.rs` wires the graph, and the modules it declares read the configuration
  once, typed, failing at startup (`app_settings.rs`, `database_settings.rs`, `environment_variable.rs`). In `src/`,
  only those files read the environment. In `tests/`, only `support/fixtures/database_fixtures.rs` reads it, for the
  test database coordinates the `Makefile` passes. `src/routes.rs` mounts the driver and the query slices on one router.
- `tests/main.rs` is the only test target (`autotests = false`). Add a test file by declaring its module there, in the
  folder that mirrors `src/`.
- No AI attribution anywhere: no co-author trailer, session link, or "generated with" footer in a commit, a pull
  request, an issue, a review, or release notes.

---

## Reading the project

The toolkit writes `<token>` wherever a value belongs to the project rather than to the toolkit, and it reads every one
of them from the project itself. Nothing is declared, because a declared value is a value that can silently disagree
with the file the tooling actually resolves against, and because a toolkit that has to be configured is a toolkit that
gets adopted wrong.

Read each one from its source, at the moment you need it. In nearly every case the rule that names a token is already
looking at the file that answers it. The third column is what the source answers today, given so a reader can check a
reading, and the source wins whenever the two disagree. A token marked absent is a normal state (§ Capability absence).

| Token                   | Read from                                                                                     | This project                                                                  |
|-------------------------|-----------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------|
| `<vendor>`              | the `org.opencontainers.image.title` label of the app Dockerfile, the segment before the slash | `gustavofreze`                                                                |
| `<service-name>`        | `Cargo.toml` `[package] name`                                                                 | `congestion-tax`                                                              |
| `<RootNamespace>`       | `Cargo.toml` `[lib] name`, else the package name with hyphens as underscores                   | `congestion_tax`, written `crate::` inside the crate                          |
| `<runtime-minor>`       | `rust-toolchain.toml` `channel`, the patch dropped (`Cargo.toml` `rust-version` agrees)        | `1.100`                                                                       |
| `<github-org>`          | `git remote get-url origin`, the owner segment                                                | absent, no remote is configured yet (the README clone URL names `gustavofreze`) |
| `<database-engine>`     | the sqlx driver feature in `Cargo.toml`, else the database image in compose                   | `postgresql` (`postgres` feature, `postgres:18.7`)                            |
| `<spec-root>`           | the sibling directory named `specifications`, absent when there is none beside the repo       | absent, contract facts come from `openapi.yaml` and the docs pages            |
| `<base-images>`         | the namespace in the app Dockerfile `FROM` line, resolved per family, absent per family       | `gustavofreze` for the Rust family, absent for the Flyway and PostgreSQL ones |
| `<base-images-dir>`     | the base-image checkout the `Makefile` names, absent when it names none                       | absent                                                                        |
| `<ci-workflows>`        | the `uses:` target of the caller workflow, absent when no workflow calls out                  | absent, `ci.yml` carries the whole workflow                                   |
| `<platform-iac>`        | the module `source` in the infrastructure stack, absent when there is no stack                | absent                                                                        |
| `<deploy-target>`       | the infrastructure subtree at the repository root, absent when there is none                  | absent                                                                        |
| `<cloud-region>`        | the region in the infrastructure provider and backend blocks                                  | absent, there is no infrastructure stack                                      |
| `<local-network>`       | the external network declared in `docker-compose.yml`, absent when none is                    | absent, the stack runs on its compose default network                         |
| `<local-domain>`        | the host suffix in the compose reverse-proxy label, absent when there is no label             | absent                                                                        |
| `<local-proxy>`         | the proxy checkout the `Makefile` names, absent when it names none                            | absent, the app publishes port 8380 directly                                  |
| `<auth-package>`        | the crate supplying the auth middleware `src/routes.rs` wires, absent when none               | absent                                                                        |
| `<idempotency-package>` | the crate supplying the idempotency middleware, absent when none                              | absent                                                                        |
| `<messaging-transport>` | the broker the outbox publishes to or the dispatch worker drains, absent if neither exists    | absent, the service has no outbox and no dispatch worker                      |

`<RootNamespace>` is read whole, never composed of a vendor part and a service part. It need not contain `<vendor>`.
Composing it breaks.

Two values that look like they belong here do not, because they are not project values at all. A secret store, a public
apex domain, a CI secret name, and a tenant discriminator column are named where they are used, in the rule that needs
them and in the file that holds them, rather than bound centrally. And this project applies no dependency cooldown, so
no vendor needs an exemption from one. See § Dependency policy.

### Where a token sits changes what it means

- **In a code block, a file template, or a filename.** Everything under `.claude/skills/*/assets/`, the code samples in
  `.claude/skills/*/references/`, and a path like `<service-name>-migrate`. The token is a hole. Fill it when the file
  is written, and never let it survive into a written file. Absence never belongs here: a sentence that only reads
  correctly with the token unfilled is nonsense once it is filled. Put that sentence in a rule or a skill instead.
- **In prose.** A rule sentence, a checklist item, a skill instruction. The token is a reference, resolved as you read
  the sentence, from the table above. It stays in the file forever. Never run a substitution pass over `.claude/`, and
  never commit a rule with a resolved value pasted into it.
- **In YAML frontmatter, never.** A rule's `description:` and a skill's `description:` are read by the tooling before
  any of this is in context, so a token there reaches the reader raw. Write the plain word and bind it in the body.

A worked example may print the resolved value beside the token, as `<database-engine>` (`postgresql`), to show which
spelling a reader on another engine has to translate. That is the one place both appear together.

Case mirrors the value: `<service-name>` kebab, `<service_name>` snake, `<Service name>` sentence, `<SERVICE_NAME>`
screaming snake. One concept, one spelling per case. Never introduce a second spelling of a token that already exists,
because a reader cannot tell a variant from a new concept.

This file reaches every subagent through `CLAUDE.md`, because a subagent loads the whole CLAUDE.md hierarchy. The
built-in Explore and Plan agents are the exception and skip it, so when you delegate to one of those and the task turns
on a token, restate it in the task text.

## Capability absence

A capability the project does not state is absent. That is a normal state and never an error. Do not invent a value, do
not substitute a plausible one, and do not treat the dependent rule as unenforceable.

Every rule that names a capability states inline what holds when it is absent, because
**an invariant is stated about the artifact and never about who supplies it**. Read a rule sentence as two claims. In
`inherits FROM a <base-images> image, never latest`, the floor against `latest` is the invariant and the namespace is
the binding.
With no `<base-images>` the floor still holds and the namespace is the upstream official image.

The same split applies to a prohibition. A rule that forbids restating what a base image already provides is conditional
on a base actually providing it. With no base, the app restates those directives itself and the prohibition does not
apply.

## Commands

Every Cargo command runs inside Docker through the `Makefile`, in the `<base-images>/rust` `cli` image, so nothing
beyond Docker and GNU Make is needed on the host. The `Makefile` recipes are the single source of truth for what each
target runs. Run `make help` for the actual list.

The target set below is the invariant `rust-skeleton-tooling` enforces, not a fact about the working copy in front of
you. Verify a target exists before invoking it, and where one is missing, use the project's nearest equivalent and
report the gap rather than concluding the command is unavailable.

- `make start`, `make stop`. Start the local stack (the app, PostgreSQL, and the one-shot Flyway run), and stop it,
  dropping its data volume.
- `make configure`. Fetch the dependencies from `Cargo.lock`.
- `make review`. No Rust line above the rustfmt `max_width`, no import from the domain to anything but `std`, itself,
  `jiff`, and `thiserror`, then `cargo sort --check` on `Cargo.toml`, `cargo fmt --check`, and Clippy with every
  warning denied. The `[lints]` table of `Cargo.toml` enables pedantic, a few nursery lints, and the restriction lints
  that turn the code rules into checks (no `unwrap`, `expect`, `panic`, or `#[allow]` in production code, no
  unreachable `pub`, bounded nesting). `clippy.toml` holds the thresholds and lifts the panic family for the test
  target. `make fix-review` autofixes what those tools can.
- `make tests`. Starts PostgreSQL, applies the Flyway migrations to a dedicated test database, and runs the whole suite
  with `cargo llvm-cov`, failing below 100 percent of lines, then `cargo mutants`. The composition root is excluded from
  both (§ Project-specific rules). Requires Docker.
- `make test-file FILE=<name>`. Runs the tests whose path matches, without coverage, migrating the test database first.

These commands are the definition of done. Run them before claiming a change works.

## Architecture

The hexagon lives under `src/`, four layers (`application/`, `driver/`, `driven/`, `query/`), dependencies pointing
inward only. The layer contract, folder skeleton, and dependency rule are owned by `rust-architecture`. Read that rule
before creating any file under `src/`. Do not restate it here.

The HTTP contract lives in `docs/` mirrored in `openapi.yaml` (synchronization owned by `web-documentation`). Schema
changes are forward-only Flyway files under `database/migrations/` (owned by `database-migration`).

## Dependency policy

Dependencies take their latest release, including same-day releases, with no cooldown, for every dependency manager
used in this repository (`cargo`). This is the owner's decision for this project, so the 7-day cooldown and its vendor
exemptions that the owner's PHP services carry do not apply here.

GitHub Actions follow the same policy. An action is pinned by commit SHA to its latest stable release, whatever its
release date (`.claude/rules/github-workflows.md` item 23), and its Dependabot entry carries no `cooldown`.

## Global defaults

These apply across every file in the repository, regardless of path.

- **Prose punctuation.** Do not use `;`, ` — ` (em-dash), ` – ` (en-dash), or ` -- ` as clause separators in prose,
  Rust doc comments (`src/**/*.rs`), shell comments (`**/*.sh`), or Markdown. Use two sentences, a comma, a colon, or
  parentheses instead. Hyphens in compound identifiers and Markdown tables are exempt.
- **No inline analyzer silencing.** `#[allow]` in any form is prohibited in every Rust file, and the
  `allow_attributes` lint makes that a check. A lint concession lives in the `[lints]` table of `Cargo.toml` or in
  `clippy.toml`, each entry carrying the comment that justifies it, or, for one item that cannot be fixed in the
  source, in `#[expect(lint, reason = "...")]` on the narrowest item. `rust-code-style` § Analyzer suppressions and
  `rust-skeleton-tooling` own the detail.
- **Secrets in versioned files.** Never commit real secret values. Every tracked file carries placeholders or local-only
  defaults, never production credentials, API keys, tokens, private keys, certificates, connection strings pointing at
  real hosts, or customer data. This covers `README.md` and everything under `docs/` (the Postman environment
  included), `docker-compose.yml`, `Dockerfile*`, `docker/`, `Makefile`, CI configuration (`.github/workflows/*`), seed
  migrations, and test data under `tests/`. It covers `.env.local` too, which this proof of concept commits
  (`skeleton-layout` § `.env` strategy), so that file is bound by this rule like any other tracked file and not
  exempted by it. A real value lives in the project's secret store, and a project with no secret store has nowhere to
  put one: it reaches the container through the deployed environment and never through a file in this repository.

## Conventions

Rules (`.claude/rules/*.md`) are persistent invariants auto-injected when you read a file matching their `paths:` glob.
Skills (`.claude/skills/<name>/SKILL.md`) are intent-triggered workflows for producing new artifacts. Rules and skills
partition by type and do not conflict with each other. A rule is an invariant (what must always hold), a skill is a
workflow (how to build an artifact within those invariants), and each concern has a single owner. A skill applies and
defers to the rules covering the files it touches, it never overrides them. Artifact names use a stack prefix (`rust-`,
`php-`, `go-`, `tf-`, `web-`). Cross-cutting infrastructure, database, documentation, and version-control artifacts
(for example `creating-migration`, `database-migration`, `docker-images`, `shell-scripts`, `writing-commit-messages`)
are stack-neutral and carry no prefix.

No file under `.claude/` may carry an organization name, a registry host, a sibling directory path, a cloud region, or a
public domain. That is what makes the tree copy-paste portable, and it is checkable with one grep for the values this
project resolves to.

### Read the rule before creating a new file

Claude Code bug (anthropics/claude-code#23478): path-scoped rules are injected on **Read** but NOT on **Write/Edit** a
new file. Before creating any file under a glob covered by a rule, explicitly read the relevant `.claude/rules/*.md`
file. Prefer running the matching skill when creating new artifacts (endpoint, handler, repository, query slice,
migration, commit).

### Authority and conflict resolution

Always follow the rules (`.claude/rules/*.md`) and skills (`.claude/skills/<name>/SKILL.md`). They are the default
authority for every task. Only two artifacts override them, each within its own domain:

1. **On business rules, the spec wins.** For domain behavior, data models, aggregate boundaries, and domain events, the
   specifications at `<spec-root>` outrank every rule, skill, and ADR, read directly where it resolves, never by
   guessing a path. With no `<spec-root>` there is no spec tier: rules and skills become the top authority and carry a
   stated default for every topic a spec would own, which a spec later overrides where it speaks. Either way the
   prohibition that matters survives. Never invent domain behavior, stop and ask.
2. **On technical decisions, an ADR wins** for the situation it addresses, over any rule or skill. ADRs live in
   `docs/adrs/` (service-specific) and, where `<spec-root>` resolves, at `<spec-root>/adrs/` (cross-cutting). Never
   create one without an explicit request.

Outside those two cases, rules and skills always apply, and they do not compete with each other. If two rules overlap,
favor the more specific one. Flag any real contradiction, between two rules or between an ADR and a rule, before
deviating from either.
