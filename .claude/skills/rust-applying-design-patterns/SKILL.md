---
name: rust-applying-design-patterns
description: Applies a Gang of Four design pattern (all 23). Use when choosing whether a pattern fits, or confirming its placement and naming in any layer. Triggers include Strategy, Adapter, Decorator, Observer, Builder.
---
<!-- Vendored from payment (.claude/skills/php-applying-design-patterns/SKILL.md) on 2026-11-18. Adapted to this project: the "already provided" table names the Rust language, the composition root, and the crates instead of the DI container and the tiny-blocks facades, the canonical homes are the src/ modules of this repository (Observer has none because this service records no domain events, Chain of Responsibility is tower middleware), and the naming owner is rust-code-style. -->

# Design patterns (Gang of Four)

A reference for the 23 GoF patterns. A pattern is a named solution to a recurring design problem, a shape, never a type
name. This skill covers every pattern generically (intent, the problem it solves, the entry signal) and then how each
lands in this hexagonal Rust service. It is intent-triggered, consulted while CHOOSING a pattern, never injected into
every file.

This skill is not a mandate to use patterns. Most code needs none. Reach for one only when its entry signal appears,
then keep the type named for the business, not the pattern (owned and enforced by `rust-code-style` § Design-pattern
names).

## When to use (and when NOT to)

| Use when                                                                        | Skip when                                                                       |
|---------------------------------------------------------------------------------|---------------------------------------------------------------------------------|
| A recurring design problem matches a known pattern's intent                     | A one-off with no recurrence, write the straight-line code                      |
| A growing conditional, a duplicated rule, or repeated wiring you want to remove | No entry signal yet, a pattern added preventively is speculative complexity     |
| Confirming the placement and naming of a pattern already present                | The language or a crate already provides it (see § Already provided)            |
| The variant arms have grown into full algorithms                                | A small `match` inside its owner still reads fine, leave it there               |

**Start simple. Reach for a pattern only when its signal appears, never preventively.**

## The 23 patterns

Intent and entry signal for each. The full structure, a Rust example, the trade-off, and the this-service note are in
`references/catalog.md`.

### Creational (construction)

| Pattern          | Intent                                       | Reach for it when                                                     |
|------------------|----------------------------------------------|-----------------------------------------------------------------------|
| Singleton        | One instance, global access                  | Almost never by hand, the composition root owns lifecycle             |
| Factory Method   | Defer which type to instantiate to a method  | Construction has a meaningful name or branches by input               |
| Abstract Factory | Produce a family of related values together  | One call must return a matched set that cannot be wired independently |
| Builder          | Assemble a complex value step by step        | Many optional parts or a graph with sensible defaults (tests)         |
| Prototype        | Create by copying an existing instance       | Copy is cheaper than construction, almost never by hand               |

### Structural (composition)

| Pattern   | Intent                                            | Reach for it when                                                   |
|-----------|---------------------------------------------------|---------------------------------------------------------------------|
| Adapter   | Make an incompatible interface fit a port         | Wrapping an external API or crate behind your own contract          |
| Bridge    | Split abstraction from implementation on two axes | Both an abstraction and its implementation vary independently       |
| Composite | Treat a part-whole tree uniformly                 | A genuine recursive hierarchy handled the same at every node        |
| Decorator | Add behavior by wrapping, same trait              | A cross-cutting concern (cache, retry, log) wraps without editing   |
| Facade    | One simple entry over a complex subsystem         | Hiding a multi-step or multi-type subsystem behind one call         |
| Flyweight | Share immutable instances to save memory          | A huge number of values share intrinsic state                       |
| Proxy     | A stand-in controlling access to a target         | Access guarding, lazy loading, or a remote stand-in, same trait     |

### Behavioral (responsibility and communication)

| Pattern                 | Intent                                                   | Reach for it when                                                  |
|-------------------------|----------------------------------------------------------|--------------------------------------------------------------------|
| Chain of Responsibility | Pass a request along handlers until one acts             | A pipeline where each stage may handle or pass on                  |
| Command                 | Turn a request into a value                              | Queue, log, undo, or decouple invoker from receiver                |
| Interpreter             | Evaluate sentences in a small language                   | A recurring grammar or expression to evaluate                      |
| Iterator                | Traverse a collection without exposing it                | Custom traversal, usually the language already gives it            |
| Mediator                | Centralize how a set of values interact                  | Many-to-many coupling you want to collapse                         |
| Memento                 | Capture and restore state without breaking encapsulation | Undo or snapshot of a value's internal state                       |
| Observer                | Notify dependents when state changes                     | One change must fan out to many reactions                          |
| State                   | Behavior changes with internal state                     | A type behaves differently across several states and operations    |
| Strategy                | Interchangeable algorithm chosen at runtime              | A closed set of full-algorithm variants behind one trait           |
| Template Method         | Fix an algorithm skeleton, defer steps                   | A stable sequence with a few varying steps                         |
| Visitor                 | Add an operation to a structure without changing it      | A new operation over a stable type hierarchy, at a boundary        |

## Problem to pattern (quick index)

| The signal you feel                                             | Pattern                            |
|-----------------------------------------------------------------|------------------------------------|
| A big conditional choosing between full algorithms              | Strategy                           |
| A business rule (eligibility, validity) reused across use cases | Specification (a Strategy variant) |
| Behavior diverges per status across several operations          | State                              |
| A cross-cutting concern must wrap an existing component         | Decorator (or Proxy for access)    |
| An external API shape does not match my port                    | Adapter                            |
| A new operation must traverse a stable structure                | Visitor                            |
| One change must notify many reactions                           | Observer                           |
| A request must be queued, logged, or undone                     | Command                            |
| A complex value needs flexible step-by-step assembly            | Builder                            |
| A recursive part-whole tree handled uniformly                   | Composite                          |

## Already provided (do not hand-build in this service)

The language, the composition root, or a crate already gives these. Reaching for them by hand is the anti-pattern.

| Pattern   | Use instead                                                                                           |
|-----------|-------------------------------------------------------------------------------------------------------|
| Singleton | `src/main.rs` builds each adapter once and shares it through `Arc`, constructor injection only        |
| Prototype | `#[derive(Clone)]` (or `Copy` for a small value such as `Amount`), never a hand-written copy method   |
| Iterator  | `Iterator`, `IntoIterator`, and `FromIterator`, as `Passages` implements them                         |
| Bridge    | Ports and adapters already decouple abstraction from implementation                                   |
| Flyweight | `Copy` enum variants (`VehicleType`), and `Arc` for a shared aggregate (`Arc<TaxRules>`)              |
| Facade    | The crates already wrap their subsystems (the axum `Router`, the sqlx `PgPool`)                       |

## In this service (the overlay)

The patterns above are universal. Their placement and naming here are opinionated:

- Canonical homes (reuse, do not reinvent): Command is `src/application/commands/`, Chain of Responsibility is tower
  middleware layered on the axum router, Factory Method is the associated constructors on a type, Adapter is the
  `driver/` and `driven/` layers themselves, Decorator and Proxy live in `driven/` over a port
  (`CachedTaxRulesCatalog`), Strategy, Specification, and State live in the domain (a domain service or a status enum).
  Observer has no home: this service records no domain events and keeps no in-process observer registry.
- Naming: no pattern-name suffix, the type keeps its business name. Owned by `rust-code-style`.
- Escalation thresholds (start simple): a status enum before per-state types (State), a `match` moved into its owner
  before it is lifted to a Strategy, a combinator only on the third real composition (Specification). The detail is in
  `references/catalog.md`.

## Scaffold it

There is no generator for a pattern. A pattern is a shape, not a type to stamp out. To apply one, read its entry in
`references/catalog.md` first (intent, structure, a minimal example, the trade-off, and the this-service note), then
write the type in the canonical home that § In this service names for it. Keep the type named for the business, never
the pattern (owned by `rust-code-style` § Design-pattern names). Reach for the pattern only when its entry signal
appears, never preventively.

## Catalog

`references/catalog.md` carries every pattern's intent, structure, a minimal Rust example, the trade-off that bounds
it, and its this-service note. Read the catalog entry before writing the type.

## Sources

- Gamma, Helm, Johnson, Vlissides, "Design Patterns: Elements of Reusable Object-Oriented Software" (1994).
- Refactoring Guru, "Design Patterns" (https://refactoring.guru/design-patterns).
- The naming invariant and the dependency rules are this service's own, in `rust-code-style` and `rust-architecture`.
