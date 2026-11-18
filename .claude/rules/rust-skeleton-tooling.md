---
description: Cross-file and audit invariants for the service's tooling files (Cargo.toml, Cargo.lock, rust-toolchain.toml, rustfmt.toml, clippy.toml, .cargo/mutants.toml, .editorconfig, .gitattributes, Makefile).
paths:
    - "Cargo.toml"
    - "Cargo.lock"
    - "rust-toolchain.toml"
    - "rustfmt.toml"
    - "clippy.toml"
    - ".cargo/mutants.toml"
    - ".editorconfig"
    - ".gitattributes"
    - "Makefile"
---
<!-- Vendored from payment (.claude/rules/php-skeleton-tooling.md) on 2026-11-18. Adapted to this project: composer.json, phpunit.xml, phpcs.xml, phpstan.neon, and infection.json.dist mapped to Cargo.toml, rust-toolchain.toml, rustfmt.toml, clippy.toml, and .cargo/mutants.toml, the six composer scripts folded into the Makefile targets, the PHP library tables replaced by the crate set of Cargo.toml, phpstan ignoreErrors mapped to the `[lints]` table and `#[expect]`, and the per-thread schemas, diff-only mutation, tests-full target, and private package authentication dropped because this repository has none of them. -->

# Tooling

Cross-file invariants that the tooling files of the service must satisfy together, and the audit signals checked
against them. The skeleton skill that generates the canonical PHP bodies is not ported, so the files in this repository
are the reference, and this rule states what must hold across them.

Repository root layout and the `docker/`, `.env`, and Dockerfile assets are owned by `skeleton-layout`.

## Pre-output checklist

1. The repository root contains all of: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `rustfmt.toml`,
   `clippy.toml`, `.editorconfig`, `.gitattributes`, `Makefile`, and `.cargo/mutants.toml` beside them. See § Config
   files.
2. `rust-toolchain.toml` `channel` is the single source of the Rust `major.minor`. The `Makefile` derives it,
   `Cargo.toml` `rust-version` and every Dockerfile `FROM` line carry the same literal. See § Rust version.
3. Every crate in `Cargo.toml` is used by the code that depends on it, the domain crates stay within the `Makefile`
   whitelist, and `cargo sort --check` passes. See § Dependencies.
4. `Cargo.lock` is committed and every Cargo command that resolves or builds runs with `--locked`. See § Dependencies.
5. No dependency lags behind its latest release. See § Audit signal.
6. The coverage gate is `--fail-under-lines 100`, `cargo mutants` runs over the whole source, and the exclusions of
   both name exactly the composition root. Lowering the gate or widening an exclusion is prohibited. See § Coverage
   and mutation thresholds.
7. All test artifact paths use `reports/` (plural), consistent across the `Makefile` targets, and each artifact sits in
   the subdirectory of the tool that writes it. See § reports consistency.
8. The `Makefile` carries the closed public target set, hides every helper target from `make help`, declares exactly
   two container runners, and leaves no unreferenced variable. See § Makefile shape.
9. A service that owns a schema removes its migration container as the last line of the `start` recipe, with no error
   tolerance. See § Migration container removal.
10. Every lint concession lives in the `[lints]` table of `Cargo.toml` or in `clippy.toml`, each entry preceded by the
    comment that justifies it, or in `#[expect(lint, reason = "...")]` on the narrowest item. No `#[allow]` appears in
    a Rust file. See § Clippy concessions.
11. `rustfmt.toml` `max_width` is the single source of the line width: the `Makefile` derives it, and `.editorconfig`
    `max_line_length` carries the same number. See § Line width.

## Config files

The config files all live at the repository root, except `.cargo/mutants.toml`, which cargo-mutants reads from
`.cargo/`. Every one is committed live. Cargo has no `.dist` convention, and none is introduced.

| File                  | Role                                                                                         |
|:----------------------|:---------------------------------------------------------------------------------------------|
| `Cargo.toml`          | package, the library and binary targets, the single test target, dependencies, `[lints]`      |
| `Cargo.lock`          | the resolved dependency graph, committed because the package ships a binary                  |
| `rust-toolchain.toml` | the toolchain `channel`                                                                      |
| `rustfmt.toml`        | the formatter settings, `max_width = 120` among them                                         |
| `clippy.toml`         | the Clippy thresholds and the lifts for the test target                                      |
| `.cargo/mutants.toml` | the cargo-mutants exclusions and timeout                                                     |
| `.editorconfig`       | the editor defaults (charset, line ending, indentation, `max_line_length`)                   |
| `.gitattributes`      | line-ending normalization, diff drivers, and the generated or binary markers (`Cargo.lock`)  |
| `Makefile`            | the container-first entry point to every Cargo command                                      |

`Cargo.toml` sets `autotests = false` and declares one `[[test]]` target, `tests/main.rs`, so every test module is
reached from that file and compiled once. `[profile.release]` carries the size and speed settings of the production
binary. The canonical body of each file is the file in this repository.

## Rust version

`rust-toolchain.toml` `channel` is the single source of the Rust version, and its `major.minor` is
`<runtime-minor>`. The `Makefile` derives it into `RUST_VERSION` and never pins it, so `RUST_IMAGE` follows the
declaration on its own. Three places still carry a literal `major.minor`: `Cargo.toml` `rust-version`, the `FROM` line
of `Dockerfile.dev`, and the two `FROM` lines of `Dockerfile.prod`, because neither Cargo metadata nor a Dockerfile can
read `rust-toolchain.toml`. A Rust bump therefore touches the `channel`, `rust-version`, and every `FROM` line, and all
of them must agree. The `FROM` lines reference the shared base images published under `<base-images>`
(`<base-images>/rust:<runtime-minor>-development`, `-builder`, `-runtime`, and the `-cli` image the `Makefile` runs),
whose tags encode the Rust `major.minor`, so a Rust bump also requires a base image release (docker-images rule). With
`<base-images>` unset the `FROM` lines reference the upstream official `rust` image, whose tag encodes the same
`major.minor`. No base image release is involved then, and the invariant is untouched: the `channel`, `rust-version`,
and every `FROM` line carry one `major.minor` and must agree.

## Dependencies

The crate set is what the code uses, and nothing else. Each crate in `[dependencies]` and `[dev-dependencies]` is
imported by the code that depends on it, and a crate with no consumer is removed. The set today:

| Concern                  | Crate                          | Used by                                               |
|:-------------------------|:-------------------------------|:------------------------------------------------------|
| HTTP framework           | `axum`                         | `src/driver/http/`, `src/query/health/http/`, routes  |
| Async runtime            | `tokio`                        | `src/main.rs` and every async adapter                 |
| Database access          | `sqlx` (`postgres` feature)    | `src/driven/postgres/`, `src/query/health/database/`  |
| Time                     | `jiff`                         | the domain and the decoding of the stored rules       |
| Errors                   | `thiserror`                    | every error type in the library                       |
| Startup errors           | `anyhow`                       | `src/main.rs` and its settings modules only           |
| Serialization            | `serde`                        | requests, responses, and the stored document          |
| Logging                  | `tracing`, `tracing-subscriber`| the driver and `src/main.rs`                          |
| Tests                    | `rstest`, `tower`, `http-body-util`, `serde_json` | `tests/` only                      |

Three invariants hold over the set:

1. The domain uses only `std`, itself, and the crates the `Makefile` whitelists in `DOMAIN_CRATES` (`jiff`,
   `thiserror`). `check-domain-dependencies` derives every other crate from `[dependencies]` and fails on any path into
   one of them from `src/application/domain/`, so adding a crate needs no edit to the check.
2. A crate that offers optional features is declared with `default-features = false` and the features the code uses
   (`axum`, `jiff`, `sqlx`, `tracing-subscriber`, `rstest`, `tower`), so the build compiles nothing the service does
   not call.
3. `Cargo.toml` stays sorted, which `cargo sort --check` enforces in `make review`.

Every crate comes from crates.io. A git dependency, a path dependency outside the repository, a `[patch]` section, or a
private registry is added only on an explicit request. `Cargo.lock` is committed, and every Cargo command in the
`Makefile` that resolves or builds runs with `--locked` (`configure`, `tests`, `test-file`, `review`), so a build never
resolves a graph different from the committed one. `make configure-and-update` is the one target that rewrites it.

## Audit signal

Dependencies take their latest release, with no cooldown (`AGENTS.md` § Dependency policy). The audit therefore checks
that every crate requirement admits the latest release published on crates.io. `make show-outdated` runs
`cargo update --dry-run --verbose`, which reports only the updates the declared requirements already admit, so a new
major is found by comparing each requirement with crates.io, and adopting it is a `Cargo.toml` edit in the same change
that adapts the code. A crate a major behind its latest release is a finding.

## Makefile shape

The public target set is closed: `start`, `stop`, `configure`, `configure-and-update`, `tests`, `test-file`, `review`,
`fix-review`, `show-reports`, `show-outdated`, `clean`, `help`. The README, CI, and the contributor workflow reference
these names, so renaming or removing any of them is prohibited, and a thirteenth public target is added only on an
explicit request. Each carries exactly one `## @<section>` tag, which is what places it in `make help`. The `Makefile`
recipes are the single source of truth for what each command runs.

`tests` runs the suite once under `cargo llvm-cov` with `COVERAGE_FILTER`, writes the HTML report, fails below 100
percent of lines with `cargo llvm-cov report --fail-under-lines 100`, and then runs
`cargo mutants --jobs $(MUTANTS_JOBS)` over the whole source. `MUTANTS_JOBS` is 2 on a developer machine, and CI raises
it to the 4 cores of its runner with `make tests MUTANTS_JOBS=4`. Each cargo-mutants job builds and tests its mutant in
a copy of the tree, so the jobs share only the test database, and the integration fixtures seed with
`ON CONFLICT DO NOTHING` so concurrent runs never break each other. Going back to one job to get around a shared state
is prohibited: the fix is the isolation, never the parallelism.
`test-file` runs `cargo test --locked -- $(FILE)` without coverage, after the same test database preparation.

A helper target carries no `## @<section>` tag, which keeps it out of `make help`. A precondition is named
`ensure-<subject>`: `ensure-test-database` starts the database container, recreates the test database, and applies the
migrations to it. It asserts one precondition and repairs it when the repair is cheap and unambiguous, otherwise it
fails with a message naming the command that fixes it. A guard never does the work its public target exists for, and
each public target lists only the guards its own recipe consumes. The steps of `review` that are not Cargo commands
(`check-line-length`, `check-domain-dependencies`) are helper targets too, listed as prerequisites of `review`. Prose
comments inside the file are limited to the header and the section banners, because every rationale belongs in this
rule instead.

The cargo tools the recipes need beyond the toolchain (`cargo-llvm-cov`, `cargo-mutants`, `cargo-sort`) are pinned in
`LLVM_COV_VERSION`, `MUTANTS_VERSION`, and `CARGO_SORT_VERSION` and installed on first use by `ENSURE_CARGO_TOOL` into
`CARGO_HOME`, so the image stays generic and the versions live in one file.

The compose stack the `start` target brings up joins the shared external Docker network `<local-network>`, so the
services of neighboring stacks reach this one by container name. A network declared `external` is never created by
compose, and `docker compose up` aborts outright when it does not exist, so that network already existing is a
precondition of `make start` and not something the compose file repairs. With `<local-network>` unset the compose file
declares no external network at all, the stack uses its own project network (`$(PROJECT_NAME)_default`), and it starts
standalone. `APP_TEST_RUN` joins that project network to reach the test database by container name.

Exactly two container runners exist, `APP_RUN` for single-shot tooling and `APP_TEST_RUN` for the suite. A third runner
is prohibited. Only `APP_TEST_RUN` joins the compose network and carries the test database variables (`DATABASE_NAME`
pointing at `$(TEST_DATABASE)`, the Flyway credentials as the database user), because only the suite reaches the
database. Every variable the file declares is referenced by a recipe or another variable, and an unreferenced one is
removed rather than kept for symmetry.

Both runners execute as the invoking host user (`-u $(HOST_USER)`), so every artifact they produce (`target/`,
`Cargo.lock`, `.cargo-home/`, everything under `reports/`) stays owned by the developer and `clean` needs no privilege
escalation. The two runners must agree on the user, because whichever runner creates `target/` first would otherwise
lock the other one out of it.

`CARGO_HOME` points at `.cargo-home/` inside the bind mount and `HOME` at `/tmp`, because the image home belongs to the
image's own user and an arbitrary uid cannot write there. Keeping `CARGO_HOME` in the working copy also keeps the
registry cache and the installed tools across runs.

A working copy that still carries root-owned artifacts fails the first `make configure` with a permission error.
Repairing it requires no host privilege, because a throwaway root container reaches the same bind mount:

```
docker run --rm -u root -v "$PWD":/app -w /app --entrypoint sh <base-images>/rust:<runtime-minor>-cli \
  -c "chown -R $(id -u):$(id -g) target reports .cargo-home Cargo.lock"
```

The double quotes matter. Single quotes would defer the expansion to the container, where `id` reports root and the
command would set the very ownership it is meant to undo.

Terminal detection tests standard input (`[[ -t 0 ]]`), never standard output. Make captures the output of
`$(shell ...)` through a pipe, so `[[ -t 1 ]]` evaluates to false even on an interactive terminal and the detection
degrades silently to the non-interactive branch.

### Migration container removal

A service that owns a schema runs Flyway in a one-shot `<service-name>-migrate` compose service, and the `start` recipe
removes that container once it has done its job:

```
@$(COMPOSE) rm -f $(PROJECT_NAME)-migrate
```

Compose offers no per-service auto-removal attribute. The `--rm` flag exists only on `docker compose run`, which is a
different execution path, so the removal is an explicit step. It sits after the `up` and never before, because the
application depends on the migration through `service_completed_successfully`, a condition that requires the container
to exist in an exited state for the `up` to release it. Removing it during the `up` would let the application start
before the schema exists.

The removal is the last line of the recipe and carries no error tolerance. `.ONESHELL:` with
`.SHELLFLAGS := -eu -o pipefail -c` runs the whole recipe in a single shell under `set -e`, so a failed migration makes
the `up` exit non-zero and aborts the recipe before the removal line is reached. The container then survives with the
full Flyway log, reachable through `docker logs <service-name>-migrate`. A migration that passes leaves nothing behind,
a migration that fails leaves everything. Wrapping the removal in `|| true` destroys that guarantee, and moving it above
the `up` breaks the ordering.

A service that owns no schema has no migration container, and its `start` recipe carries no removal line. The
discriminator is local and needs no external catalog: a `database/migrations/` directory exists in the repository, or it
does not.

## Clippy concessions

The `[lints]` table of `Cargo.toml` is where the lint policy lives: the `rust` group (`unsafe_code` forbidden,
`unreachable_pub`, `unused_qualifications`, `rust_2018_idioms`), Clippy `pedantic` at `warn` with `priority = -1`, the
nursery lints taken one by one, and the restriction lints that turn the code rules into checks (`allow_attributes`,
`absolute_paths`, `mod_module_files`, `unwrap_used`, `expect_used`, `panic`, and the rest). `clippy.toml` holds the
thresholds (`cognitive-complexity-threshold`, `excessive-nesting-threshold`, `too-many-lines-threshold`) and the
`allow-*-in-tests` lifts. `make review` runs Clippy over `--all-targets --all-features` with `-D warnings`, so every
warning is a failure.

`#[allow]` is prohibited in every Rust file of the service, and `rust-code-style` (§ Analyzer suppressions) states the
same ban from the code side. A concession is admitted for a finding the service cannot resolve in its own source: a
third-party signature that cannot express what the code guarantees, a macro expansion the service does not control, a
lint whose premise a project rule contradicts. A finding about the service's own types is fixed in those types. Each
concession follows these rules:

- A comment directly above the entry says why the lint does not fit. An entry with no reason is a finding.
- A crate-wide concession is one lint lowered in the `[lints]` table, never a whole group (`pedantic = "allow"` is
  prohibited), and a threshold is raised in `clippy.toml` only with the comment that names the bound it mirrors.
- A concession for one item is `#[expect(lint, reason = "...")]` on the narrowest item, never on a module or the crate.
  Rust has no path-scoped central exclusion, so this is the narrowest form available.
- `unfulfilled_lint_expectations` stays at its default `warn`, which `-D warnings` turns into a failure. Lowering it is
  prohibited, because it is what turns an obsolete `#[expect]` into a build failure instead of dead configuration.

```toml
[lints.clippy]
# Doc comments live on port traits only, so a public fallible constructor carries no `# Errors` section.
missing_errors_doc = "allow"
```

## Line width

`rustfmt.toml` `max_width` is the single source of the line width (`120`). `cargo fmt` applies it, and the `Makefile`
derives `MAX_LINE_LENGTH` from it for `check-line-length`, which rejects any line in `src/` or `tests/` that rustfmt
cannot break (a long string literal, a comment). `.editorconfig` `max_line_length` carries the same number for editors,
because an editor cannot read `rustfmt.toml`. A width change touches `rustfmt.toml` and `.editorconfig` together.

## Coverage and mutation thresholds

The `tests` target fails below 100 percent of lines (`--fail-under-lines 100`), and `cargo mutants` fails on any missed
mutant. Lowering the line gate is prohibited. The behavioral discipline behind these thresholds (every line covered,
every mutant killed) is owned by `rust-testing` (§ Coverage and mutation).

The exclusions are the composition root and nothing else: `COVERAGE_FILTER` in the `Makefile` and `exclude_globs` in
`.cargo/mutants.toml` name the same four files (`src/main.rs`, `src/app_settings.rs`, `src/database_settings.rs`,
`src/environment_variable.rs`), and the two lists change together (`rust-architecture` § Infrastructure files
exception). `minimum_test_timeout` stays at the value its comment justifies, because a timeout derived from a fast
baseline reports a caught mutant as a timeout when the suite reaches PostgreSQL on a loaded machine.

cargo-mutants reports a mutant that does not build as unviable and never counts it against the run, so no
`exclude_re` entry is admitted. A mutant that builds and survives is never excluded: it is killed through the tests, as
`rust-testing` requires.

## reports consistency

Every test artifact path uses `reports/` (plural, `REPORT_DIR` in the `Makefile`), consistent across the `tests`
target, the `show-reports` target, and `clean`. `report/` (singular) is prohibited.

Artifacts group by producing tool, one subdirectory each, never loose at the root of `reports/`:

| Subdirectory           | Written by                         | Holds                                         |
|:-----------------------|:-----------------------------------|:----------------------------------------------|
| `reports/coverage/`    | `cargo llvm-cov --html`            | `html/index.html` and its assets              |
| `reports/mutants.out/` | `cargo mutants --output reports`   | `outcomes.json`, the logs, and the mutant lists |

`make show-reports` opens `COVERAGE_HTML` and `MUTATION_REPORT`, and `make clean` removes `reports/` whole, with
`target/` and `.cargo-home/`. A new artifact goes into the subdirectory of the tool that writes it.
