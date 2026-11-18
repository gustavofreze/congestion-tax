<!-- Vendored from payment (.claude/skills/php-applying-design-patterns/references/catalog.md) on 2026-11-18. Adapted to this project: every example is Rust (Decorator, Command, Builder, and Iterator quote CachedTaxRulesCatalog, CalculateCongestionTax, TaxRulesBuilder, and Passages), interfaces became traits and abstract bases became traits with provided methods, the this-service notes name the composition root, the crates, and the src/ modules, Observer has no home because this service records no domain events, the event-history clause of Memento was dropped for the same reason, and Template Method notes that Rust has no inheritance. -->

# GoF pattern catalog

This is the pattern catalog for the `rust-applying-design-patterns` skill. It covers all 23 Gang of Four patterns
generically (intent, entry signal, structure, a minimal example, the trade-off that bounds it), then a one line note on
how each lands in this hexagonal Rust service. The examples use domain-neutral names on purpose, they teach the pattern
shape only, except where this repository already holds the pattern and the example quotes it. The this-service overlay
(canonical homes, naming, escalation thresholds) is the authority for placement here. Read the entry before writing the
type. A pattern is a shape, never a type name. The type keeps its business name (the naming invariant is owned by
`rust-code-style`, § Design-pattern names).

## Contents

- This-service home at a glance
- Creational
- Structural
- Behavioral
- Sources

## This-service home at a glance

| Family     | Pattern                 | In this service                                                            |
|:-----------|:------------------------|:---------------------------------------------------------------------------|
| Creational | Singleton               | Provided by the composition root, one instance shared through `Arc`        |
| Creational | Factory Method          | Associated constructors on the type (verb, then `new`, then `FromStr`)     |
| Creational | Abstract Factory        | `driven/`, rare, usually a second adapter behind a port instead            |
| Creational | Builder                 | Tests only, Test Data Builder under `tests/support/builders/`              |
| Creational | Prototype               | Provided by `#[derive(Clone)]` and `Copy`                                  |
| Structural | Adapter                 | The `driver/` and `driven/` layers themselves                              |
| Structural | Bridge                  | Provided by Ports and Adapters                                             |
| Structural | Composite               | Rare, a `query/` hierarchical read response                                |
| Structural | Decorator               | `driven/` over a port, concern prefix, port name kept                      |
| Structural | Facade                  | Provided by the crates (the axum `Router`, the sqlx `PgPool`)              |
| Structural | Flyweight               | Provided by `Copy` enum variants and `Arc` sharing                         |
| Structural | Proxy                   | `driven/` over a port, lazy init by the crate or `std::sync::LazyLock`     |
| Behavioral | Chain of Responsibility | Tower middleware on the axum router only                                   |
| Behavioral | Command                 | `src/application/commands/`, one handler wired at the composition root     |
| Behavioral | Interpreter             | No special stance                                                          |
| Behavioral | Iterator                | Provided by `Iterator`, `IntoIterator`, and `FromIterator`                 |
| Behavioral | Mediator                | No special stance                                                          |
| Behavioral | Memento                 | Rare, in-memory aggregate undo only                                        |
| Behavioral | Observer                | No home, this service records no domain events                             |
| Behavioral | State                   | Status enum first, per-state types past the threshold                      |
| Behavioral | Strategy                | Domain service under `src/application/domain/services/` (Specification too) |
| Behavioral | Template Method         | No special stance, prefer composition                                      |
| Behavioral | Visitor                 | A `query/` projection or reporting adapter, never the domain               |

## Creational

### Singleton

- **Intent**: Ensure a type has exactly one instance and give a global access point to it.
- **Entry signal**: You feel tempted to reach a shared value through a global accessor from many places.
- **Structure**: A `static` holds the only instance and an associated function hands it out.
- **Example**:

```rust
use std::sync::OnceLock;

pub struct Logger;

static INSTANCE: OnceLock<Logger> = OnceLock::new();

impl Logger {
    pub fn instance() -> &'static Self {
        INSTANCE.get_or_init(|| Self)
    }
}
```

- **Trade-off**: Global state, hidden coupling, hard to test (you cannot substitute a double). When one instance is all
  you need, let the composition root build it once and inject it.
- **In this service**: Already provided. `src/main.rs` builds each adapter once and hands it to its consumers (sharing
  it through `Arc` where several need it). Use constructor injection only, never a `static`, `OnceLock`, or `LazyLock`
  instance.

### Factory Method

- **Intent**: Defer to a method which concrete shape to instantiate, giving the construction a name.
- **Entry signal**: Construction carries a meaningful name or branches on its input, so a struct literal reads poorly.
- **Structure**: A creator exposes a function that returns the product, the function decides the concrete shape.
- **Example**:

```rust
pub struct Notification {
    body: String,
    channel: Channel,
}

impl Notification {
    #[must_use]
    pub const fn email(body: String) -> Self {
        Self {
            body,
            channel: Channel::Email,
        }
    }

    #[must_use]
    pub const fn sms(body: String) -> Self {
        Self {
            body,
            channel: Channel::Sms,
        }
    }
}
```

- **Trade-off**: One more indirection. When construction is a plain `new` with no name and no branch, the indirection
  earns nothing.
- **In this service**: Canonical, as associated functions on the type itself (a business verb when one fits, then
  `new`, then the `FromStr` or `TryFrom` traits for parsing raw input), as `SingleChargeWindow::of_minutes` and the
  `FromStr` of `Passage` and `VehicleType` do. The GoF subclass-decides form has no Rust equivalent, because Rust has no
  inheritance.

### Abstract Factory

- **Intent**: Produce a family of related values together without naming their concrete types.
- **Entry signal**: One call must return a matched set of collaborators that cannot be wired independently.
- **Structure**: A factory trait declares one creator per product, each concrete factory builds one consistent family.
- **Example**:

```rust
pub trait WidgetFactory {
    fn button(&self) -> Box<dyn Button>;

    fn checkbox(&self) -> Box<dyn Checkbox>;
}

pub struct DarkWidgetFactory;

impl WidgetFactory for DarkWidgetFactory {
    fn button(&self) -> Box<dyn Button> {
        Box::new(DarkButton)
    }

    fn checkbox(&self) -> Box<dyn Checkbox> {
        Box::new(DarkCheckbox)
    }
}
```

- **Trade-off**: Many types for a small gain, and a new product means touching every factory. When the composition root
  can build each member independently, a factory type only duplicates that wiring.
- **In this service**: Lives in `driven/`, but rarely needed. Most provider variation is a second adapter behind the
  same port selected in `src/main.rs`, reach for the factory only when one call must return a matched set.

### Builder

- **Intent**: Assemble a complex value step by step, separating construction from the final representation.
- **Entry signal**: Many optional parts, or a value graph with sensible defaults you want to override a few at a time.
- **Structure**: A builder accumulates parts through consuming steps that return `Self`, a final `build` returns the
  assembled product.
- **Example** (the Test Data Builder of this repository, `tests/support/builders/tax_rules_builder.rs`):

```rust
pub(crate) struct TaxRulesBuilder {
    daily_maximum: u32,
}

impl TaxRulesBuilder {
    pub(crate) const fn gothenburg() -> Self {
        Self { daily_maximum: 60 }
    }

    pub(crate) fn build(self) -> TaxRules {
        TaxRules::new(
            swedish_kronor(),
            [gothenburg_period(
                date(2013, 1, 1),
                date(2013, 12, 31),
                self.daily_maximum,
            )],
        )
        .expect("a single period overlaps no other")
    }

    pub(crate) const fn with_daily_maximum(mut self, daily_maximum: u32) -> Self {
        self.daily_maximum = daily_maximum;
        self
    }
}
```

- **Trade-off**: An extra type and indirection. For a value with few required fields, a `new` with those arguments
  already covers the need, so a builder is overkill.
- **In this service**: Tests only, as the Test Data Builder shape under `tests/support/builders/`. Production
  construction uses `new` plus the domain factories, a Builder for a domain type is prohibited.

### Prototype

- **Intent**: Create new values by copying an existing instance rather than constructing from scratch.
- **Entry signal**: Copying a configured instance is cheaper or simpler than building one anew.
- **Structure**: A prototype exposes a copy operation that returns a duplicate of itself.
- **Example**:

```rust
pub struct Document {
    blocks: Vec<String>,
}

impl Document {
    #[must_use]
    pub fn duplicate(&self) -> Self {
        Self {
            blocks: self.blocks.clone(),
        }
    }
}
```

- **Trade-off**: Deep versus shallow copy is easy to get wrong with shared references (cloning an `Arc` field shares the
  value instead of copying it). When construction is cheap and explicit, copying hides intent.
- **In this service**: Already provided. Value objects derive `Clone` (and `Copy` when small, as `Amount` and
  `Passage` do), so a copy is `.clone()` or a plain move, never a hand-written copy method.

## Structural

### Adapter

- **Intent**: Convert one interface into another so an existing type fits the contract a caller expects.
- **Entry signal**: You wrap an external API or crate whose shape does not match your own port.
- **Structure**: An adapter implements your target trait and translates each call onto the adaptee.
- **Example**:

```rust
pub trait Mailer {
    fn send(&self, to: &str, body: &str);
}

pub struct VendorMailer {
    client: VendorSdkClient,
}

impl Mailer for VendorMailer {
    fn send(&self, to: &str, body: &str) {
        self.client.dispatch(VendorMessage {
            recipient: to,
            text: body,
        });
    }
}
```

- **Trade-off**: One more layer to maintain. When the external interface already matches your needs, an adapter is just
  a passthrough.
- **In this service**: Canonical. The `driver/` (inbound) and `driven/` (outbound) layers are themselves the Adapter
  (`PostgresTaxRulesCatalog` adapts sqlx to `TaxRulesCatalog`), the adapter kinds and naming are the driven layer's
  concern.

### Bridge

- **Intent**: Split an abstraction from its implementation so the two can vary independently.
- **Entry signal**: Both an abstraction and its implementation grow along separate axes and a type explosion looms.
- **Structure**: An abstraction holds an implementor behind a trait and delegates the varying work to it.
- **Example**:

```rust
pub trait Renderer {
    fn draw_circle(&self, radius: f64) -> String;
}

pub trait Shape {
    fn draw(&self) -> String;
}

pub struct Circle<R: Renderer> {
    radius: f64,
    renderer: R,
}

impl<R: Renderer> Shape for Circle<R> {
    fn draw(&self) -> String {
        self.renderer.draw_circle(self.radius)
    }
}
```

- **Trade-off**: Upfront indirection that pays off only when both axes truly vary. With a single implementation, it is
  needless structure.
- **In this service**: Satisfied. Ports and Adapters already decouple abstraction from implementation along the I/O
  axis, there is no separate Bridge construct.

### Composite

- **Intent**: Compose values into part-whole trees and treat leaves and compositions uniformly.
- **Entry signal**: A genuine recursive hierarchy must be handled the same way at every node.
- **Structure**: A component trait is implemented by both leaves and composites, the composite holds children of the
  same trait.
- **Example**:

```rust
pub trait FileNode {
    fn size(&self) -> u64;
}

pub struct FileLeaf {
    bytes: u64,
}

impl FileNode for FileLeaf {
    fn size(&self) -> u64 {
        self.bytes
    }
}

pub struct Folder {
    children: Vec<Box<dyn FileNode>>,
}

impl FileNode for Folder {
    fn size(&self) -> u64 {
        self.children.iter().map(|child| child.size()).sum()
    }
}
```

- **Trade-off**: A uniform interface can blur leaf-only and composite-only operations. When nesting is shallow or fixed,
  flat rows plus a parent reference are simpler, and a closed set of node kinds reads better as one enum with a
  recursive variant.
- **In this service**: Rare. The legitimate case is a hierarchical read response assembled in a `query/` projection from
  flat rows, inside the domain it must respect the aggregate boundary.

### Decorator

- **Intent**: Attach added behavior to a value by wrapping it, keeping the same trait.
- **Entry signal**: A cross-cutting concern (cache, retry, log, metrics) must wrap a component without editing it.
- **Structure**: A decorator implements the component trait and holds an inner component it delegates to, adding
  behavior around the call.
- **Example** (this repository's cache over the catalog, `src/driven/cache/cached_tax_rules_catalog.rs`):

```rust
pub struct CachedTaxRulesCatalog<C, K> {
    clock: K,
    origin: C,
    entries: RwLock<HashMap<City, CachedTaxRules>>,
    time_to_live: Duration,
}

impl<C: TaxRulesCatalog, K: MonotonicClock> TaxRulesCatalog for CachedTaxRulesCatalog<C, K> {
    async fn find(&self, city: &City) -> Result<Option<Arc<TaxRules>>, TaxRulesUnavailable> {
        if let Some(rules) = self.fresh(city) {
            return Ok(Some(rules));
        }

        let found = self.origin.find(city).await?;

        if let Some(rules) = &found {
            self.keep(city, rules);
        }

        Ok(found)
    }
}
```

- **Trade-off**: Many small wrappers can be hard to trace at runtime. When only one implementation will ever exist, fold
  the behavior in directly.
- **In this service**: Lives in `driven/` over a port, concern as prefix, port name kept (`CachedTaxRulesCatalog` over
  `TaxRulesCatalog`). Wiring is composition in `src/main.rs` (outermost concern first), no pattern-name suffix.

### Facade

- **Intent**: Provide one simple entry point over a complex subsystem.
- **Entry signal**: Callers must orchestrate several values or steps you would rather hide behind a single call.
- **Structure**: A facade exposes a small interface and coordinates the subsystem values behind it.
- **Example**:

```rust
pub struct MediaConverter {
    mixer: AudioMixer,
    writer: FileWriter,
    encoder: VideoEncoder,
}

impl MediaConverter {
    pub fn convert(&self, source: &Path, target: &Path) -> Result<(), ConversionFailed> {
        let audio = self.mixer.extract(source)?;
        let video = self.encoder.encode(source)?;

        self.writer.write(&audio, &video, target)
    }
}
```

- **Trade-off**: A facade can grow into a catch-all if it accretes unrelated calls. When the subsystem is one value,
  the facade is just a passthrough.
- **In this service**: Already provided. The crates already wrap their subsystems (the axum `Router` over routing and
  serving, the sqlx `PgPool` over connections), do not hand-build one over them.

### Flyweight

- **Intent**: Share immutable instances to support large numbers of fine-grained values efficiently.
- **Entry signal**: A huge number of values share the same intrinsic state and memory pressure is real.
- **Structure**: A factory returns shared instances keyed by intrinsic state, the extrinsic state is passed in per call.
- **Example**:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glyph {
    A,
    B,
}

impl Glyph {
    #[must_use]
    pub fn render(self, x: u32, y: u32) -> String {
        format!("{self:?} at ({x},{y})")
    }
}
```

- **Trade-off**: A sharing pool adds complexity and is pointless without real memory pressure. Premature pooling trades
  clarity for nothing.
- **In this service**: Satisfied. `Copy` enum variants (`VehicleType`, `Month`) are shared values by definition, and the
  catalog hands out `Arc<TaxRules>` so cached rules are shared without copying. No manual instance pool.

### Proxy

- **Intent**: Provide a stand-in that controls access to another value behind the same trait.
- **Entry signal**: You need access guarding, a remote stand-in, or otherwise controlled access, with the same surface
  as the target.
- **Structure**: A proxy implements the subject trait and holds the real subject, mediating each call.
- **Example**:

```rust
pub trait Image {
    fn render(&self) -> Result<String, AccessDenied>;
}

pub struct ProtectedImage<I: Image> {
    origin: I,
    policy: AccessPolicy,
}

impl<I: Image> Image for ProtectedImage<I> {
    fn render(&self) -> Result<String, AccessDenied> {
        self.policy.ensure_allowed()?;
        self.origin.render()
    }
}
```

- **Trade-off**: Same shape as Decorator, so the intent is easy to confuse. When you only need deferred construction,
  the crate or the standard library already covers it, so no hand-written proxy.
- **In this service**: Lives in `driven/` over a port, same rules as Decorator. Lazy initialization is handled by the
  crate (`PgPoolOptions::connect_lazy_with` in `src/main.rs`) or by `std::sync::LazyLock`, do not hand-write a lazy
  proxy.

## Behavioral

### Chain of Responsibility

- **Intent**: Pass a request along a chain of handlers until one of them handles it.
- **Entry signal**: A pipeline where each stage may handle the request or pass it to the next.
- **Structure**: Each handler holds the next, it either acts or delegates onward.
- **Example**:

```rust
pub trait Stage {
    fn handle(&self, request: Request) -> Response;
}

pub struct TokenCheck<S: Stage> {
    next: S,
}

impl<S: Stage> Stage for TokenCheck<S> {
    fn handle(&self, request: Request) -> Response {
        if !request.has_token() {
            return Response::unauthorized();
        }

        self.next.handle(request)
    }
}
```

- **Trade-off**: A request can fall off the end unhandled, and the order is implicit. For a fixed two-step flow, a
  direct call is clearer.
- **In this service**: Canonical, as tower middleware layered on the axum router only (this service mounts none
  today). Do not build a general in-domain handler chain.

### Command

- **Intent**: Turn a request into a standalone value carrying everything needed to perform it.
- **Entry signal**: You need to queue, log, undo, or decouple the invoker from the receiver of a request.
- **Structure**: A command value holds its parameters, a handler executes it.
- **Example** (this repository's use case, `src/application/commands/` and `src/application/handlers/`):

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalculateCongestionTax {
    pub city: String,
    pub vehicle: String,
    pub passages: Vec<String>,
}

impl<C: TaxRulesCatalog> CalculatingCongestionTax for CalculatingCongestionTaxHandler<C> {
    async fn calculate(&self, command: CalculateCongestionTax) -> Result<CongestionTax, CalculationFailure> {
        // Performs the request the command describes.
    }
}
```

- **Trade-off**: Two types per action. For a direct synchronous call with no queueing or logging need, a plain method is
  enough.
- **In this service**: Canonical, a struct under `src/application/commands/` bound to one handler, wired in
  `src/main.rs`. There is no in-process command bus or event bus.

### Interpreter

- **Intent**: Define a grammar for a small language and evaluate its sentences.
- **Entry signal**: A recurring expression or grammar (a filter, a rule string) must be parsed and evaluated.
- **Structure**: Each grammar rule is a type with an `evaluate` method, composed into an expression tree.
- **Example**:

```rust
pub trait Expression {
    fn evaluate(&self, context: &HashMap<String, i64>) -> bool;
}

pub struct GreaterThan {
    key: String,
    value: i64,
}

impl Expression for GreaterThan {
    fn evaluate(&self, context: &HashMap<String, i64>) -> bool {
        context.get(&self.key).copied().unwrap_or(0) > self.value
    }
}
```

- **Trade-off**: A type per rule grows fast, and a real language outgrows it. For anything nontrivial, a parser crate
  beats a hand-rolled interpreter.
- **In this service**: No special stance, apply the generic guidance.

### Iterator

- **Intent**: Traverse the elements of a collection without exposing its underlying representation.
- **Entry signal**: You need custom traversal over a collection while hiding how it is stored.
- **Structure**: An iterator exposes sequential access, the aggregate produces one on demand.
- **Example** (this repository's `Passages`, `src/application/domain/models/tax_rules/passages.rs`):

```rust
impl Passages {
    pub(super) fn by_day(&self) -> impl Iterator<Item = (Date, &[Passage])> {
        self.0
            .chunk_by(|current, next| current.date() == next.date())
            .filter_map(|day_passages| day_passages.first().map(|first| (first.date(), day_passages)))
    }
}

impl FromIterator<Passage> for Passages {
    fn from_iter<I: IntoIterator<Item = Passage>>(passages: I) -> Self {
        Self::new(passages)
    }
}
```

- **Trade-off**: Hand-rolling iteration is error-prone. When the language already iterates, a custom iterator type is
  wasted code.
- **In this service**: Satisfied by the language. Return `impl Iterator` from the owner and implement `IntoIterator` or
  `FromIterator` when a collection needs them, never a hand-rolled `next` over an index.

### Mediator

- **Intent**: Centralize how a set of values interact so they do not refer to each other directly.
- **Entry signal**: Many-to-many coupling between values you want to collapse into one coordinator.
- **Structure**: Colleagues talk to a mediator, the mediator routes the interaction between them.
- **Example**:

```rust
pub trait Mediator {
    fn notify(&mut self, event: FormEvent);
}

pub struct FormCoordinator {
    submit: Button,
}

impl Mediator for FormCoordinator {
    fn notify(&mut self, event: FormEvent) {
        match event {
            FormEvent::NameFilled => self.submit.enable(),
            FormEvent::NameCleared => self.submit.disable(),
        }
    }
}
```

- **Trade-off**: The mediator can become a god object that knows too much. With only two colleagues, a direct reference
  is simpler.
- **In this service**: No special stance, apply the generic guidance.

### Memento

- **Intent**: Capture a value's internal state so it can be restored later, without breaking encapsulation.
- **Entry signal**: An in-memory undo or snapshot of a value's state is a real requirement.
- **Structure**: The originator produces a memento of its state and later restores from one, the memento stays opaque to
  others.
- **Example**:

```rust
pub struct EditorState {
    content: String,
}

pub struct Editor {
    content: String,
}

impl Editor {
    #[must_use]
    pub fn snapshot(&self) -> EditorState {
        EditorState {
            content: self.content.clone(),
        }
    }

    pub fn restore(&mut self, state: EditorState) {
        self.content = state.content;
    }
}
```

- **Trade-off**: Snapshots can be large and easy to leak state through. When the prior state can be re-read from its
  store, a memento is redundant.
- **In this service**: Rare, admitted only for an aggregate that needs in-memory undo within one request, owned by the
  aggregate (`snapshot` and `restore`).

### Observer

- **Intent**: Notify all dependents automatically when a subject's state changes.
- **Entry signal**: One state change must fan out to many independent reactions.
- **Structure**: A subject keeps a list of observers and notifies each on change, observers react without the subject
  knowing them concretely.
- **Example**:

```rust
pub trait Observer {
    fn notify(&self, event: &OrderPlaced);
}

pub struct Subject {
    observers: Vec<Box<dyn Observer>>,
}

impl Subject {
    pub fn subscribe(&mut self, observer: Box<dyn Observer>) {
        self.observers.push(observer);
    }

    pub fn publish(&self, event: &OrderPlaced) {
        for observer in &self.observers {
            observer.notify(event);
        }
    }
}
```

- **Trade-off**: Notification order and cascades can be hard to follow, and a slow observer blocks the subject. For a
  single known reaction, a direct call is clearer.
- **In this service**: No home. This service records no domain events, and there is no manual listener wiring and no
  in-process observer registry.

### State

- **Intent**: Let a value alter its behavior when its internal state changes, as if it changed type.
- **Entry signal**: A value behaves differently across several states over several operations.
- **Structure**: A state type declares the operations, the context delegates to its current state, transitions move
  between states.
- **Example**:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorState {
    Open,
    Closed,
    Locked,
}

impl DoorState {
    #[must_use]
    pub const fn toggle(self) -> Self {
        match self {
            Self::Open => Self::Closed,
            Self::Closed => Self::Open,
            Self::Locked => Self::Locked,
        }
    }
}
```

- **Trade-off**: Per-state types multiply fast. Below a few states across a few operations, a status enum with
  predicate methods is the simpler form.
- **In this service**: A status enum with predicate and transition methods first. Per-state types only when three or
  more states diverge across three or more operations, under `src/application/domain/models/{aggregate}/`, named by
  the business state.

### Strategy

- **Intent**: Define a family of interchangeable algorithms behind one trait and choose one at runtime.
- **Entry signal**: A conditional chooses between full algorithm variants from a closed set.
- **Structure**: A strategy trait declares the operation, one type per algorithm, the context holds the chosen one.
- **Example**:

```rust
pub trait Compressor {
    fn compress(&self, data: &[u8]) -> Vec<u8>;
}

pub struct GzipCompressor;

impl Compressor for GzipCompressor {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        gzip::encode(data)
    }
}

pub struct NoopCompressor;

impl Compressor for NoopCompressor {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        data.to_vec()
    }
}
```

- **Trade-off**: A type per variant for a one-line difference is overkill. A small `match` inside its owner reads fine
  until the arms grow into full algorithms.
- **In this service**: Lives in the domain as a domain-service trait with one implementation per variant under
  `src/application/domain/services/`. The Specification variant (a business-named predicate) also lives there, generic
  combinators only on the third real composition.

### Template Method

- **Intent**: Fix the skeleton of an algorithm in one method and defer a few steps to implementers.
- **Entry signal**: A stable sequence repeats with only a few steps varying.
- **Structure**: Rust has no base class, so the closest form is a trait whose provided method calls required and
  defaulted steps. Unlike a `final` base method, an implementer can still override the provided one.
- **Example**:

```rust
pub trait ReportGenerator {
    fn body(&self) -> String;

    fn header(&self) -> String {
        String::new()
    }

    fn footer(&self) -> String {
        String::new()
    }

    fn generate(&self) -> String {
        format!("{}{}{}", self.header(), self.body(), self.footer())
    }
}
```

- **Trade-off**: The steps couple to the skeleton and resist change. When steps vary independently, composition
  (Strategy) is more flexible than a trait of hooks.
- **In this service**: No special stance, apply the generic guidance (prefer composition over the hook form).

### Visitor

- **Intent**: Add a new operation over a stable structure without changing its types.
- **Entry signal**: A new operation must traverse a stable type hierarchy and does not belong on each type.
- **Structure**: Each element accepts a visitor and dispatches to the method for its type, the visitor carries the
  operation.
- **Example**:

```rust
pub trait ShapeVisitor {
    fn visit_circle(&self, circle: &Circle) -> f64;

    fn visit_square(&self, square: &Square) -> f64;
}

pub struct AreaVisitor;

impl ShapeVisitor for AreaVisitor {
    fn visit_circle(&self, circle: &Circle) -> f64 {
        std::f64::consts::PI * circle.radius().powi(2)
    }

    fn visit_square(&self, square: &Square) -> f64 {
        square.side().powi(2)
    }
}
```

- **Trade-off**: Adding a new element type forces a change to every visitor. For a closed set, an exhaustive `match` on
  an enum inside the owner is simpler than the double dispatch.
- **In this service**: Lives only at a boundary (a `query/` read-side projection or a reporting adapter), never inside
  the domain. For a closed set, prefer a `match` inside the owner.

## Sources

Gamma, Helm, Johnson, Vlissides, "Design Patterns: Elements of Reusable Object-Oriented Software" (1994). Refactoring
Guru, "Design Patterns" (https://refactoring.guru/design-patterns).
