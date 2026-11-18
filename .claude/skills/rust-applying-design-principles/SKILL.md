---
name: rust-applying-design-principles
description: Object-design principles beyond SOLID. Use when deciding if a type does too much, whether to add a trait or port now, whether to unify duplicate blocks, or where a behavior variant belongs.
---
<!-- Vendored from payment (.claude/skills/php-applying-design-principles/SKILL.md) on 2026-11-18. Adapted to this project: "class" became "type" and interfaces became traits, the SOLID landings name this repository's ports and adapters, the examples in the reference are Rust, and every route names the rust-* rules and skills. -->

# Design principles

This skill is an opinionated synthesis of the object-design principles a well-factored model leans on, distilled for a
Rust service. SOLID is one part of it, not the whole. The principles are language-agnostic, so the concepts use a
neutral vocabulary (Order, Customer, Account, Money, Report, Shape), and the good-versus-bad Rust example for each
principle lives in `references/principles.md`, drawn from this repository's code where it already shows the principle.
They are heuristics, not laws, and they pull against each other on purpose. DRY pushes toward sharing, KISS and the
wrong-abstraction caveat push back. YAGNI says wait, an architectural boundary says extract now. Each principle earns
its keep on code that has real variation, and over-applied it breeds indirection no reader thanks you for. Treat this as
a teaching reference, not dogma. Where a principle is operationalized and enforced in this repository, the § In this
service overlay routes you to the rule that owns it, it does not restate it.

## When to use (and when NOT to)

| Use when                                                             | Skip when                                                          |
|----------------------------------------------------------------------|--------------------------------------------------------------------|
| A type keeps growing and you suspect it does too much                | The type has one clear reason to change and is merely long         |
| Modules feel tangled, a change in one keeps forcing edits in others  | A single module changes alone, its neighbors are untouched         |
| A conditional over a variant grows an arm every time a kind is added | A small `match` inside the type that owns the variant, leave it    |
| Deciding whether to add a trait, a port, or a strategy now           | A future consumer that has not arrived, write the concrete code    |
| Two blocks look alike and you wonder whether to unify them           | The blocks encode the same knowledge with one obvious owner        |
| A method reaches through several values to get its work done         | A value object chains transformations and returns a peer each step |

**Start simple. Add a principle's machinery only when its smell actually appears, never preventively.**

## The principles at a glance

| Principle                       | One-line rule                                              | The smell it removes                                                      | The move                                               |
|---------------------------------|------------------------------------------------------------|---------------------------------------------------------------------------|--------------------------------------------------------|
| SOLID (SRP, OCP, LSP, ISP, DIP) | Five heuristics for a type and the seams around it         | A type doing too much, a match that grows, a leaky or dishonest contract | See the per-letter table under § SOLID                 |
| Separation of Concerns (SoC)    | Each section owns one concern, sections barely overlap     | One unit weaving a rule, a format, and I/O together                       | Give each concern its own seam                         |
| High Cohesion                   | What belongs together lives together, one task per module  | A grab-bag of unrelated fields and methods                                | Group by the thing that changes together               |
| Loose Coupling                  | Minimal knowledge between modules, change does not cascade | One edit forcing edits across many modules                                | Depend on an abstraction, narrow the surface           |
| Tell, Don't Ask                 | Tell a value to act, do not pull its state out to decide   | A getter whose result feeds an external branch                            | Move the decision onto the type that holds the data    |
| Law of Demeter (LoD)            | Talk to immediate neighbors, not a stranger's internals    | A train wreck chain `a().b().c().d()`                                     | Ask the nearest collaborator, let it delegate          |
| DRY                             | Each piece of knowledge has one authoritative owner        | The same rule coded in two places, drifting apart                         | One owner, callers tell it to act                      |
| KISS                            | The simplest thing that works, complexity earns its place  | Cleverness and layers a reader must decode                                | Remove the indirection, choose the plain solution      |
| YAGNI                           | Build it when a real consumer needs it, not before         | A hook or parameter added for an imagined future                          | Write the concrete code, extract on the second variant |
| Composition over inheritance    | Assemble behavior from collaborators, not deep hierarchies | Trait inheritance or a shared base used for code reuse, not a true is-a   | Hold a collaborator behind a trait                     |

## How they fit together

Two of these are the goal, the rest are means to reach it. High cohesion (what belongs together stays) and loose
coupling (modules barely lean on each other) are the two properties a sound design maximizes at once. The others serve
one or the other.

| Serves             | Principles                                           | How                                                                                        |
|--------------------|------------------------------------------------------|--------------------------------------------------------------------------------------------|
| High cohesion      | Separation of Concerns, Single Responsibility        | Each unit holds one concern and one reason to change, so its parts belong together         |
| Loose coupling     | Dependency Inversion, Tell-Don't-Ask, Law of Demeter | Depend on abstractions, talk only to neighbors, never reach into a stranger to decide      |
| Both, by restraint | DRY, KISS, YAGNI, Composition over inheritance       | Keep knowledge single-owned and code minimal, so fewer things bind and fewer things tangle |

Aim for high cohesion and loose coupling together. Chasing one alone backfires. Maximal decoupling with no cohesion
scatters one idea across many anemic units. Maximal cohesion with tight coupling welds the system into a block that
cannot change in one place.

## Cognitive load

Every principle here serves one measurable end, the cognitive load a reader carries to understand a piece of code.
Cognitive load is how much you must hold in your head at once. Working memory holds about four chunks, so once a block
forces more than that, the load has overflowed and the code is mis-shaped, not the reader.

Two kinds exist. Intrinsic load is inherent to the problem and cannot be removed. Extraneous load is imposed by the way
the code is written, and it is pure waste. Every move below cuts extraneous load.

| Move                | What it does                                                                                                          |
|---------------------|-----------------------------------------------------------------------------------------------------------------------|
| Name the condition  | A compound boolean becomes a predicate that says why (`calendar.is_toll_free(date)`), so the reader reads intent      |
| Fail fast           | A guard clause returns early on the exceptional case (`?`, `let ... else`), so the happy path is never nested         |
| Deep over shallow   | One well-named unit that hides its complexity beats many tiny ones the reader must stitch back together (Ousterhout)  |
| Obvious over clever | The plain construct a reader groks at a glance beats the smart one that saves a line and costs a re-read              |
| Keep it local       | What changes together stays close, so understanding one change does not send the eye jumping across files             |

The smell is the feeling of holding too much at once, a conditional you must simulate in your head, a name that hides
what it is, a stack of indirection where a plain call would read. The fix is always to push the load back toward four
chunks. KISS, YAGNI, and the wrong-abstraction caveat are this metric applied to complexity, abstraction, and
duplication. When two readings tie, choose the one a newcomer understands faster.

The mechanical face of this (a type that names itself, vertical paragraphs, a named intermediate over a nested call) is
enforced by the `rust-code-style` rule. This section is why beneath it.

## SOLID

The five type-level heuristics. In a hexagon they are not separate from ports and adapters, they are how the hexagon
stays sound (the sibling skill `rust-applying-hexagonal-architecture` carries the architecture in full).

| Letter | Principle             | One-line rule                                                  | The smell it removes                                    | The move                                          |
|--------|-----------------------|----------------------------------------------------------------|---------------------------------------------------------|---------------------------------------------------|
| S      | Single Responsibility | One reason to change, one actor it answers to                  | A type mixing a rule, a format, and storage             | Split along the axis of change                    |
| O      | Open-Closed           | Open to extension, closed to modification                      | A type match that grows an arm per new variant          | Add a new type or policy, edit nothing stable     |
| L      | Liskov Substitution   | An implementation is usable anywhere its trait is expected     | An implementation that fails or panics on a trait method | Honor the full contract, or split the trait       |
| I      | Interface Segregation | No client depends on methods it does not use                   | A wide trait forcing stubs and `unimplemented!`         | Split into cohesive, single-resource contracts    |
| D      | Dependency Inversion  | Depend on abstractions, not on concretions                     | A high-level type wired to a concrete low-level one     | Both sides depend on a trait the core owns        |

The good-versus-bad example for each letter is in `references/principles.md`.

| Letter | Where it lands in ports and adapters                                                                                                                 |
|--------|------------------------------------------------------------------------------------------------------------------------------------------------------|
| S      | Each artifact owns one job, a handler orchestrates one use case, an adapter speaks one resource over one protocol                                    |
| O      | A new variant arrives as a new port implementation (`CachedTaxRulesCatalog` beside `PostgresTaxRulesCatalog`), a new policy, or a new enum variant |
| L      | Every adapter honors its port's full contract, the core never learns which implementation it holds                                                   |
| I      | Ports stay cohesive around one resource (`TaxRulesCatalog` only finds), and CQRS already segregates the read side from the write side               |
| D      | The outbound port IS the inversion, the core declares it in domain terms and the adapter implements it, so the source dependency points inward       |

## Separation of Concerns

One concern (rule, format, persistence, transport, validation) per section, each with its own seam. Definition,
examples, and the trade-off: `references/principles.md` § Separation of Concerns.

## High Cohesion

One well-defined task per unit: if you cannot name a unit's job without the word "and", split it. Definition, examples,
and the trade-off: `references/principles.md` § High Cohesion.

## Loose Coupling

Minimal knowledge between modules, so change does not cascade. The coupling-level ladder (content, control, stamp,
data), examples, and the trade-off: `references/principles.md` § Loose Coupling.

## Tell, Don't Ask

Tell the type that holds the data to act, never pull its state out to decide. Definition, examples, and the
boundary-reads trade-off: `references/principles.md` § Tell, Don't Ask.

## Law of Demeter

Talk only to immediate collaborators, never through one value into another's internals. Definition, examples, and the
value-object trade-off: `references/principles.md` § Law of Demeter.

## DRY

One authoritative owner per piece of knowledge, and DRY governs knowledge, not text. The caveats (wrong abstraction,
rule of three) and examples: `references/principles.md` § DRY.

## KISS

Prefer the fewest moving parts: before adding an abstraction, layer, or knob, ask what breaks without it, and if nothing
breaks today, leave it out. Depth: `references/principles.md` § KISS.

## YAGNI

Build a generalization only when a real consumer demands it, never for an imagined future. Definition, examples, and the
boundary trade-off: `references/principles.md` § YAGNI.

## Composition over inheritance

Assemble behavior from held collaborators. Rust has no struct inheritance, so the temptation shows up as a supertrait
or a provided method used only to share code. Definition and examples: `references/principles.md` § Composition over
inheritance.

## Decision tree: is this really a Single Responsibility violation?

```
Count the distinct reasons this type would change.
│
├─ One actor, one concept, asks for every change here?
│   └─► NOT a violation. Size alone is not the test. Leave it.
│
└─ Two or more unrelated reasons drive change to the SAME type?
    (for example a business rule AND a wire format AND storage)
    │
    ├─ Do those parts share private state and always change together?
    │   └─► STILL one responsibility. Cohesion decides, not line count.
    │
    └─ Do they vary independently, for different reasons, on different days?
        └─► VIOLATION. Split along the axis of change:
            the rule   → the type that owns the data (aggregate, value object)
            the I/O    → an adapter behind an outbound port
            the view   → a separate response mapper
```

## Decision tree: should I extract an abstraction now, or wait?

```
You feel pressure to add a trait, a strategy, or a port.
│
├─ Is there a real SECOND implementation today
│   (a second adapter, a swappable implementation, a double you must inject to test)?
│   ├─ YES → EXTRACT NOW. The abstraction has a paying consumer.
│   └─ NO  → keep going ↓
│
├─ Does an architectural boundary REQUIRE the seam
│   (the core must not import a driver, driven, or framework type)?
│   ├─ YES → EXTRACT the port. The dependency rule drives it, not a guess.
│   └─ NO  → keep going ↓
│
└─ Are you adding it for a future that has not arrived?
    └─► WAIT (YAGNI, KISS). Write the concrete code now. Extract on the
        second real variant, never on the first imagined one.
```

## Decision tree: is this a coupling or a cohesion problem?

```
You are placing behavior, or judging a module's shape.
│
├─ Does everything in the module change together, for one reason?
│   ├─ NO  → cohesion is LOW. Split along the axis of change (SoC, SRP).
│   │        Move the stranger to the unit that owns its data.
│   └─ YES → keep going ↓
│
├─ Does the module reach into another's internals to work
│   (a().b().c(), or read a field then branch on it)?
│   ├─ YES → coupling is TOO TIGHT. Tell the neighbor to act
│   │        (Tell-Don't-Ask, Law of Demeter).
│   └─ NO  → keep going ↓
│
└─ Does the module name a concrete collaborator it cannot swap?
    ├─ YES → invert it. Depend on an abstraction the core owns (DIP).
    └─ NO  → cohesion high, coupling loose. Leave it.
```

## Anti-patterns

| Anti-pattern           | Problem                                                                                             | Fix                                                                                               |
|------------------------|-----------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------|
| God type               | One type accretes many responsibilities, every change risks the others, tests grow tangled          | Split by the axis of change, one reason to change per type (SRP, SoC)                             |
| Shotgun surgery        | One conceptual change forces edits scattered across many types                                      | Consolidate the knowledge into a single owner so the change is local (DRY, loose coupling)        |
| Feature envy           | A method is more interested in another value's data than its own, reaching for its fields to decide | Move the method onto the type that owns the data, then tell it to act (Tell-Don't-Ask, cohesion)  |
| Train wreck            | A chain reaches through several values to do work (`a().b().c().act()`)                             | Tell the nearest collaborator to do the work and answer (Law of Demeter)                          |
| Rigid type switch      | A `match` over a kind grows an arm per variant and is edited in many places                         | Make the behavior polymorphic, or move the `match` into the type that owns the variant (OCP)      |
| Fat interface          | A wide trait forces implementers to stub or fail methods they cannot serve                          | Split into cohesive, single-resource contracts (ISP)                                              |
| The wrong abstraction  | A shared helper grows flags or branches to serve callers with different rules                       | Inline it back into each caller, re-extract only the genuinely shared part (DRY)                  |
| Premature abstraction  | A trait or hook added before any second implementation exists                                       | Inline it back, extract only when the second real variant arrives (YAGNI)                         |
| Speculative generality | Parameters, hooks, generics, or config added for futures that never come                            | Delete the unused flexibility, keep the code shaped to what it does today (YAGNI, KISS)           |

## Heuristic order of moves

A pragmatic sequence for applying these principles during a change, simplest first.

1. Start concrete and simple. One type, straight-line code, no trait until a present consumer or a boundary needs it
   (KISS, YAGNI).
2. Separate concerns. When a unit weaves a rule, a format, and I/O, give each its own seam (SoC), and give every type
   one reason to change (SRP).
3. Raise cohesion, then check coupling. Group what changes together, and make sure a module does not reach into a
   stranger to do its work (high cohesion, Law of Demeter, Tell-Don't-Ask).
4. Close a growing variant switch. When a conditional over a variant grows a second arm outside its owner, make it
   polymorphic or move the `match` into the owner (OCP).
5. Invert I/O at the boundary. Declare a port in domain terms and push the technology to an adapter, so the core stays
   loosely coupled and runnable (DIP).
6. Keep each port cohesive and honest. Split the moment an adapter would stub or fail a method (ISP), and let every
   implementation honor its port's full contract (LSP).
7. Deduplicate knowledge, not text. One rule, one owner. Tolerate look-alike blocks that encode different rules (DRY).
8. When two principles collide, defer to the precedence order in the rule. Record any novel trade-off rather than
   resolving it silently.

## In this service (the overlay)

The principles above are universal. Their enforced shape in this repository lives in rules (the single source of truth).
This overlay routes you to them, it does not restate them. Read the rule before you write.

- **The precedence order when principles collide, the binding force, and the this-service homes for shared code** are
  owned by the `rust-design-principles` rule (the `.claude/rules/` file of that name, not this skill, the two share the
  concept name). Read that rule for the MUST versus SHOULD classification, this overlay does not restate it. The DRY
  limits (knowledge versus text, the rule of three, the wrong abstraction), complexity as a decision input, and the
  decision routing table stay in this skill (`references/principles.md` § DRY, plus the § Cognitive load and § Decision
  tree sections above). For the admitted homes for shared code and the correct-by-design duplication list, read that
  rule's § DRY. Consult that rule before applying any principle here.
- **The dependency rule and the four-layer contract** (the structural face of Dependency Inversion, Separation of
  Concerns drawn at the architecture scale, concrete outer-layer types never crossing inward) are owned by
  `rust-architecture`.
- **Naming and structure** (no pattern-name suffix, no generic technical names like `Manager` or `Util`, one concept per
  type) are owned by `rust-code-style`.

## Scaffold it

These principles are cross-cutting, not an artifact you generate. When applying one produces real structure, reach for
the matching builder skill.

- The outbound port and handler that embody Dependency Inversion, Interface Segregation, and high cohesion (the core
  declaring its own contracts): `rust-creating-application`.
- The driven adapter that implements a port and honors its full contract (Liskov), with the anti-corruption translation
  that keeps coupling loose: `rust-creating-driven`.
- When Open-Closed pressure means a behavior variant needs a home (Strategy, State, or a policy), pick the shape with
  `rust-applying-design-patterns`.

See also the sibling reference skills: `rust-applying-hexagonal-architecture` for ports and adapters (where Dependency
Inversion is the port and the layers are Separation of Concerns at scale), `rust-applying-ddd` for where a rule belongs
(the Single Responsibility and Tell-Don't-Ask call at the model level), and `rust-applying-design-patterns` for the
patterns that resolve Open-Closed and coupling pressure (Strategy, State, Decorator, Facade).

## Reference documentation

| Topic                                                                                                                                                                                                                                                                                                         | File                       |
|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------|
| Each principle in depth (the five of SOLID, plus Separation of Concerns, High Cohesion, Loose Coupling with the coupling-level ladder, Tell-Don't-Ask, Law of Demeter, DRY, KISS, YAGNI, composition), with a fuller good-versus-bad Rust example and the trade-off that bounds it, plus the source literature | `references/principles.md` |

## Sources

- Robert C. Martin, "Design Principles and Design Patterns" (2000), the paper that gathered the five
  (https://web.archive.org/web/20150906155800/http://www.objectmentor.com/resources/articles/Principles_and_Patterns.pdf).
- Robert C. Martin, "Clean Code: A Handbook of Agile Software Craftsmanship" (Prentice Hall, 2008), for cohesion,
  function size, and the smells.
- Robert C. Martin, "Clean Architecture: A Craftsman's Guide to Software Structure and Design" (Prentice Hall, 2017),
  the SOLID chapters.
- Robert C. Martin, "The Single Responsibility Principle" (2014)
  (https://blog.cleancoder.com/uncle-bob/2014/05/08/SingleReponsibilityPrinciple.html).
- Barbara Liskov and Jeannette Wing, "A Behavioral Notion of Subtyping" (ACM TOPLAS, 1994), the origin of the
  substitution principle.
- Gamma, Helm, Johnson, and Vlissides, "Design Patterns" (1994), for "favor object composition over class inheritance".
- Andrew Hunt and David Thomas, "The Pragmatic Programmer" (1999), for DRY, Tell-Don't-Ask, and the Law of Demeter
  (Lieberherr and Holland, 1989).
- Martin Fowler, "TellDontAsk" (https://martinfowler.com/bliki/TellDontAsk.html) and "Yagni"
  (https://martinfowler.com/bliki/Yagni.html).
- Martin Fowler, "Refactoring: Improving the Design of Existing Code" (2nd ed., 2018), for the code smells (feature
  envy, shotgun surgery, divergent change) and the DRY discussion.
- Sandi Metz, "The Wrong Abstraction" (2016) (https://sandimetz.com/blog/2016/1/20/the-wrong-abstraction).
- Edward Yourdon and Larry Constantine, "Structured Design" (1979), the origin of the coupling and cohesion ladders.
- Meilir Page-Jones, "The Practical Guide to Structured Systems Design" (2nd ed., 1988), for the coupling and cohesion
  taxonomy in practice.
- Edsger W. Dijkstra, "On the role of scientific thought" (1974), the origin of "separation of concerns".
- Artem Zinnatullin (zakirullin), "Cognitive Load Developer's Handbook", for the working-memory model and the
  intrinsic-versus-extraneous split (https://github.com/zakirullin/cognitive-load).
- John Ousterhout, "A Philosophy of Software Design" (2018), for deep versus shallow modules and defining complexity as
  what a reader must hold in their head.
- The enforced specifics are this service's own, in the rules named under § In this service.
