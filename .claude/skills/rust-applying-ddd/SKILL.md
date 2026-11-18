---
name: rust-applying-ddd
description: Domain-Driven Design reference, tactical and strategic. Use when deciding entity vs value object, drawing aggregate boundaries, placing a business rule, or integrating two bounded contexts.
---
<!-- Vendored from payment (.claude/skills/php-applying-ddd/SKILL.md) on 2026-11-18. Adapted to this project: the generic structure became the Rust module tree of src/application/domain, the domain event block and the emit step were dropped because this service emits no domain events, the folder layout routes to rust-architecture (from account) instead of the skipped skeleton skill, the spec read became a read of openapi.yaml and the docs pages, and every route names the rust-* rules and skills. -->

# Domain-Driven Design (tactical and strategic)

This skill is an opinionated synthesis of Domain-Driven Design, not the canon itself. It distills the tactical building
blocks and the strategic patterns into the decisions you actually make while modeling, and it routes you to the source
of record for each question (Evans for the foundations, Vernon for aggregate design, Fowler for context boundaries).
Treat it as a map, not a mandate. DDD earns its complexity only on a domain with real rules. On a thin CRUD surface a
transaction script is the honest choice.

The concepts here are language-agnostic. The short examples come from this repository's own model (`TaxRules`,
`Currency`, `Amount`, `TaxPeriodsOverlap`) where one exists, and from a neutral domain (Order, Customer, Account, Money)
where it does not, so the shape carries over to any model. The ENFORCED specifics for this repository live in rules,
routed in § In this service.

This service records no domain events: its one aggregate is read and evaluated, never transitioned and persisted. The
domain event block and the emit step of a transition are therefore left out of this skill. When a future aggregate
records events, the owning rule is `rust-hex-application-domain`.

## When to use (and when NOT to)

| Use When                                                        | Skip When                                                                          |
|-----------------------------------------------------------------|------------------------------------------------------------------------------------|
| The domain has real invariants and behavior, not just fields    | A thin CRUD form over one table with no rules, a transaction script is honest      |
| You must decide entity vs value object, or where a rule lives   | A read-only list or report (a CQRS read model needs no aggregate)                  |
| You are drawing aggregate boundaries and consistency rules      | A throwaway script, a one-off import, or a spike                                   |
| You are aligning names and code with the business vocabulary    | The whole feature is a pass-through to an external system with no model of its own |
| Two contexts must integrate and you need a translation boundary | The team has no shared language yet (settle the language first)                    |

**Start simple. Evolve complexity only when needed.**

## Tactical building blocks

The types the domain is modeled with. Each answers one primary question. Treating one as another is the most common
modeling error, so the last column names the trap.

| Block          | Primary Question                       | Use It For                                                | Do Not Treat As                                    |
|----------------|----------------------------------------|-----------------------------------------------------------|----------------------------------------------------|
| Value Object   | What is it, by its value?              | Money, Currency, a date range, an amount, a status        | A row to persist on its own, or a thing with an id |
| Entity         | Which one is it, over time?            | Something with identity and a lifecycle (Order, Customer) | A bag of public fields, or a record                |
| Aggregate      | What changes together?                 | A consistency boundary around one root plus its parts     | A convenience graph of every related type          |
| Aggregate Root | Who guards the invariants?             | The single entry point for changes inside the boundary    | A pass-through that exposes its internals          |
| Repository     | Where do aggregates live?              | A collection illusion, one per aggregate root             | A per-table DAO, or a query bag for the read side  |
| Domain Service | Whose rule is it, when no one owns it? | A rule spanning two aggregates with no natural host       | A home for orchestration or persistence            |
| Factory        | How is a valid one born?               | Encapsulating non-trivial construction of a valid value   | A place to skip invariants                         |
| Domain Error   | Which invariant was violated?          | A named failure in business language                      | A carrier of HTTP codes or formatted strings       |
| Invariant      | What must always be true?              | A rule enforced inside the boundary, every change         | A check sprinkled across handlers and endpoints    |

Deep detail and an example per block are in `references/tactical.md`.

## Decision tree: Entity vs Value Object

```
Does the thing have a distinct identity that must be
tracked over time, independent of its attributes?
│
├─ NO ──► Are two instances with equal attributes fully
│         interchangeable (no lifecycle of their own)?
│         │
│         ├─ YES ──► VALUE OBJECT
│         │          immutable, compared by value,
│         │          self-validating in its constructor
│         │
│         └─ NO ───► reconsider. Most "no identity" things
│                    still resolve to a value object
│
└─ YES ─► Does it change state over its lifetime and need
          to be told apart from an equal-looking peer?
          │
          ├─ YES ──► ENTITY
          │          identity by id, changed through methods
          │
          └─ NO ───► VALUE OBJECT after all
                     (equal value, no separate lifecycle)
```

## Decision tree: Aggregate boundaries

```
Start from one entity. For each related type, ask:

Must this value stay transactionally consistent with the
root on every change (a true invariant spans both)?
│
├─ YES ─► Is it meaningful only inside this root
│         (no independent lifecycle, never loaded alone)?
│         │
│         ├─ YES ──► SAME AGGREGATE
│         │          reached only through the root
│         │
│         └─ NO ───► its OWN AGGREGATE
│                    referenced by id, loaded separately
│
└─ NO ──► SEPARATE AGGREGATE
          reference by id, accept eventual consistency
          (never a shared transaction)

Rule of thumb: keep aggregates small. One root plus the
values that cannot be correct on their own.
```

## Decision tree: Where does this logic go?

```
Where does this business rule belong?
│
├─ Uses only one entity's own state?
│   └─► METHOD ON THAT ENTITY (or its aggregate root)
│
├─ Uses only the fields of one value (format, range,
│   arithmetic, comparison)?
│   └─► METHOD ON THE VALUE OBJECT
│
├─ Spans two or more aggregates, or has no honest home
│   on any single one?
│   └─► DOMAIN SERVICE (stateless, domain types in and out)
│
└─ Loads, saves, calls an external system, or sequences
    a use case end to end?
    └─► NOT THE DOMAIN. An application handler over a port.
```

## Generic structure

A neutral domain layout, organized by aggregate and concept, never by technical type. Placeholders in `{braces}` stand
for your own names. In this repository `models/tax_rules.rs` declares the `TaxRules` root and `models/tax_rules/` holds
its value objects.

```
src/application/domain/
├── models.rs                          # declares one module per aggregate
├── models/
│   ├── commons/                       # value objects shared by more than one aggregate
│   │   ├── {shared_value_object}.rs
│   │   └── {aggregate_identity}.rs
│   ├── {aggregate}.rs                 # the aggregate root, the one entry point
│   ├── {aggregate}/
│   │   ├── {aggregate}_id.rs          # typed identifier
│   │   ├── {child_entity}.rs          # reached only through the root
│   │   ├── {value_object}.rs          # immutable, self-validating
│   │   └── {status}.rs                # enum, guards its own transitions
│   └── {other_aggregate}/
│       └── {other_aggregate}_id.rs    # foreign aggregate, referenced by id only
├── errors/
│   └── {invariant_violated}.rs        # named after the rule, business language
└── services/
    └── {business_operation}.rs        # only when a rule fits no single aggregate
```

## Short examples

A value object is immutable, equal by value, and self-validating: the worked shape, `Currency`, is in
`references/tactical.md` § Value Object. The aggregate-root creation invariant (`TaxRules::new` rejecting overlapping
periods), the neutral guard-then-mutate transition, and the cross-aggregate domain service example are in
`references/tactical.md` (§ Invariant and § Domain Service).

## Strategic design

Strategic DDD is about boundaries between models, not types inside one. It decides where one model ends, where another
begins, and how the two talk.

| Concept               | Primary Question                  | Use It For                                                   | Do Not Treat As                                     |
|-----------------------|-----------------------------------|--------------------------------------------------------------|-----------------------------------------------------|
| Ubiquitous Language   | What do we call this, together?   | A shared vocabulary binding code, tests, and conversation    | Jargon the developers invented alone                |
| Bounded Context       | Where does this model apply?      | The boundary inside which one model and one language hold    | A synonym for a module or a microservice by default |
| Context Map           | How do contexts relate?           | Naming the relationship between two contexts (and its risks) | An afterthought drawn once and never revisited      |
| Anti-Corruption Layer | How do we stay clean at the edge? | Translating a foreign model into our own at the boundary     | A thin rename, it is a real translation             |
| Shared Kernel         | What do two contexts truly share? | A small, jointly-owned model shared by explicit agreement    | A dumping ground of common code                     |

Each term and the full set of context-map relationship patterns are in `references/strategic.md`.

## Anti-patterns

| Anti-pattern           | Problem                                                                           | Fix                                                                                     |
|------------------------|-----------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------|
| Anemic Domain Model    | Entities are data bags, all logic sits in services, the model carries no behavior | Move the rule onto the aggregate, the entity, or the value object that owns the data    |
| Repository per Entity  | A repository for every table, including child entities, leaks the boundary        | One repository per aggregate root. Children are reached through the root                |
| God Aggregate          | One aggregate pulls in every related type, transactions get huge and contend      | Split by true invariants. Reference other aggregates by id, accept eventual consistency |
| CRUD thinking          | Modeling create, read, update, delete instead of business intentions              | Name the intentions (place, cancel, confirm). Methods reveal behavior, not storage      |
| Leaking infrastructure | The domain imports a framework, sqlx, axum, serde, or an HTTP client type         | Keep the domain pure. Depend on ports, translate at the adapter boundary                |
| Primitive obsession    | Money, currency, and identifiers passed as raw `u32` and `String`                 | Promote each to a value object that validates and carries its own behavior              |

## Implementation order

1. Settle the ubiquitous language with the domain experts. Names come first, code follows.
2. Draw the bounded context and its neighbors on a context map before writing a type.
3. Model value objects and enums, validating in the constructor.
4. Name the domain errors after the invariants they guard.
5. Build the aggregate root: its constructor and intention-revealing transitions.
6. Add a domain service only when a rule spans aggregates and fits in none of them.
7. Define one repository port per aggregate root, in domain types.
8. Test behavior and invariants through the aggregate, never the accessors.

## In this service (the overlay)

The concepts above are universal. Their ENFORCED shape in this repository lives in rules, so this overlay points, it
does not restate. The rules are the single source of truth. Read the rule before you write.

- The aggregate root as the one entry point and the guarded transition are owned by the `rust-creating-application`
  skill. The `models/errors/services` module layout is owned by the `rust-architecture` rule. Named domain error
  placement and the context-grouping threshold are owned by `rust-hex-application-domain`.
- The domain service shape (a stateless struct, domain-only signatures, the only allowed collaborator is another domain
  service, ubiquitous-language name, Strategy and Specification as domain services) is carried by the
  `rust-creating-application` skill.
- Strategic DDD (one bounded context per service, the anti-corruption layer at the `driven/` boundary, referencing
  another context by id, the inward dependency rule, CQRS) is owned by `rust-architecture`.
- Naming (no pattern-name suffix, no generic technical verbs) is owned by `rust-code-style` (§ Design-pattern names, §
  Naming).

## Scaffold it

To generate the artifact (an aggregate, value object, enum, domain error, or domain service) rather than hand-roll it,
run the `rust-creating-application` skill. It carries the canonical templates and the completeness gate, and it loads
the rules above in priority order. For the strategic boundary work that anchors a new aggregate, read the contract
first: `openapi.yaml`, the docs pages under `docs/`, and the domain vocabulary in `AGENTS.md`.

## Reference documentation

| Reference               | Read it for                                                                                                                                                   |
|-------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------|
| references/tactical.md  | Entity, value object, aggregate and root, repository, domain service, factory, domain error, and invariant, each with an example, plus guarding versus validating |
| references/strategic.md | Ubiquitous language, bounded context, context map and its relationship patterns, anti-corruption layer, and shared kernel                                     |

## Sources

- Eric Evans, "Domain-Driven Design: Tackling Complexity in the Heart of Software" (Addison-Wesley, 2003), the Blue
  Book. Distilled in the free DDD Reference (https://www.domainlanguage.com/ddd/reference/).
- Vaughn Vernon, "Implementing Domain-Driven Design" (Addison-Wesley, 2013), the Red Book.
- Vaughn Vernon, "Effective Aggregate Design" (2011), the three-part essay
  (https://www.dddcommunity.org/library/vernon_2011/).
- Martin Fowler, "BoundedContext" (https://martinfowler.com/bliki/BoundedContext.html).
- Martin Fowler, "AnemicDomainModel" (https://martinfowler.com/bliki/AnemicDomainModel.html).
- The enforced specifics are this service's own, in the rules named under § In this service.
