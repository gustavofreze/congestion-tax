---
description: Core invariants, naming, typing, formatting, and doc comment rules for all Rust files in the service.
paths:
    - "src/**/*.rs"
    - "tests/**/*.rs"
---
<!-- Vendored from payment (.claude/rules/php-code-style.md) on 2026-11-18. Adapted to this project: PHP idioms mapped to Rust (immutable structs with private fields, traits, thiserror errors, `Self`), PHPStan and phpcs mapped to the compiler, Clippy, and rustfmt as `Cargo.toml` `[lints]`, `clippy.toml`, and `rustfmt.toml` configure them, PHPDoc mapped to `///` on port traits, named arguments, default parameter values, `strict_types`, and tiny-blocks sections dropped or restated for Rust, examples taken from `src/`. -->

# Code style

Conventions for every Rust file. Run `make review` after every change. **Target Rust version: the `channel` of
`rust-toolchain.toml`, edition and `rust-version` as `Cargo.toml` `[package]` declares them.**

## Pre-output checklist

1. Every function signature is fully typed, struct fields are built with the field init shorthand, no `else` or
   `else if` appears, no raw `Vec`, tuple, or `String` stands where a typed collection or value object fits, and no
   logic is duplicated across two or more places. See § Core invariants.
2. Types are immutable by default: private fields, no `&mut self` on domain types, no inheritance, and a type names
   itself as `Self` inside its own `impl`. See § Type shape, § Self-reference.
3. Identifiers carry no abbreviations and no generic names, and generic technical verbs (`process`, `handle`,
   `validate`, and the like) stay out of `src/application/`. See § Naming.
4. Identifiers, enum variants, comments, messages, and error codes use American English spelling. See § American
   English.
5. Casing follows the Rust API Guidelines, with `snake_case` HTTP responses and database columns and
   `SCREAMING_SNAKE_CASE` error codes. See § Casing conventions.
6. The domain layer declares no stand-in for a default value: no `Default` implementation and no `Option` parameter that
   means "use the usual value". See § Default values.
7. Struct fields, parameters, and struct literals order by identifier length ascending (alphabetical tie-break, semantic
   pairs in natural order), and the members of an `impl` order by group. See § Member ordering, § Parameter ordering.
8. Doc comments live on port traits only, every block opens with a domain-terms summary line, and no prohibited opener
   or technical category word appears. See § Doc comments.
9. No dead code remains, absence and emptiness are tested through `Option` and `is_empty()`, value objects compare
   through a predicate, and no error is discarded. See § Dead code, § Comparisons, § Error handling.
10. No `#[allow]` appears anywhere in the file. An unavoidable lint concession is
    `#[expect(lint, reason = "...")]` on the narrowest item, or an entry with a comment in the `[lints]` table of
    `Cargo.toml` or in `clippy.toml`. See § Analyzer suppressions.
11. Formatting is what `cargo fmt` produces with `rustfmt.toml` (`max_width = 120`), never hand-aligned, and each
    function body reads as blank-line-separated paragraphs. See § Formatting, § Vertical rhythm.
12. `make review` was run after the change and reports clean. See the note above.

## Core invariants

Invariants with no owning section below.

1. Every parameter and return type is written out in the signature. Local bindings may rely on inference.
2. Struct literals use the field init shorthand (`Self { starts_on, ends_on }`), which `rustfmt.toml` enforces with
   `use_field_init_shorthand = true`.
3. No `else` or `else if`: early `return`, `?`, a `let ... else` guard, `match`, or an `Option` and `Result` combinator
   (`map_or_else`, `ok_or_else`). The `let Some(rules) = ... else { return ... }` guard of
   `CalculatingCongestionTaxHandler` is the guard form, not an `else` branch.
4. No raw `Vec`, tuple, or primitive where a typed collection or value object is available (`Passages`, not
   `Vec<Passage>`, once the domain owns the collection).
5. No O (N²) or worse, no N+1 queries. Complexity discipline is owned by the `rust-applying-design-principles` skill.
6. No logic duplicated across two or more places, and no abstraction without real duplication or isolation need.
7. No justification comments (`// NOTE:`, `// REASON:`). The project narrows this further (`AGENTS.md` §
   Project-specific rules): no comment at all beyond the `///` doc comments on port traits and their methods and the
   `// Given`, `// And`, `// When`, `// Then` markers of the tests (`rust-testing`), so the `// TODO: <reason>` marker
   the owner's other services admit has no place here either. A name or a signature that
   needs a comment is renamed instead.
8. Every path is brought in with `use`, never written inline as an absolute path. Clippy enforces it through
   `absolute_paths`.
9. Never widen a visibility or add an item in `src/` solely to serve tests, and never add a `#[cfg(test)]` item to
   `src/`. The tests reach the crate through its public API.
10. Use the cleanest syntax the toolchain and edition natively support over a more verbose legacy form (`let ... else`,
    `?`, inline format arguments such as `format!("<{name}>")`).
11. One type per file, named after the type in `snake_case` (project rule in `AGENTS.md`).

PHP's `declare(strict_types=1)`, the ban on arguments equal to a default, and the ban on parentheses around `new` have
no Rust counterpart: the compiler never coerces types, and the language has neither default arguments nor a `new`
operator.

## Type shape

Structs keep their fields private by default and expose what callers need through methods named after the concept
(`Money::amount`, `Currency::code`). Public fields are admitted only on the plain data carriers that cross a boundary
with no invariant of their own: commands (`CalculateCongestionTax`), read models (`CongestionTax`), and the settings the
binary reads (`AppSettings`). Rust has no inheritance between concrete types, so composition is the only reuse
mechanism, and a trait is never used to emulate an abstract base.

Domain types are immutable. A value object takes `self` or `&self`, never `&mut self`, and a change returns a new value
(`ChargeWindow::raised_to` returns `Self { highest, ..self }`). A local `let mut` that builds a value before it is
wrapped (the sorted vector in `TaxRules::new`) is construction, not mutation of a domain value. Interior mutability
(`Cell`, `RefCell`, `Mutex`, `RwLock`) never appears in `src/application/`. A driven adapter whose job is to keep state
across requests holds it in a private field behind a lock, as `CachedTaxRulesCatalog` does with its entries, and no
domain or application type reads or writes that state.

## Self-reference

Inside its own `impl`, a type names itself as `Self`: the return type of `new` and of every associated constructor, the
struct literal, and the tuple constructor (`Ok(Self(normalized))`). The PHP rule that a class spells its own name
instead of `self` or `static` guards against late static binding, which Rust does not have.

## Casing conventions

Internal identifiers follow the Rust API Guidelines: `snake_case` for functions, methods, modules, and files,
`UpperCamelCase` for types, traits, and enum variants, `SCREAMING_SNAKE_CASE` for constants (`Amount::ZERO`). Beyond
that: HTTP responses and database columns use **`snake_case`**, error codes (`code` field of error payloads, written in
`src/driver/http/error_response.rs`) use **`SCREAMING_SNAKE_CASE`**.

## Default values

Rust has no default parameter values, so the PHP rule maps to what stands in for one. The domain layer implements no
`Default` and takes no `Option` parameter whose `None` means "the usual value". Every input is required, and each input
shape is a named associated constructor or constant (`Amount::new`, `Amount::ZERO`). Elsewhere, a default is acceptable
only as a neutral identifiable value most callers would repeat verbatim. A default that skips a field or expresses a
business choice is forbidden in every layer. A trait method whose third-party contract supplies a default body is the
unconditional exception.

## Private methods

A private method is a method or free function with no visibility modifier, callable only inside its own module. A
`pub(super)` method a sibling type calls is part of the module's interface, not a private method.

Private methods are prohibited by default. The domain layer allows a method that is pure (no I/O, no side effect, no
outbound port), has two or more callers within the same type, and represents an invariant or derived calculation of the
type. Single-caller logic is inlined or expressed as a closure, regardless of layer. Every other layer keeps the
prohibition: extract a collaborator or a value object, never a private method.

## Member ordering

Members order by visibility and group. Inside an `impl`, associated constants come first, then `new` and the other
associated constructors, then the remaining methods, public before restricted (`pub(crate)`, `pub(super)`) before
private. The inherent `impl` precedes the trait implementations of the same type (the `impl` of `Passage`, then
`impl FromStr for Passage`). Intra-group order is not enforced, name length is not a criterion.

At call sites, a method chain on the same receiver (a builder such as `PgConnectOptions::new().host(...).port(...)`)
orders its methods by identifier length ascending, ignoring a `with_` prefix, with alphabetical tie-break and semantic
pairs in natural order (the § Parameter ordering criterion). A boolean toggle takes no group of its own, it orders by
length like any other call. A terminal method that changes the receiver type (`build()`, `connect_lazy_with(...)`)
stays last whatever its length. Ordering restarts when a call returns a different type.

## Parameter ordering

Function, `new`, and associated-constructor parameters, struct fields, and struct literals order by identifier length
ascending, alphabetical tie-break, semantic pairs in natural order. Rust has neither defaulted nor variadic parameters.
A trailing `impl IntoIterator<Item = T>` parameter plays the variadic role and stays last
(`TaxRules::new(currency, periods)`). A start and its end (`start`/`end`, `starts_at`/`ends_at`, `starts_on`/`ends_on`,
`<prefix>_start`/`<prefix>_end`) are a semantic pair, start first whatever their length or alphabet.

`TaxPeriod` shows the field order (`tariff`, `calendar`, `validity`, `daily_maximum`, `exempt_vehicles`,
`single_charge_window`), and `ValidityPeriod` the pair (`starts_on` before `ends_on`).

## Naming

Identifiers carry no abbreviations and no generic names. A name says what in business terms, not how technically
(`daily_maximum`, not `limit_value`). Collections are plural. Closure parameters use full words
(`|passage| passage.parse::<Passage>()`). Single letters are admitted only for generic type parameters (`C`, `K`) and
lifetimes.

Methods returning `bool` read as a predicate, in one of two shapes. A state predicate about the receiver itself uses an
auxiliary prefix: `is_`, `has_`, `can_`, `cannot_`, `was_`, `should_` (`Amount::is_zero`,
`TollFreeCalendar::is_toll_free`). A relational predicate names the relation with a third-person verb and takes the
compared operand as an argument: `belongs_to`, `contains`, `equals`, `matches`, `supports`. Both lists are closed.

Generic technical verbs are restricted in every form (bare, third-person, gerund): `process`, `handle`, `execute`,
`perform`, `validate`, `check`, `manage`, `do`, `run`, `apply`. As type or module identifiers they are prohibited in
every layer, and adapter types keep their sanctioned names (`Repository`, `Gateway`, `Publisher`, `Consumer`, `Client`,
`Settings`). As method or free-function identifiers the scope is layered:

1. Throughout `src/application/`, which holds the domain models under `application/domain/`, they stay prohibited. A
   domain verb naming the actual action always exists there, use it.
2. In the driven, driver, and query adapter layers (`rust-creating-driven`, `rust-creating-driver`,
   `rust-creating-query`) they are permitted only as a fallback when no domain verb fits the actual action. A domain
   verb still wins when one exists, the generic verb is the last resort, never a default.

Three named exceptions apply independent of layer:

1. Inbound port methods carry the domain verb of their trait (`CalculatingCongestionTax::calculate`). The fixed PHP
   `handle()` has no counterpart here.
2. Third-party methods at call sites (`execute` and `fetch_optional` on sqlx, `route` and `merge` on axum).
3. `new` is the constructor name by Rust convention and is never read as a generic verb.

### Design-pattern names

A design pattern is a shape, never a name. A GoF pattern enters a change only on its own entry signal, never
preventively, and the type keeps its business name. Pattern-name suffixes (`...Strategy`, `...Decorator`, `...Factory`,
`...Visitor`, and the like) are prohibited in type names, so the caching decorator is `CachedTaxRulesCatalog`, never
`TaxRulesCatalogDecorator`. The adapter-kind suffixes (`Repository`, `Gateway`, `Publisher`, `Consumer`, `Client`) and
`Settings` are legitimate adapter naming. `Builder` is legitimate on a Test Data Builder under
`tests/support/builders/` (`TaxRulesBuilder`), the one home that pattern has. Entry signals, canonical homes, and
catalogs live in the layer skills (`rust-creating-application`, `rust-creating-driven`, `rust-creating-driver`,
`rust-creating-query`).

## Dead code

No dead or unused code: a function, method, constant, field, parameter, import, or type with no caller or reference is
removed. The compiler reports most of it (`dead_code`, `unused_imports`), and `unreachable_pub` reports a `pub` item no
other crate can reach. Never silence these with a leading underscore or an attribute to keep code alive. An import a doc
comment names in an intra-doc link is live and stays.

## Analyzer suppressions

No `#[allow(...)]` appears in a Rust file, on an item, a module, or the crate. `Cargo.toml` turns the ban into a check:
the restriction lints `allow_attributes` and `allow_attributes_without_reason` are on, and Clippy runs with
`-D warnings`. `#[rustfmt::skip]` is prohibited for the same reason. There are two admitted channels, whose entry shape
is owned by `rust-skeleton-tooling` (§ Clippy concessions):

1. A crate-wide concession is an entry in the `[lints]` table of `Cargo.toml`, or a threshold in `clippy.toml`, each
   with a comment directly above it saying why (`missing_errors_doc = "allow"` and the test-target lifts in
   `clippy.toml`).
2. A finding at one item that cannot be fixed in the service's own source is `#[expect(lint, reason = "...")]` on the
   narrowest item. Rust has no path-scoped central exclusion, so this is the one inline form admitted.

An `#[allow]` silences a finding and then travels with the item. It survives the rename that moved the code, it survives
the refactor that made it unnecessary, and nothing ever reports it as obsolete. An `#[expect]` raises
`unfulfilled_lint_expectations` once the lint stops firing, which `-D warnings` turns into a build failure, and a
`[lints]` entry reads in one file with its reason. Every concession the service makes to the analyzer then stays
reviewable.

Fix the code before adding a concession. A concession is correct only for a finding the service cannot resolve in its
own source: a third-party signature that cannot express what the code guarantees, a macro expansion the service does not
control, a lint whose premise the project rules contradict (`missing_errors_doc`, because doc comments live on port
traits only). A finding about the service's own types is a defect in those types, and the concession that hides it is a
second defect.

## Comparisons

1. Absence is an `Option`, tested with `let Some(...) = ... else`, `match`, `is_none()`, or a combinator, never with a
   sentinel value.
2. Empty text and empty collections are tested with `is_empty()`, never by comparing a length with zero or the text with
   `""`.
3. Two value objects of the same type compare through a predicate or `PartialEq` the type exposes
   (`earlier.validity().overlaps_next(later.validity())`), never by unwrapping state at the call site
   (`earlier.ends_on() >= later.starts_on()` written in the caller).

## American English

All identifiers, enum variants, comments, messages, and error codes use American English spelling: `canceled`,
`organization`, `initialize`, `behavior`, `modeling`, `labeled`, `fulfill`, `color`.

## Doc comments

`///` doc comments are restricted to port traits and their methods (`CalculatingCongestionTax`,
`TaxRulesCatalog`, `DependencyProbe`, `MonotonicClock`), never on structs, enums, functions, or `impl` blocks. The trait
and every method on it carry one. Prose follows the global punctuation default.

Every block opens with a domain-terms summary line ending with a period, never naming the Rust construct, the storage
mechanism, or the design rationale. Trait summaries: nominal `"{Concept} {qualifier}."`, or
`"{Verb}ing of {object} {condition}."` when the name ends in `*ing`. Method summaries: third-person present
`"{Verbs} the {object} {effect}."`, never imperative, past, or passive.

Prohibited openers, on the trait or its methods: `Marker for`, `Contract for`, `Interface for`, `Trait for`,
`Wrapper for`, `Implementation of`, `Represents`, `Used to`, `Helper for`, `Provides`, `Defines`,
`Receives a request to`, `Handles a request to`, `Processes`. A doc comment never names persistence, concurrency,
design justification, or technical category words (`use case`, `marker`, `port`, `adapter`, `command`, `payload`).

The list yields in one case: an opener that is the third-person present of the domain verb the trait name already
carries. A `*ing` trait is required to share its verb with its single method (see below), so the method of a
`CalculatingCongestionTax` opens with `Calculates` because that is the domain action, not because it describes the Rust
construct. The carve-out reaches only that coincidence. The same word opening a doc comment that does not derive from
the trait name stays prohibited, and it never licenses renaming the trait to reach a more comfortable verb.

A trait named `*ing` shares its domain verb with its single method: gerund on the trait, third-person present on the
method. Synonyms and generic verbs are prohibited even when natural. Ports of the same kind share one doc comment
template across the codebase.

Rust doc comments carry no tags. Parameters are named in the prose of the summary, and the failures go in a closing
paragraph of the shape `Fails when {domain condition}.` or `Fails with {Error} when {domain condition}.`, as
`TaxRulesCatalog::find` does. A doc comment with no summary line is prohibited.

## Common type primitives

A service keeps the value-object primitives its own domain repeats under `src/application/domain/models/commons/` (a
non-empty text, a length-capped text, a positive integer, and the like). Which ones exist, what each exposes, and which
error each raises are per-service facts. Read them from that directory before writing a value object, never from a list
kept here. This service has no `commons/` directory today. Two invariants hold whatever the catalog turns out to be:

1. Compose an existing primitive instead of re-implementing its validation inline. Where a primitive already raises the
   empty or length failure, a per-value-object error for the same condition is not introduced.
2. Normalization, charset, and format-specific rules live on the owning value object. A value object whose validation
   goes beyond what a primitive covers implements its own and does not compose that primitive (`City::new` trims and
   lowercases, `Currency::new` checks the ISO 4217 shape).

A primitive exists only once a second value object needs it, and it exposes only the operations its consumers call
(`rust-design-principles` § Precedence, no preventive abstraction). A service with a single text value object therefore
has no shared text primitive to compose and guards on the type itself, which is conformance, not a gap.

## Format strings

A Rust format string is a literal the macro checks at compile time, so the PHP rule of assigning the template to a
variable before `sprintf` has no counterpart. The format string stays inline in the macro and captures identifiers by
name where it can (`#[error("No tax period of the city covers the passages of <{date}>.")]`,
`format!("The environment variable <{name}> must be set.")`).

## Return statements

Guard returns (before the main logic begins) are unlimited. After the first non-guard return, at most three more
`return` points (an explicit `return` or the tail expression) are allowed, a hard ceiling of four non-guard returns.
Beyond that, rewrite with `match` or map dispatch to a single non-guard return.

## Function size

Length is a readability signal, not a metric: past roughly fifty logical lines a function is reviewed for the cause of
its length, never trimmed to a number. Clippy enforces the outer bounds `clippy.toml` sets (`too-many-lines-threshold`,
`cognitive-complexity-threshold`, `excessive-nesting-threshold`). Branching or accumulated logic is rewritten with
`match` or map dispatch, or the rule moves to the owning aggregate, value object, or domain service. Mechanical
low-complexity assembly is left as it reads. Length is never reduced by extracting a private method outside the domain
layer (§ Private methods).

## Local variables and call nesting

In `src/`, prefer a named intermediate binding over nesting a call's result inside another call (a readability
preference, not a hard rule, applied where it helps). Typical shapes: promote each command field into a named value
object before the call that uses it (`city`, `vehicle`, `passages` in `CalculatingCongestionTaxHandler`), resolve
collaborators into named bindings before a constructor call (`catalog` in `src/main.rs`). A method chain on a single
receiver stays a chain. This is the opposite of the test-side guidance (owned by `rust-writing-tests`), where a value
consumed once is inlined.

## Formatting

- **rustfmt owns the layout.** `cargo fmt --check` runs in `make review` with `rustfmt.toml` (`max_width = 120`,
  `style_edition = "2024"`, `use_field_init_shorthand`, `use_try_shorthand`). Never align by hand, and never fight the
  formatter's choice of where to break a signature, an argument list, or a chain.
- **Lines within 120 characters.** The `check-line-length` step of `make review` rejects any line in `src/` or `tests/`
  over `max_width`, which catches the string literals and comments rustfmt cannot break.
- **Trailing commas** follow rustfmt: present after the last element of a multi-line list, absent on a single line. The
  PHP rules on `=>` alignment and on the trailing comma do not carry over.

## Vertical rhythm

A function body reads as short paragraphs, one per step, separated by a single blank line: guards one paragraph, the
work the next, the tail expression set off by a blank line above it. A blank line never splits a tight pair (a value
built and immediately handed to the call it serves). The unit is the step, not the line. Do not pad a one-statement body
or a guard-only function.

## Error handling

No error is discarded. A `Result` is propagated with `?`, translated into a domain or transport outcome
(`map_err(|_| PassageFormatNotValid::new(raw))`), or recovered with a meaningful value
(`unwrap_or_else(PoisonError::into_inner)`). `let _ = fallible()`, `.ok()` used only to drop the error, and an `Err(_)`
arm with an empty body are prohibited. `unwrap()`, `expect()`, `panic!`, `todo!()`, and `unimplemented!()` stay out of
production code, which the restriction lints in `Cargo.toml` enforce, and `clippy.toml` lifts the panic family for the
test target only.
