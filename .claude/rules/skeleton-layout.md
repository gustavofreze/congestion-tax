---
description: Repository root layout, .env strategy, docker/ substructure, and per-environment asset naming for a Rust service repository.
paths:
    - "docker-compose.yml"
    - "Dockerfile*"
    - "docker/**"
    - ".env*"
    - ".gitignore"
---
<!-- Vendored from payment (.claude/rules/skeleton-layout.md) on 2026-11-18. Adapted to this project: the root tree is this repository's Rust layout, the .env strategy and the per-family docker/ subtrees follow account because this proof of concept commits .env.local, docker/postgres/initdb.d/ replaces the MySQL, PHP, nginx, and dispatch subtrees, and the paths globs keep only the files this project has. -->

# Project layout

Defines the layout of the repository at the root level: which files and directories live at the root, the `.env`
strategy, the `docker/` substructure, and per-environment asset naming. The contents of `src/` and `tests/` (Rust code,
owned by `rust-architecture` and the `rust-testing` rules), `database/migrations/` (SQL migrations),
`.github/workflows/` (CI workflow files), the root tool-configuration files (`Cargo.toml`, `clippy.toml`,
`rust-toolchain.toml`, `rustfmt.toml`, `.cargo/mutants.toml`, `Makefile`, `.editorconfig`, owned by
`rust-skeleton-tooling`), and the README and `docs/` tree are each governed by their own rules, not this one.
Dockerfile bodies, dockerignore content, and base-image inheritance are owned by the `docker-images` rule.

## Pre-output checklist

1. Every file and directory sits where the canonical layout places it, and no prohibited root path (`bin/`, `scripts/`,
   `config/`, `.env.example`, `.env.secrets`, `database/init/`, `database/scripts/`) is present. See § Repository root
   canonical layout and § Prohibited paths and patterns.
2. The single environment file is `.env.local`, it is committed, and it carries local-only defaults and no real
   credential. No other environment file exists, verified by filtering `git ls-files` to the `.env` names and getting
   that one line back. See § `.env` strategy.
3. Container assets group by containerized service under `docker/`, and every subtree (`postgres/`, `flyway/`) exists
   exactly when it has a file of its own to carry, none of them mandatory. `<base-images>` resolves per family, so a
   family that namespace does not publish leaves its subtree expected when that container needs a file. See §
   `docker/` substructure.
4. Any asset whose content differs by environment carries a `.dev` or `.prod` suffix before the extension, never a bare
   basename. See § Per-environment asset naming.

## Repository root canonical layout

```
<repo>/
├── .dockerignore                        # Build-context filter, shared by both Dockerfiles.
├── .editorconfig
├── .env.local                           # Committed. Local-only defaults, never a real credential.
├── .gitattributes
├── .gitconfig
├── .gitignore
├── AGENTS.md                            # Project instructions, imported by CLAUDE.md.
├── Cargo.lock                           # Committed.
├── Cargo.toml
├── CLAUDE.md                            # One line, the @AGENTS.md import.
├── clippy.toml                          # Clippy thresholds and the allowances of the test target.
├── docker-compose.yml
├── Dockerfile.dev
├── Dockerfile.prod
├── LICENSE
├── Makefile
├── openapi.yaml
├── README.md
├── rust-toolchain.toml
├── rustfmt.toml
├── .cargo/
│   └── mutants.toml                     # Mutation testing configuration.
├── .cargo-home/                         # Gitignored. Cargo dependency cache of the Makefile runners.
├── .claude/                             # Rules, skills, hooks, and settings for Claude Code.
├── .github/
│   ├── dependabot.yml
│   └── workflows/                       # CI workflow files.
├── .<vendor>/
│   └── pipeline.yml                     # Optional. Present only when a pipeline default must change.
├── database/
│   └── migrations/                      # Flyway SQL files.
├── docker/                              # When applicable. No subtree is mandatory.
│   ├── postgres/                        # When applicable. Here, initdb.d/ with the least-privilege user hook.
│   └── flyway/                          # When applicable (base override, or expected if <base-images> unset).
├── docs/                                # Documentation tree.
├── reports/                             # Gitignored. Analysis output (coverage, mutation).
├── src/                                 # Rust library and binary crate.
├── target/                              # Gitignored. Cargo build output.
└── tests/                               # The single integration test target.
```

No `bin/`. No `scripts/`. No `config/`. No `.env.example`. No `.env.secrets`. No `database/init/`. No
`database/scripts/`. A root `bin/` is distinct from Cargo's `src/bin/`, which is a source location and is allowed.

An infrastructure subtree (`ecs/`, with `development/` and `production/`) is present only when `<deploy-target>` is set,
and it keeps that directory name whatever the target is called (never renamed). With `<deploy-target>` unset the
repository carries no infrastructure subtree at all, and the skeleton gate skips those items instead of failing them.
The pipeline config file is `.<vendor>/pipeline.yml`, the canonical spelling owned by `github-workflows` (item 17), and
it is optional. With `<ci-workflows>` unset it is unnecessary, because the service repository owns its workflow
outright and an override is edited in place.

## `.env` strategy

Configuration via environment variables (12-Factor App, Factor III). Exactly one environment file exists, `.env.local`,
and it is committed.

### `.env.local` (committed, the single environment file)

Lives at the repository root, **tracked**. This service is a proof of concept, so the file ships with the repository
and a clone runs with no setup step. That is what the tracking buys, and the price is that
**the file may carry nothing but local-only defaults**: a container password the compose stack itself sets, a Compose
service name, a port, a schema name. A signing key, a third-party API token, a private certificate, or any value
pointing at a real host is forbidden in it, exactly as in every other tracked file (`AGENTS.md` § Global defaults), and
no gitignored sibling is introduced to hold one. A real value reaches a deployed container through that environment and
through the project's secret store, never through this repository.

The consequence to accept deliberately: a per-developer override now edits a tracked file, so it shows up as a working
tree change. That is the trade, and it is the right one only while the values stay worthless. The `creating-env-local`
skill produces the file when a service does not ship one yet. Header, verbatim, at the top:

```
# Local defaults for the proof of concept. Committed on purpose, so a clone runs with no setup step.
# Every value here is local-only and reaches nothing outside this machine. Never put a real credential in it.
```

The `README.md` "Environment setup" documents the full variable surface and, because the file is tracked, prints the
literal value of every key rather than hiding any of them behind a placeholder. Hiding a value that sits in a tracked
file two directories away is theatre, and a value that could not be printed had no business being in the file. The
`README.md` states that the repository is a proof of concept and that this is why the values are committed.

### Loading order

Docker Compose interpolation reads `.env.local` through the `--env-file .env.local` flag carried by the `Makefile`
compose invocations (the `COMPOSE` variable), never through an autoloaded `.env`. Services consume it via an explicit
`env_file:` entry marked `required: true`:

```yaml
services:
  app:
    env_file:
      - path: .env.local
        required: true
```

The flag is `true` because the file is committed. `required: false` accepts a missing file in silence and starts the
container with no variables at all, which the application then discovers deep inside itself when a required variable
reads as absent. That was the right trade only while the file was gitignored and a fresh clone legitimately had none. A
tracked file that is missing is a file someone deleted, so the run should stop at `docker compose` with the path named,
which is what `true` does.

Inside the application, variables come from the process environment only, which `env_file` fills in the container. The
binary never reads `.env.local` itself.

### Prohibited

- A bare `.env`. The single environment file is `.env.local`, and a parallel `.env` duplicates values and invites drift.
- `.env.example`. The variable surface is documented in the `README.md`, a parallel example file invites drift, and a
  tracked `.env.local` already shows every key with its value.
- `.env.secrets`. A secret has no home in this repository at all, so a file named for one is forbidden whether it is
  tracked or gitignored.
- Any environment file other than `.env.local`. Verify with `git ls-files`, filtered to the `.env` names: `.env.local`
  is the only line it may print.

## `docker/` substructure

The `docker/` directory groups container assets by containerized service. Each subtree carries the files that the
corresponding container consumes via volume mounts, COPY directives, or init hooks, and each exists exactly when it has
a file to carry. None is mandatory, because git holds no empty directory: a subtree with nothing service-specific in it
is a subtree that cannot exist, so demanding one would be a rule no repository could satisfy.

**`<base-images>` resolves per family, not once for the whole namespace.** It answers "who publishes the base for this
container", and the answer differs by container. A namespace that publishes a Rust base and no Flyway base leaves the
Flyway side in the same state as a project with no namespace at all, which is what `AGENTS.md` § Capability absence
describes: the binding is absent for that family, and the rule's unset branch is the one that applies to it. Read each
subsection below against the family it names, never against whether the namespace exists at all.

The app container needs no subtree while a `<base-images>` Rust base exists: that base carries the non-root user, the
toolchain, and the runtime filesystem, and the app Dockerfile declares its own health check, port, and command
(`docker-images` § What the base provides).

### `docker/postgres/initdb.d/` (when applicable)

Present only when the service operates its own PostgreSQL container. Holds bootstrap scripts executed once on first
container start by the PostgreSQL image's `docker-entrypoint-initdb.d` hook, mounted read-only by `docker-compose.yml`.
Its scripts follow `shell-scripts` § Carve-out. Schema migrations never live here, they belong to
`database/migrations/` and run via Flyway.

### `docker/flyway/` (when applicable)

Where a `<base-images>` Flyway base exists, the migration entrypoint lives in that image (`<base-images>/flyway`,
docker-images rule), consumed directly by the compose file with the Flyway environment variables, and this directory
exists only when the service deliberately overrides the baked entrypoint, which is the exception. Where none exists,
whether because the namespace publishes no Flyway family or because there is no namespace, the default inverts: the
upstream official Flyway image bakes no such entrypoint, so this directory is expected unless the migrate service
drives the image through its own flags alone, which needs no entrypoint and therefore no directory.

## Per-environment asset naming

Files that vary between development and production use a suffix BEFORE the file extension:

| File kind  | Development      | Production        |
|:-----------|:-----------------|:------------------|
| Dockerfile | `Dockerfile.dev` | `Dockerfile.prod` |

The suffix `.dev` or `.prod` is the second-to-last segment of the basename. A bare basename without an environment
suffix is prohibited for any asset whose content differs by environment. Files identical across environments do NOT
carry a suffix (e.g., `generate-certificates.sh`).

## Prohibited paths and patterns

The patterns below are prohibited at the repository root. Each has a reason rooted in an external standard or in
repeated drift observed across services.

| Pattern                               | Reason                                                                                                                                                                               |
|:--------------------------------------|:-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `bin/`                                | Operational entrypoints live in the `Makefile` (workflow shortcuts) or in a container's own directory under `docker/` (container-specific scripts).                                  |
| `scripts/` (at root)                  | Workflow shortcuts live in the `Makefile`. Container-specific scripts live under the `scripts/` subdirectory of that same container directory (permitted, owned by their container). |
| `config/`                             | 12-Factor App (Factor III), configuration is read from environment variables, not from per-environment files committed to the repo.                                                  |
| `config/<env>/`                       | Same as above.                                                                                                                                                                       |
| `.env.example`                        | The variable surface is documented in the `README.md`. A parallel example file invites drift.                                                                                        |
| `.env.secrets`                        | A secret has no home in this repository at all, so a file named for one is forbidden whether it is tracked or gitignored.                                                            |
| Bare `Dockerfile` (no `.dev`/`.prod`) | Every Dockerfile is environment-specific. The naming convention makes the target explicit at the `docker build -f` call site.                                                        |
| `database/init/`                      | Bootstrap SQL belongs to the database container, not to the schema migration history. It lives in `docker/postgres/initdb.d/`.                                                       |
| `database/scripts/`                   | Migration entrypoint scripts belong to the migration container, in its base image or under `docker/flyway/`.                                                                         |
| `report/` (singular)                  | Analysis output is `reports/` (plural). Tooling defaults and the rest of the rule bundle assume the plural.                                                                          |
| Committed `*.pem`, `*.key`, `*.crt`   | OWASP universal rule. Private keys and certificates of any kind are never committed. Generators (`generate-*.sh`) live in the directory of the container that uses them.             |
