---
name: rust-applying-hexagonal-architecture
description: Ports and adapters (hexagonal) reference. Use when deciding which layer code belongs to, whether something is a port or an adapter, or which side (inbound or outbound) it sits on.
---
<!-- Vendored from payment (.claude/skills/php-applying-hexagonal-architecture/SKILL.md) on 2026-11-18. Adapted to this project: the examples and the generic structure are Rust over this repository's src/ tree, the composition root is the binary (src/main.rs, src/routes.rs) instead of a DI container, interfaces became traits, the folder skeleton routes to rust-architecture instead of the skipped skeleton skill, domain events and publishers were dropped from the tables, and every route names the rust-* rules and skills. -->

# Hexagonal architecture (ports and adapters)

This skill is an opinionated synthesis of the ports and adapters pattern, distilled for a Rust service. The architecture
is universal and language-agnostic, so the concepts use a domain-neutral vocabulary (Order, Customer, Account, Money)
and the short Rust examples come from this repository's own code wherever an equivalent exists. For the canonical answer
to any single question, follow the primary source named in § Sources (Cockburn for the shape, Martin for the dependency
direction). This is a teaching reference, not dogma, and not a restatement of the enforced rules. Where a specific is
enforced in this repository, the § In this service overlay routes you to the rule that owns it.

## When to use (and when NOT to)

| Use when                                                     | Skip when                                                            |
|--------------------------------------------------------------|----------------------------------------------------------------------|
| Deciding which layer a type belongs to                       | The placement is obvious and already covered by a layer rule         |
| Cannot tell a port from an adapter, or inbound from outbound | Only a literal file shape is needed (run the scaffold skill instead) |
| A dependency seems to point the wrong way (outward)          | A tiny script with no business rule and no swappable I/O             |
| A technology needs to be swappable behind a contract         | The "technology" is the language itself, not an external dependency  |
| Reviewing whether the core stays free of infrastructure      | The change touches only one adapter's internals, behind its port     |

**Start simple. Evolve complexity only when needed.** A port exists when a real consumer or a real boundary requires it,
never preventively.

## The dependency rule (and the runnable-core test)

Source-code dependencies point inward only, and the core must run with every adapter replaced by a double (no UI, no
database). The full rule, the runnable-core test, and the inversion mechanism, each with a Rust example:
`references/ports-and-adapters.md` § The dependency rule and § The runnable-core test.

## Concept boundaries

| Block                   | Primary question                               | Use it for                                                            | Do not treat as                                        |
|-------------------------|------------------------------------------------|-----------------------------------------------------------------------|--------------------------------------------------------|
| Domain                  | What is always true, regardless of technology? | Entities, value objects, aggregates, domain services                  | A place for SQL, HTTP, framework, or crate types       |
| Application (use cases) | What does the system do, step by step?         | Handlers orchestrating one use case, the ports they depend on         | A home for business rules (those belong to the domain) |
| Inbound port            | How does the outside ask the core to act?      | The entry contract a driver calls to drive one use case               | An endpoint, a request type, or a transport type       |
| Outbound port           | What does the core need from the outside?      | The capability contract an adapter implements                         | A database, an HTTP client, or a leaked crate type     |
| Driver adapter          | How does a real transport reach the core?      | Endpoints, CLI commands, consumers                                    | The place to make a domain decision                    |
| Driven adapter          | How is a need actually fulfilled?              | Repositories, catalogs, gateways, caches, clocks                      | A concrete type the core imports directly              |
| Composition root        | Where are the wires soldered?                  | The binary building the object graph, choosing the concrete adapters | A home for business logic or per-request decisions     |

## Decision trees

### Where does this code go (domain vs application vs adapter)?

```
Does the code express a business rule or an invariant?
├─ YES → Does it touch I/O (DB, HTTP, queue, clock, env, framework)?
│        ├─ NO  → DOMAIN (entity, value object, aggregate, domain service)
│        └─ YES → It is not pure domain. Split it:
│                 the rule  → DOMAIN
│                 the I/O   → ADAPTER, behind an outbound port
└─ NO  → Does it orchestrate a use case (load, call the domain, persist)?
         ├─ YES → APPLICATION (a handler behind an inbound port)
         └─ NO  → Does it speak a concrete technology (HTTP, SQL, broker, env)?
                  ├─ YES → ADAPTER (Driver if inbound, Driven if outbound)
                  └─ NO  → It is a contract. A PORT (a trait in the core)
```

### Is this a port or an adapter?

```
Is it a trait the core owns and the core depends on?
├─ YES → PORT (lives inside the application core)
│        It names a capability in business terms, with no transport words.
└─ NO  → Is it a concrete type that speaks one specific technology?
         ├─ YES → ADAPTER (lives outside the core)
         └─ NO  → Neither. A pure rule is DOMAIN, a use case is a HANDLER.
```

### Is this port inbound or outbound?

```
Which way does control cross this port?
├─ The outside DRIVES the core through it
│   (a request enters: HTTP endpoint, CLI, message consumer)
│   └─ INBOUND port (driving, primary)
│      Implemented BY the core (a handler implements it).
└─ The core DRIVES the outside through it
    (the core needs something: persistence, a provider, the clock)
    └─ OUTBOUND port (driven, secondary)
       Implemented OUTSIDE the core (an adapter implements it).
```

### Is this adapter a driver or a driven?

```
Which side of the port does the adapter sit on?
├─ It RECEIVES input and then calls an inbound port
│   (endpoint, consumer, CLI command)
│   └─ DRIVER adapter (primary, the inbound side)
│      Depends on an inbound port. Never on the core internals.
└─ It IMPLEMENTS an outbound port and speaks a technology
    (repository, catalog, gateway / API client, cache, clock)
    └─ DRIVEN adapter (secondary, the outbound side)
       Implements an application outbound port. Owns the transport.
```

## Generic structure

A neutral shape with `{placeholders}`, mirrored by this repository's `src/` tree. The concrete module skeleton is owned
by the `rust-architecture` rule, so treat the tree below as the idea, not the literal layout.

```
src/
├── application/                   # the application core (zero outward dependencies)
│   ├── domain/                    # entities, value objects, aggregates, services
│   │   └── models/{aggregate}/
│   ├── ports/
│   │   ├── inbound/               # driving contracts: {UseCase}ing traits
│   │   └── outbound/              # driven contracts: {Aggregate}s, {Resource}Catalog
│   ├── commands/                  # {use_case}.rs, the input to one use case
│   └── handlers/                  # {use_case}ing_handler.rs, implements an inbound port
├── driver/                        # primary / driving adapters (call inbound ports)
│   └── {transport}/               # http endpoints, CLI, consumers
├── driven/                        # secondary / driven adapters (implement outbound ports)
│   └── {technology}/              # postgres, cache
├── query/{context}/               # read slices, the CQRS read side
├── routes.rs                      # mounts the driver and query endpoints on one router
└── main.rs                        # the composition root, wires concrete adapters to ports
```

The port shapes (inbound and outbound), the handler that implements the inbound port, the driver and driven adapters
around them, and the composition root that wires them are in `references/ports-and-adapters.md`.

## Building blocks

| Building block        | Role                                                | Lives in                  |
|-----------------------|-----------------------------------------------------|---------------------------|
| Entity / Aggregate    | Identity plus the invariants of its boundary        | Domain (core)             |
| Value object          | An immutable value and the rules that bind it       | Domain (core)             |
| Domain service        | One rule that spans aggregates, with no I/O         | Domain (core)             |
| Inbound port (trait)  | The entry contract for one use case                 | Application (core)        |
| Command               | The typed input to one use case                     | Application (core)        |
| Handler               | Orchestrates one use case over the domain and ports | Application (core)        |
| Outbound port (trait) | A capability the core needs, in domain terms        | Application (core)        |
| Driver adapter        | Receives a transport, calls an inbound port         | Driver (outside, driving) |
| Driven adapter        | Implements an outbound port, owns the transport     | Driven (outside, driven)  |
| Composition root      | Wires concrete adapters to the ports                | The binary (outside)      |

## Anti-patterns

| Anti-pattern                                                                                     | Problem                                                                                                         | Fix                                                                                                           |
|--------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------|
| Domain imports infrastructure (sqlx, axum, serde, an HTTP client crate)                          | The core can no longer run without that technology, the dependency rule is broken, the runnable-core test fails | Declare an outbound port in the core, move the technology into a driven adapter behind it                     |
| An endpoint calls a repository directly                                                          | The use case has no home, orchestration and transport tangle, the core cannot run without HTTP                  | Route the endpoint through an inbound port to a handler, let the handler call the outbound port               |
| A concrete adapter type crosses the boundary (a handler names `PostgresTaxRulesCatalog`)         | The core is welded to one adapter, swapping it means editing the core                                           | Depend on the port trait (a generic bound), let the composition root inject the concrete adapter              |
| A port exposes infrastructure types (a method takes a `PgPool`, returns a raw row or a response) | The technology leaks through the contract, the inversion is only cosmetic                                       | Express the port in domain types (value objects, aggregates, read models), serialize inside the adapter       |
| An adapter holds a business rule                                                                 | The rule hides outside the domain, two adapters drift, the knowledge has no single owner                        | Move the rule onto the owning aggregate, value object, or domain service, leave the adapter to translate only |
| Anemic core, behavior scattered in handlers and adapters                                         | The "domain" is data bags, real logic lives in the outer layers                                                 | Push behavior inward, tell the aggregate to act rather than asking for its state                              |
| The driver names the handler type instead of the inbound port                                    | The driver couples to one concrete implementation, not a contract                                               | Bound the driver on the inbound port trait, inject the handler through the composition root                   |

## Implementation order

1. Model the domain first: entities, value objects, aggregates, domain services. No I/O at all.
2. Name the use case as an inbound port and define its command.
3. Discover the outbound ports the use case needs (persistence, providers, the clock), and declare them in the core in
   domain terms.
4. Write the handler that implements the inbound port and orchestrates over the domain and the outbound ports.
5. Implement the driven adapters behind the outbound ports (repository, catalog, cache), with the anti-corruption
   translation at the boundary.
6. Implement the driver adapter that builds the command from transport input and calls the inbound port.
7. Wire everything at the composition root, where `src/main.rs` picks the concrete adapters.
8. Verify the dependency rule: the core compiles and its use cases run with no outer-layer import and no real database.

## In this service (the overlay)

The concepts above are universal. Their enforced specifics in this repository live in rules (the single source of
truth). This overlay only routes you to them, it does not restate their content.

- **The dependency rule, the four-layer contract, CQRS, the composition root, and the anti-corruption boundary** are
  owned by `rust-architecture`. Read it before creating any file under `src/`.
- **Inbound ports** (the `-ing` entry contract, its single method, the one handler per port) are carried by the
  `rust-creating-application` skill.
- **Outbound ports** (hexagon-only parameter types, optional reads with caller-side not-found, no infrastructure
  leakage, repository and catalog naming) are carried by the `rust-creating-application` skill.
- **Driven (outbound, secondary) adapters** (adapter-kind naming) are owned by `rust-hex-driven`. The
  driver-versus-driven criterion is owned by the `rust-creating-driven` and `rust-creating-driver` skills.
- **Driver (inbound, primary) HTTP adapters** (the `src/driver/http/` layout, the error stack) are owned by
  `rust-hex-driver-http`.
- **The Dependency Inversion Principle** as operationalized here (concrete outer-layer types never cross inward) is
  owned by `rust-architecture` (§ Dependency rule).

## Scaffold it

Once you know where the code goes, generate the artifact with the matching builder skill:

- The application core (domain artifacts plus the use case: command, inbound port, handler, outbound port):
  `rust-creating-application`.
- A driver (inbound, primary) HTTP adapter (endpoint, request, response): `rust-creating-driver`.
- A driven (outbound, secondary) adapter (repository, catalog, cache decorator): `rust-creating-driven`.

## Reference documentation

| Topic                                                                                                                                                                | File                               |
|----------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------|
| The dependency rule, the runnable-core test, the port and adapter taxonomy, the composition root, and the anti-corruption boundary, each with a Rust example from `src/` | `references/ports-and-adapters.md` |

## Sources

- Cockburn, Alistair. "Hexagonal Architecture" (2005). https://alistair.cockburn.us/hexagonal-architecture/
- Martin, Robert C. "The Clean Architecture" (2012).
  https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html
- Martin, Robert C. "Clean Architecture: A Craftsman's Guide to Software Structure and Design", Prentice Hall (2017).
- Martin, Robert C. "The Dependency Inversion Principle", C++ Report (1996). The inversion that makes the outbound port
  point inward.
