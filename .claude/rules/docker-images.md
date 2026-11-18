---
description: Authoring and consumption rules for shared container base images. The image bodies, their configuration, and the verification gate live in the base-image repository. App-specific compose services and tuning stay in each app. Where the organization publishes no base image for a family, only the consuming half applies, against the upstream official image.
paths:
    - ".dockerignore"
    - "Dockerfile*"
    - "docker-compose*.{yml,yaml}"
---
<!-- Vendored from payment (.claude/rules/docker-images.md) on 2026-11-18. Adapted to this project: the PHP base, FPM, nginx sidecar, and Composer authentication became the Rust base family and Cargo, the consumption and tag sections follow the floating alias that the Rust base namespace publishes and the app Dockerfiles consume (as in account), the per-family resolution, the compose pin, and the tracked .env.local come from account, and the paths globs keep only the Docker files this project has. -->

# Shared container images

When `<base-images>` is set, a `docker-images` repository publishes that namespace and is the single source for every
container artifact replicable across the organization's apps. Apps consume the published base images and add only their
own dependencies, code, and tuning. This rule covers both roles: authoring a base image inside `docker-images` and
consuming one inside an app. With `<base-images>` unset there is no such repository, § Authoring does not apply, and an
app inherits the upstream official image pinned to an exact tag. Bash inside `RUN` steps, entrypoints, and container
scripts follows the project's Bash script conventions (the `shell-scripts` rule). The repository root layout is owned
by `skeleton-layout`.

## Pre-output checklist

Verify every item before producing any Dockerfile, dockerignore, or compose file.

1. Every app Dockerfile inherits `FROM` the floating `<upstream-minor>-<role>` alias of a `<base-images>` image, never
   the full version tag and never `latest`. With `<base-images>` unset the base is the upstream official image pinned to
   an exact tag. See § Consuming.
2. No app re-declares what its base already provides, and where no base provides it, the app declares it itself. See §
   What the base provides.
3. A `.dockerignore` sits next to every Dockerfile and secrets never enter a build context copy. See § Build context.
4. Builds authenticate a private Cargo registry through a BuildKit secret, never through a baked `ARG` or `ENV`. See §
   Secrets.
5. Base images are multi-target, pinned to an exact upstream tag, and hardened. Service and consumer-facing targets (a
   runtime an app runs, a sidecar, a static server) run non-root and ship a health check where the base knows the probe.
   Build/toolchain and one-shot/CLI targets (a builder stage discarded in the multi-stage app build, a job that runs
   once and exits) keep root only as a named, justified exception and expose no port. See § Authoring.
6. Compose files declare no `version:` key. Every long-running service has resource limits and a health signal. See §
   Compose.
7. Every Dockerfile, base image or consumer, begins with `# syntax=docker/dockerfile:1` on its first line. See § Syntax.
8. A consumer Dockerfile carries no comment other than that syntax directive. See § Consuming.
9. Layers are lean: one `RUN` per concern with its cache cleaned in the same layer, ephemeral build dependencies deleted
   before the layer closes, `COPY` never `ADD`, and dependency layers before code layers. See § Lean layers.

## Syntax

Every Dockerfile begins with `# syntax=docker/dockerfile:1` as its literal first line, the base images in
`docker-images` and the thin consumer files in an app alike. It pins the stable BuildKit frontend so build-time mounts,
secrets, and heredocs parse identically on every machine and in CI. The directive is the one line that is not a build
instruction and, in a consumer Dockerfile, the one line that is not a comment-free instruction either.

## What the base provides

A base bakes in the directives below, and the prohibition on restating them holds only where a base actually provides
them. With `<base-images>` unset there is no such base, so the app Dockerfile inherits the upstream official image at an
exact tag and declares them itself: the non-root `USER` with its `WORKDIR` owned by that user, the toolchain, and the
hardening. The invariant is that the runtime image runs non-root, ships a health check, shuts down gracefully, and is
hardened. Which layer declares it is the binding.

The Rust base (`<base-images>/rust`, four targets) bakes in everything below, and while it does, an app Dockerfile that
restates any of these is wrong:

- Non-root user: `app` (uid and gid 1000) with `WORKDIR /app` owned by it, the last `USER` of the `builder`, `runtime`,
  and `development` targets.
- Toolchain: Cargo and the musl target in `builder`, plus Clippy, rustfmt, and `rust-src` in `development`.
- Runtime: a minimal root filesystem with no toolchain, `RUST_BACKTRACE=1`, and the busybox `wget` a health probe calls.
- A `cli` tooling variant (Cargo, Clippy, rustfmt, git, bash, curl) consumed by the `Makefile` `RUST_IMAGE` runner and
  by nothing else, never by a Dockerfile.

The Rust base bakes no `HEALTHCHECK` and no `EXPOSE`, and its `CMD` is a shell placeholder, because the probe path, the
port, and the binary belong to the app. The app runtime stage declares those three itself, which is a declaration the
base does not provide and therefore not a restatement.

The migration base (`<base-images>/flyway`, single `runtime` target) bakes the Flyway entrypoint script and prunes the
driver jars no service uses. Where that family is absent, whether because the namespace publishes no Flyway base or
because there is no namespace, the migrate service runs the upstream official Flyway image at its `<major>.<minor>` tag
(§ Compose), mounting only `database/migrations/` read-only and passing the Flyway environment variables, and any
entrypoint it needs lives under `docker/flyway/`. The database container follows the same split, and with no database
family it runs the upstream official image, its bootstrap hooks under `docker/` (owned by `skeleton-layout`).

## Stage vocabulary and tags

Five stage names are the closed vocabulary. Each names what the image is, and a family ships only the stages its role
calls for.

- **`builder`**. The compilation base an app inherits via `FROM`. It carries the toolchain that installs dependencies
  and compiles them, and the multi-stage app build discards it. Families: php, node, rust.
- **`runtime`**. The production image the app runs (the statically linked Rust binary non-root, the nginx sidecar
  non-root on 8080, the Flyway one-shot job). Families: php, rust, nginx, flyway.
- **`cli`**. Command-line tooling for one-shot commands (Cargo, Clippy, and rustfmt for Rust). Families: php, rust.
- **`static`**. The nginx server for a frontend's pre-built assets, non-root on 8080. Family: nginx.
- **`development`**. A variant of `runtime` built `FROM runtime`, adding local development tooling. It runs on a
  developer's machine and is never deployed. It is subordinate to `runtime`, not a peer of it. Families: php, rust,
  nginx.

Every version tag is `<upstream-minor>-<role>-<semver>`, where `role` is identical to the stage name. There is no bare
tag: a role always sits between the upstream minor and the semantic version. Version tags are immutable, so any input
change bumps the family `VERSION`. The version tag is the record of what was published. `latest` is forbidden.

Each stage also carries one floating alias `<upstream-minor>-<role>` that tracks its latest release, written in the
registry by the publishing pipeline and never pushed from an app. An app consumes that alias, so a fix published in the
base reaches every app on its next build with no change in the app.

## Version layer

Each family that pins an upstream keeps one build unit per upstream minor or major, under a `<upstream-minor>/`
subdirectory (`images/rust/1.100/`, `images/php/8.5/`). The subdirectory segment is the exact minor (or major, for a
family versioned by major) read from the Dockerfile `FROM` pin. A build unit is self-contained: its own `Dockerfile`,
`VERSION`, configuration, baked scripts, and `scripts/smoke`, with a semantic version that advances independently of
every other upstream version. Rust 1.100 can sit at `1.3.0` while a future Rust 1.101 sits at `1.0.0`.

Two kinds of upstream change move differently:

- **Minor or major bump** (Rust 1.100 to 1.101, Flyway 12 to 13). The Dockerfile and its configuration diverge (a new
  pin, a changed component, a changed build flag), so the build unit moves to a new `<upstream-minor>/` subdirectory
  whose `VERSION` restarts at `1.0.0`. Two forms are allowed, and the fleet decides which. **Renamed in place** when
  every consumer moves at once: the old subdirectory stops existing and there is one line. **Sibling** when consumers
  migrate on different schedules: the two lines publish in parallel, nothing in the old subdirectory changes, and the
  old line is deleted once no consumer pins it. A sibling with no consumer is dead weight in `build` and `scan`, so do
  not leave one standing.
- **Patch bump** (Rust 1.100.0 to 1.100.1). The upstream pin and the family `VERSION` both move inside the same
  subdirectory. No new directory appears, and the floating alias tracks the new patch.

The invariant that stands: a new upstream minor or major starts a new `<upstream-minor>/` subdirectory with its own
`VERSION` at `1.0.0`, honoring the dependency cooldown on the upstream pin where the base-image repository applies one.
The step-by-step wiring (Makefile build, lint, scan, audit, efficiency, smoke, publish) lives in an authoring skill, not
this rule. A scaffold family with no pinned upstream has no build unit and no `<upstream-minor>/` subdirectory yet. The
subdirectory is born with its first Dockerfile.

## Consuming

An app consumes the base in exactly two thin files:

- `Dockerfile.dev`: `FROM <base-images>/rust:<minor>-development` plus a `COPY` of the app and the debug build.
- `Dockerfile.prod`: a builder stage `FROM <base-images>/rust:<minor>-builder` running `cargo fetch --locked` and then
  `cargo build --release --locked --offline`, then a runtime stage `FROM <base-images>/rust:<minor>-runtime` copying the
  built binary with `--chown=app:app`. The application directory of the production image then holds only that binary.

With `<base-images>` unset there is no floating alias. No base-image repository reviews a patch before a moving tag
reaches it, so both files name the upstream official Rust image at an exact tag on `<runtime-minor>`, and a patch
arrives only through a commit. Both files keep the same two-stage shape and additionally carry the directives § What the
base provides lists. They stop being thin, and the exact pin, the builder/runtime split, the application directory
contents, and the `--chown` all still hold.

An app adds only its own dependencies, code, and tuning. Dependency layers come before code layers so the cache survives
code changes: copy `Cargo.toml`, `Cargo.lock`, and `rust-toolchain.toml`, fetch, then copy the source. The usage
contract is documented in the base-image repository README.

Whichever file names the tag, the image has to reach the local daemon before the build runs, and it always gets there by
being pulled from the registry. An app never builds a base image locally, not even where a checkout of the base-image
repository sits beside it. A published tag names published bytes, while a local build names whatever that checkout
happens to hold, so two machines on the same tag would stop agreeing and the pin would document nothing. The pull is
Docker's own, triggered by the `FROM` line or by a runner naming the image, and it covers every image the stack consumes
(the app base, the database, the Flyway migration image) with no `Makefile` target in between. Building those images is
the base-image repository's own job, and publishing them is what makes them reachable here.

A consumer Dockerfile carries no comment. The only line that is not a build instruction is the
`# syntax=docker/dockerfile:1` directive. A consumer file is short and does one obvious thing per line, so a comment
would only restate the instruction. The reasoning behind each step lives in the base image it inherits and in this rule,
never copied into the app.

`<base-images>` resolves per family, and every sentence below is read against the family it names rather than against
the namespace as a whole. A namespace publishing a Rust base and no Flyway base is set for the app image and absent for
the migrate service at the same time, which is `AGENTS.md` § Capability absence applied one container at a time. Asking
only whether the namespace exists gets the migrate service wrong.

The migrate service is consumed directly in the compose file. Where a Flyway family exists it is
`image: <base-images>/flyway:<minor>-runtime` with no entrypoint override, and where none does it is the upstream
official image at its `<major>.<minor>` tag (§ Compose), with a read-only mount of `database/migrations/` and the
Flyway variables in both cases. The app keeps its Traefik labels while `<local-proxy>` is set, publishing its port
directly otherwise.

## Build context

Every Dockerfile has a dockerignore covering at least `.git`, `.env` and `.env.local`, IDE directories, `reports/`,
`tests/`, and the `Dockerfile*` and `docker-compose*` files themselves. Cargo's `target/` and the `Makefile` dependency
cache `.cargo-home/` stay out too, so the builder stage compiles from the lock file alone. Both Dockerfiles of this
project build from `Cargo.lock` with `--locked`, so one `.dockerignore` serves both, and a per-Dockerfile override
(`Dockerfile.prod.dockerignore`) appears only when the two contexts must differ.

## Lean layers

Every layer carries only what the image keeps. One `RUN` groups a single concern and cleans up after itself in the same
layer, so nothing it fetched survives into the image:

- Package installations use the no-cache mode of the package manager, and build-only dependencies are installed under a
  virtual group and deleted (`apk add --virtual .build-deps … && … && apk del .build-deps`) before the `RUN` ends. A
  tool removed in a later layer still occupies the earlier one, so the removal and the installation share a layer.
- `COPY` moves files in, never `ADD`. `ADD` also fetches URLs and unpacks archives, which hides what enters the image.
  Fetch explicitly and verify a checksum when a remote artifact is needed.
- Dependency layers come before code layers so a code change does not invalidate the dependency cache. Copy the lock
  file and fetch, then copy the source.

The efficiency stage of the base-image review gate holds these to fixed thresholds. A widening layer is a signal to
prune the `RUN`, or to delete in the stage a `COPY` reads from, not to relax the threshold.

## Secrets

No secret is embedded in any image layer or versioned compose file. A private Cargo registry authenticates with
`RUN --mount=type=secret,id=cargo_token,env=CARGO_REGISTRIES_<NAME>_TOKEN` and the caller passes
`--secret id=cargo_token,env=CARGO_REGISTRIES_<NAME>_TOKEN`. In CI the value is the read-only registry token the
pipeline carries, the single credential that reads private crates, held as a repository or organization secret and
passed in by `<ci-workflows>` or, with `<ci-workflows>` unset, by the service's own workflow. It reaches the build as a
build secret and never as a baked `ARG`, an `ENV`, or a tracked file, whatever the project happens to name it. Where the
project has no such token, no build secret is mounted, because no private crate needs one, and the mount is dropped
rather than replaced by a baked `ARG`. Runtime configuration enters through environment variables (compose `env_file`,
the project's secret store in the cloud), never through files baked into the image. The `.env.local` that `env_file`
reads is a tracked file (`skeleton-layout` § `.env` strategy), so it holds local-only defaults and never a real value.
With no secret store a real value has no home in the repository at all, and it reaches the container through the
deployed environment.

## Authoring

This section applies only where the organization runs a base-image repository (`<base-images>` set). It describes that
repository, not this one: nothing below is authored in an app. With `<base-images>` unset nothing below is authored
anywhere, and this section, § Stage vocabulary and tags, and § Version layer describe a repository the project does not
have. Every other section still holds, § Syntax and § What the base provides included, the latter as the list the app
Dockerfile declares itself.

Inside `docker-images`, every image family lives under `images/`, one directory per family (like `images/rust/` and
`images/php/`). Each family that pins an upstream holds one self-contained build unit per upstream minor or major under
a `<upstream-minor>/` subdirectory (like `images/rust/1.100/`), per § Version layer. Each build unit ships a single
`Dockerfile` with the targets it needs (`runtime` always, `development`, `builder`, `cli`, and a `static` server where
the family calls for them), a `VERSION` file with its base semantic version, its configuration or baked scripts, and a
`scripts/smoke` assertion script.

Non-root and a health check are not a blanket property of every target, they are the contract of the targets an app
actually runs. A service or consumer-facing target (a runtime an app runs, a sidecar, a static server) runs as a
non-root last `USER` with its `WORKDIR` owned by that user, ships a `HEALTHCHECK` where the base knows the probe (a base
that cannot know it, like the Rust runtime, leaves it to the app), and exposes only an unprivileged port. A
build/toolchain target (a builder stage the multi-stage app build discards) or a one-shot/CLI target (a job that runs
once and exits, a local tooling image) may keep root as its last user, but only as an exception named and justified
where it is declared, and it exposes no port. The review gate enforces this split per target: the audit stage keeps the
non-root check active on every service target and exempts only the named root targets, and each `scripts/smoke` asserts
non-root where the target's class requires it. Upstream images are pinned to an exact tag (patch version and distro
release). Nothing app-specific enters the base-image repository. Every change runs the review gate (`make review`: lint,
build, vulnerability scan, CIS audit, layer-efficiency check, and smoke) before it is published, and any input change
bumps `VERSION`. An upstream vulnerability finding that cannot be fixed locally (a vulnerable jar inside an upstream
bundle) is accepted only through a suppression scoped to the single family it affects, carrying a written justification
and the revisit condition, never a repository-wide one. Publication pushes the immutable version tags to the registry,
itself provisioned as infrastructure code (through `<platform-iac>` where the organization has a shared stack) and never
created by hand, and then points each floating alias at the version tag just published.

## Compose

Tag pinning splits by where the tag is read, and the compose file sits on the permissive side of that split. A `FROM`
line decides what a built artifact contains, and a base that moves outside a reviewed alias silently changes what ships
(§ Consuming). A compose image is built into nothing: this file is the local development stack, no CI job runs it, and
the deployed counterparts of these containers are provisioned elsewhere. An upstream official image consumed here is
therefore pinned to `<major>.<minor>`, which picks up a patch fix on the next `docker compose pull` rather than on the
next pull request, and the upstream minor is a real compatibility boundary. The floor stays a floor: `latest` and a bare
major are still forbidden, because neither states what the service runs on. A `<base-images>` image is consumed through
its `<upstream-minor>-<role>` alias here too, the same as in § Consuming.

No `version:` key (deprecated). Every long-running service declares `cpus` and `mem_limit`. Health signals: services
whose image ships a `HEALTHCHECK` inherit it, other long-running services declare a `healthcheck:` block. The Rust
runtime ships none and `Dockerfile.dev` declares none, so the app service carries the block. With `<base-images>` unset
an image ships one only because the app declared it, so the compose service carries the block unless its own
Dockerfile already carries the directive. One-shot jobs (a Flyway migration container) are exempt. Databases persist on
named volumes prefixed with the service name so they can be purged per service on re-provision. Configuration reaches
containers via an `env_file:` entry or via `${NAME:-}` interpolations with an empty default, never as a value written
into the compose file. An empty-default interpolation commits no value and hands the container only the names it needs.
Neither carries a real credential, because the compose file and the `.env.local` both read are tracked.

## Delegated to owners

- Script content (entrypoints, init hooks, smoke scripts): the shell-scripts rule.
- Repository root layout and `docker/` substructure: the skeleton-layout rule.
- CI workflow structure: the github-workflows rule.
