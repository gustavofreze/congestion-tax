<!-- Vendored from payment (.claude/skills/php-applying-hexagonal-architecture/references/ports-and-adapters.md) on 2026-11-18. Adapted to this project: every example is Rust from this repository (TaxRulesCatalog, PostgresTaxRulesCatalog, TaxRulesCatalogSpy, CalculatingCongestionTax and its handler, the axum endpoint, src/main.rs, the tax_rules_document decoders), the DI container became the binary's composition root, and the outbound HTTP gateway example was replaced by the PostgreSQL catalog because this service calls no outbound HTTP API. -->

# Ports and adapters, in depth

Deep detail behind the `rust-applying-hexagonal-architecture` skill. Generic and language-agnostic, with short Rust
examples taken from this repository's own code. The enforced specifics for this repository live in the rules named in
the skill's § In this service overlay, this file teaches the universal concept only.

## Contents

- The dependency rule
- The runnable-core test
- Ports: inbound versus outbound
- Adapters: driver versus driven
- The composition root
- The anti-corruption boundary
- Putting it together

## The dependency rule

Source-code dependencies point inward only. Draw the system as concentric rings: the domain at the center, the
surrounding application, the adapters on the outside, the frameworks and drivers furthest out. A name resolved in an
inner ring may never refer to a name in an outer ring. The domain imports nothing from the application. The application
imports nothing from the adapters or the framework.

Control flow and source dependency point in opposite directions across an outbound boundary, and that opposition is the
whole trick. A request flows inward (an endpoint calls a handler), then the handler needs to reach back out to a
database, yet the source dependency of that reach still points inward, because the handler depends on a port trait that
the core owns, and the database adapter depends on that same trait from the outside.

```rust
// Inner ring (core) owns the contract: src/application/ports/outbound/tax_rules_catalog.rs
pub trait TaxRulesCatalog: Send + Sync {
    fn find(&self, city: &City) -> impl Future<Output = Result<Option<Arc<TaxRules>>, TaxRulesUnavailable>> + Send;
}

// Outer ring (adapter) depends on the inner contract, not the reverse: src/driven/postgres/
pub struct PostgresTaxRulesCatalog {
    pool: PgPool,
}

impl TaxRulesCatalog for PostgresTaxRulesCatalog {
    async fn find(&self, city: &City) -> Result<Option<Arc<TaxRules>>, TaxRulesUnavailable> {
        let decoded = sqlx::query_scalar::<_, Json<DecodedTaxRules>>(FIND_TAX_RULES)
            .bind(city.name())
            .fetch_optional(&self.pool)
            .await
            .map_err(TaxRulesUnavailable::new)?;

        Ok(decoded.map(|Json(DecodedTaxRules(rules))| Arc::new(rules)))
    }
}
```

## The runnable-core test

Cockburn's framing: you should be able to run the application with no UI and no database. Make the test operational by
trying to exercise a use case with every adapter replaced by an in-memory double.

```rust
// A hand-written double, no database, satisfies the same outbound port: tests/support/spies/
pub(crate) struct TaxRulesCatalogSpy {
    outcome: TaxRulesCatalogOutcome,
    searched_cities: Mutex<Vec<City>>,
}

impl TaxRulesCatalog for &TaxRulesCatalogSpy {
    fn find(&self, city: &City) -> impl Future<Output = Result<Option<Arc<TaxRules>>, TaxRulesUnavailable>> + Send {
        self.searched_cities
            .lock()
            .expect("searched cities are never poisoned")
            .push(city.clone());

        ready(match &self.outcome {
            TaxRulesCatalogOutcome::Found(rules) => Ok(Some(Arc::clone(rules))),
            TaxRulesCatalogOutcome::Missing => Ok(None),
            TaxRulesCatalogOutcome::Unavailable => Err(TaxRulesUnavailable::new("connection refused")),
        })
    }
}

// The same handler runs unchanged against the double.
let catalog = TaxRulesCatalogSpy::holding(TaxRulesBuilder::gothenburg().build());
let tax = CalculatingCongestionTaxHandler::new(&catalog).calculate(command).await;
```

If the use case runs here, the core is free of infrastructure. If it cannot run without booting a web server or
connecting to a real database, an outward dependency leaked inward. Find it and invert it behind a port.

## Ports: inbound versus outbound

A port is a trait the core owns. The core depends on the trait, never on whatever implements it. Ports come in two
kinds, distinguished by who drives whom.

| Kind     | Also called       | Direction of control        | Implemented by       | Example                                |
|----------|-------------------|-----------------------------|----------------------|----------------------------------------|
| Inbound  | Driving, primary  | The outside drives the core | The core (a handler) | `CalculatingCongestionTax`             |
| Outbound | Driven, secondary | The core drives the outside | An adapter           | `TaxRulesCatalog`, `MonotonicClock`    |

An inbound port states one use case as one contract. The driver builds a command from transport input and calls the
port. The handler implements the port.

```rust
pub trait CalculatingCongestionTax: Send + Sync {
    fn calculate(
        &self,
        command: CalculateCongestionTax,
    ) -> impl Future<Output = Result<CongestionTax, CalculationFailure>> + Send;
}
```

The handler implements the inbound port and depends only on outbound ports and the domain. It builds value objects from
the command, tells the aggregate to act, and calls the outbound ports it needs. The outbound port is a generic bound, so
the handler never names a concrete adapter.

```rust
pub struct CalculatingCongestionTaxHandler<C> {
    catalog: C,
}

impl<C: TaxRulesCatalog> CalculatingCongestionTax for CalculatingCongestionTaxHandler<C> {
    async fn calculate(&self, command: CalculateCongestionTax) -> Result<CongestionTax, CalculationFailure> {
        let city = City::new(&command.city)?;
        let vehicle = command.vehicle.parse::<VehicleType>()?;
        let passages = command
            .passages
            .iter()
            .map(|passage| passage.parse::<Passage>())
            .collect::<Result<Passages, _>>()?;

        let Some(rules) = self.catalog.find(&city).await? else {
            return Err(TaxRulesNotFound.into());
        };

        let assessment = rules.tax_for(vehicle, &passages)?;
        // ... maps the assessment to the CongestionTax read model
    }
}
```

An outbound port states a capability the core needs, expressed entirely in domain terms. No SQL, no pool, no HTTP
client, no framework type appears in the signature. The adapter performs the serialization to the transport form:
`TaxRulesCatalog::find` takes a `City` and answers `TaxRules`, and only `PostgresTaxRulesCatalog` knows that a city
becomes a bound `$1` parameter.

A port that leaks a crate type is a fake inversion. The contract below couples every caller to the database driver, so
swapping the technology forces a change in the core.

```rust
// WRONG. The pool (a crate type) and a raw row leak through the contract.
pub trait TaxRulesCatalog {
    fn find(&self, pool: &PgPool, city: &str) -> impl Future<Output = Result<Option<PgRow>, sqlx::Error>>;
}
```

## Adapters: driver versus driven

An adapter is a concrete type that speaks one specific technology. It sits outside the core, on one side of a port.

A driver adapter (primary) is on the inbound side. It receives input from a real transport, builds the command, and
calls the inbound port. It owns nothing about the use case itself. The endpoint is generic over the inbound port, never
over the handler type.

```rust
pub(crate) async fn calculate_congestion_tax<P: CalculatingCongestionTax>(
    State(calculating_congestion_tax): State<Arc<P>>,
    payload: Result<Json<CalculateCongestionTaxRequest>, JsonRejection>,
) -> Response {
    let request = match payload {
        Ok(Json(request)) => request,
        Err(rejection) => return error_response::invalid_request(&rejection),
    };

    match calculating_congestion_tax.calculate(request.into()).await {
        Ok(tax) => (StatusCode::OK, Json(CongestionTaxResponse::from(tax))).into_response(),
        Err(failure) => error_response::from_failure(&failure),
    }
}
```

A driven adapter (secondary) is on the outbound side. It implements an outbound port and owns the transport (SQL, a
cache, an HTTP call). It translates between the domain and the technology in both directions.
`PostgresTaxRulesCatalog` (above) runs one SQL query and decodes the row into `TaxRules`, and `CachedTaxRulesCatalog`
wraps any other `TaxRulesCatalog` and keeps its answers in memory for a time to live.

The two sides follow opposite naming conventions on purpose. A driver is named for the action it exposes
(`calculate_congestion_tax`). A driven adapter is named for the technology and the resource it implements
(`PostgresTaxRulesCatalog`). The enforced naming for this repository is owned by the driven and driver rules in the
skill overlay.

## The composition root

The composition root is the single place in the system that knows the concrete adapters. It builds the object graph and
injects each adapter into the core through its port. Nothing else constructs an adapter, and the core never builds one.
In this repository it is the binary, `src/main.rs`, and `src/routes.rs` mounts the result on one router.

```rust
// One place wires the graph. Swap an adapter here without touching the core.
let readiness_probe = Arc::new(PostgresDatabaseProbe::new(pool.clone()));
let catalog = CachedTaxRulesCatalog::new(
    SystemMonotonicClock,
    PostgresTaxRulesCatalog::new(pool),
    app.tax_rules_cache_ttl,
);
let calculating_congestion_tax = Arc::new(CalculatingCongestionTaxHandler::new(catalog));

axum::serve(listener, router(readiness_probe, calculating_congestion_tax))
```

Constructor injection is the delivery mechanism. Passing every dependency to `new` keeps it explicit and every
collaborator a port. Because the wiring lives in one place, a test composes the same handler with hand-written doubles,
and a different deployment composes it with real ones, with no change to the core.

## The anti-corruption boundary

Foreign concepts (another service's model, a third-party API, a stored document shape, a legacy field name) are
translated into this system's vocabulary at the adapter boundary. The translation lives in the driven adapter, never in
the handler and never in the domain. The core stays expressed in its own ubiquitous language, protected from the shape
of whatever it integrates with.

```rust
// The stored document speaks columns and JSON keys. The domain speaks TaxRules.
// src/driven/postgres/tax_rules_document/decoded_tax_rules.rs
#[derive(Deserialize)]
#[serde(try_from = "StoredTaxRules")]
pub(in crate::driven::postgres) struct DecodedTaxRules(pub(in crate::driven::postgres) TaxRules);

impl TryFrom<StoredTaxRules> for DecodedTaxRules {
    type Error = TaxPeriodsOverlap;

    fn try_from(stored: StoredTaxRules) -> Result<Self, Self::Error> {
        TaxRules::new(stored.currency.0, stored.periods.into_iter().map(|period| period.0)).map(Self)
    }
}
```

When the translation grows beyond a single mapping (a configurable catalog, selection rules over it), that logic belongs
inside the adapter behind the port, not in the domain. A rule that selects over an external catalog is an adapter
concern, because the domain must not learn the catalog's shape.

## Putting it together

The flow of one use case, end to end:

1. A driver adapter receives transport input and builds a command.
2. The driver calls an inbound port. The handler behind it runs.
3. The handler builds value objects, tells the aggregate to act, and enforces invariants in the domain.
4. The handler calls outbound ports for anything it needs from the outside.
5. Driven adapters fulfill those ports, translating at the boundary.
6. The composition root decided, once, which concrete adapters back those ports.

At no step does an inner ring name an outer one. That is the property the architecture buys, and the runnable-core test
is how you confirm you still have it.
