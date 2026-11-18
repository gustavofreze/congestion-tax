<!-- Vendored from payment (.claude/skills/php-applying-ddd/references/tactical.md) on 2026-11-18. Adapted to this project: every example is Rust, taken from src/application/domain (Currency, TaxRules, TaxPeriodsOverlap, SingleChargeWindow, Amount) and the TaxRulesCatalog port where an equivalent exists, the domain exception became a thiserror domain error, the Domain Event section and the emit step were dropped because this service emits no domain events, and the rule routes name the rust-* rules. -->

# Tactical building blocks

The types you model a domain with. This file is the deep reference behind the table in `SKILL.md`. Each block carries
the question it answers, the rules that keep it honest, and an example, taken from this repository's model where one
exists and from a neutral domain (Order, Customer, Account, Money) where it does not. The ENFORCED shape for this
repository is routed in `SKILL.md` § In this service, read the rule before you write.

## Contents

- Value Object
- Entity
- Aggregate and aggregate root
- Repository
- Domain Service
- Factory
- Domain Error
- Invariant
- Guarding versus validating

## Value Object

A value object answers "what is it, by its value". It has no identity. Two value objects with equal attributes are the
same thing and are interchangeable. Three properties define it.

1. Immutable. It never changes after construction. An operation returns a new instance.
2. Equal by value. Equality compares attributes (a derived `PartialEq, Eq`), not addresses.
3. Self-validating. The constructor rejects an invalid state, so an instance is always valid.

A value object is the cure for primitive obsession. A raw `u32` for money or a raw `String` for a currency carries no
rules and no behavior. Promote each to a type that validates and behaves. In this repository `Currency` wraps the ISO
4217 code and refuses anything else.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Currency(String);

impl Currency {
    pub fn new(code: &str) -> Result<Self, CurrencyFormatNotValid> {
        let is_iso_code = code.len() == 3 && code.bytes().all(|letter| letter.is_ascii_uppercase());

        if !is_iso_code {
            return Err(CurrencyFormatNotValid::new(code));
        }

        Ok(Self(code.to_owned()))
    }

    #[must_use]
    pub fn code(&self) -> &str {
        &self.0
    }
}
```

Behavior belongs on the value object too. Arithmetic over an amount (`Amount::plus`), the overlap of two validity
periods (`ValidityPeriod::overlaps_next`), the coverage of a charge window (`SingleChargeWindow::covers`), all live
here. A primitive in a domain method signature is the signal that a value object is missing.

## Entity

An entity answers "which one is it, over time". It has identity that persists across changes to its attributes. A
customer who changes their email is the same customer. Equality is by identity, never by attributes. This service has no
entity (its aggregate is keyed by city and never changes), so the example is neutral.

An entity has a lifecycle. It is created, it transitions through states, it may be archived. It expresses those
transitions as intention-revealing methods, never as public fields or setters. A transition takes `self` and returns
the changed value or a domain error, so no caller ever sees a half-changed entity. The difference between
`order.cancel(reason)` and `order.status = OrderStatus::Canceled` is the difference between a domain model and a data
structure.

```rust
pub struct Customer {
    id: CustomerId,
    email: Email,
    status: CustomerStatus,
}

impl Customer {
    pub fn change_email(self, email: Email) -> Result<Self, ClosedCustomerCannotChange> {
        if self.status.is_closed() {
            return Err(ClosedCustomerCannotChange);
        }

        Ok(Self { email, ..self })
    }
}
```

## Aggregate and aggregate root

An aggregate answers "what changes together". It is a cluster of entities and value objects treated as one unit for the
purpose of consistency. The boundary is drawn by invariants: values that must stay consistent on every change belong in
the same aggregate. Values that may be eventually consistent belong in separate aggregates.

The aggregate root is the single type that guards the boundary. Four rules follow.

1. Outside code holds a reference to the root only. Child entities are reached through it.
2. Every change to anything inside the boundary goes through a root method, so the root enforces every invariant in one
   place.
3. One aggregate references another by the other root's identifier, never by holding the foreign root as a field. The
   use case loads the counterpart separately.
4. Only values cross the boundary. A port, a service, or a closure never enters a root method as an argument and never
   lives as a field. When a behavior seems to need a collaborator, the handler resolves it first and passes the
   resulting value in. The double-dispatch variants that hand a service to an entity are rejected here by design, so the
   aggregate stays free of anything that could perform I/O (enforced as `rust-hex-application-domain` § Aggregate method
   contract).

Keep aggregates small. A large aggregate widens the transaction, increases contention, and tends to pull in values that
have their own lifecycle. When in doubt, make a smaller aggregate and connect by id (Vernon, "Effective Aggregate
Design").

In this repository `TaxRules` is the root. Its tax periods, tariff schedules, calendars, and charge windows are reached
only through it, and its private fields keep every caller on its methods.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxRules {
    periods: Vec<TaxPeriod>,
    currency: Currency,
}

impl TaxRules {
    pub fn tax_for(
        &self,
        vehicle: VehicleType,
        passages: &Passages,
    ) -> Result<TaxAssessment, PassageOutsideTaxPeriods> {
        let days = passages
            .by_day()
            .map(|(date, day_passages)| {
                let period = self
                    .period_on(date)
                    .ok_or_else(|| PassageOutsideTaxPeriods::new(date))?;
                Ok(DailyTax::new(date, period.daily_tax(vehicle, date, day_passages)))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(TaxAssessment::new(self.currency.clone(), days))
    }
}
```

`TaxPeriod` is a part inside the `TaxRules` boundary. It has no repository, is never loaded alone, and is reached only
through `TaxRules` (its `daily_tax` is `pub(super)`). The `City` the rules belong to is the key the catalog finds them
by, not a field of the aggregate.

## Repository

A repository answers "where do aggregates live". It presents the illusion of an in-memory collection of aggregates,
hiding persistence behind it. The domain depends on the repository as a port (a trait), and the adapter implements it
outside the domain.

One repository per aggregate root. Never one per entity. A child entity has no repository, because it is only ever
loaded and saved through its root. A repository that exposes child entities directly leaks the aggregate boundary.

The trait speaks domain types: aggregates in, aggregates or their identifiers out, never rows or primitives. The read
side is separate. A repository is for loading and saving aggregates on the write side, not for arbitrary queries that
feed a screen (that is the CQRS read model). In this repository the aggregate is content maintained outside the service,
so its port only finds, and is named after the business resource (`TaxRulesCatalog`).

```rust
pub trait TaxRulesCatalog: Send + Sync {
    fn find(&self, city: &City) -> impl Future<Output = Result<Option<Arc<TaxRules>>, TaxRulesUnavailable>> + Send;
}
```

## Domain Service

A domain service answers "whose rule is it, when no single type owns it". It hosts a domain operation that crosses two
or more aggregates, or a calculation over value objects with no natural host. It is part of the domain and carries
business rules, not orchestration.

Domain services are rare by design. Most behavior belongs on an aggregate or a value object. A single-aggregate rule is
never a domain service, it is a method on that aggregate. Reach for a domain service only when the operation honestly
fits nowhere else. This service has none, so the examples are neutral.

The shape is strict. Stateless, no infrastructure field, and every parameter and return type is a domain type. A
primitive in the signature is the signal that the logic belongs on a value object instead. It never calls a repository
or an outbound port. That coordination is the handler's job, which loads through a port and then calls the service with
the loaded aggregates.

A Strategy (a domain decision with one implementation per business variant) and a Specification (a stateless business
predicate) are domain services and follow the same shape.

```rust
pub struct FundsAvailability;

impl FundsAvailability {
    pub fn withdrawable_from(&self, policy: &OverdraftPolicy, account: &Account) -> Money {
        policy.apply_to(account.balance())
    }
}
```

The cross-aggregate variant hosts a rule spanning two aggregates the handler has already loaded. Its parameters and
return are domain types, never primitives.

```rust
pub struct MoneyTransfer;

impl MoneyTransfer {
    pub fn transfer(
        &self,
        amount: Money,
        source: Account,
        destination: Account,
    ) -> Result<TransferReceipt, InsufficientFunds> {
        let debited = source.debit(amount.clone())?;
        let credited = destination.credit(amount);

        Ok(TransferReceipt::issue(debited, credited))
    }
}
```

## Factory

A factory answers "how is a valid one born". When constructing a valid aggregate or value object takes more than a
trivial assignment (building child entities, generating identity, enforcing a creation invariant), encapsulate it so no
caller can produce an invalid instance.

The common form is an associated function on the type itself, named after the business act (`place`, `open`, `issue`),
falling back to `new` when no verb fits, and to the `FromStr` or `TryFrom` traits when the input is a raw string or a
foreign shape. An associated constant names a well-known value. A separate factory type is warranted only when creation
spans several types or needs collaborators.

```rust
impl Amount {
    pub const ZERO: Self = Self(0);
}

impl SingleChargeWindow {
    #[must_use]
    pub fn of_minutes(minutes: u16) -> Self {
        Self(SignedDuration::from_mins(i64::from(minutes)))
    }
}
```

## Domain Error

A domain error answers "which invariant was violated". Every domain failure returns a dedicated type named after the
rule it guards, in business language (`TaxPeriodsOverlap`, `CurrencyFormatNotValid`, `ValidityPeriodEndsBeforeStart`),
never a generic error with a string message and never a panic.

The type is pure. No HTTP status, no formatted user message beyond its `thiserror` display text. It may carry domain
context as typed fields (value objects, identifiers, enums, dates), and the rejected raw input when the failure is that
the input did not parse. Formatting to transport happens at the boundary, outside the domain.

A not-found is not a domain error. A lookup miss is an application failure, not a domain invariant, so it lives in the
application layer (`TaxRulesNotFound`, beside the inbound port), not here. Infrastructure failures (database, file
system, raw transport) are not domain errors either (`TaxRulesUnavailable` sits beside the outbound port).

```rust
#[derive(Debug, Error)]
#[error("The tax period starting on <{later_starts_on}> overlaps the one starting on <{earlier_starts_on}>.")]
pub struct TaxPeriodsOverlap {
    earlier_starts_on: Date,
    later_starts_on: Date,
}
```

## Invariant

An invariant answers "what must always be true". It is a business rule that holds at every observable moment, for
example the tax periods of a city never overlap, or a validity period never ends before it starts. The aggregate
enforces every invariant inside its boundary, so an aggregate loaded from storage is always valid.

Invariants live in the model, never in a handler or an endpoint. A value object enforces its own invariants in the
constructor. An aggregate enforces cross-value invariants in its constructor and its transition methods, guarding before
it changes anything: ask the state whether the transition is allowed, then return the changed value.

The creation invariant of this repository's root: `TaxRules::new` sorts the periods and refuses to exist when two of
them overlap, so no caller ever holds overlapping rules.

```rust
impl TaxRules {
    pub fn new(currency: Currency, periods: impl IntoIterator<Item = TaxPeriod>) -> Result<Self, TaxPeriodsOverlap> {
        let mut chronological: Vec<TaxPeriod> = periods.into_iter().collect();
        chronological.sort_by_key(|period| period.validity().starts_on());

        let overlap = chronological.windows(2).find_map(|pair| match pair {
            [earlier, later] if earlier.validity().overlaps_next(later.validity()) => Some(TaxPeriodsOverlap::new(
                earlier.validity().starts_on(),
                later.validity().starts_on(),
            )),
            _ => None,
        });

        overlap.map_or_else(
            || {
                Ok(Self {
                    periods: chronological,
                    currency,
                })
            },
            Err,
        )
    }
}
```

The transition form, on a neutral aggregate that changes state, guards and then returns the changed value, never
through a setter:

```rust
impl Order {
    pub fn cancel(self) -> Result<Self, InvalidOrderStatusTransition> {
        if self.status.cannot_be_canceled() {
            return Err(InvalidOrderStatusTransition);
        }

        Ok(Self {
            status: OrderStatus::Canceled,
            ..self
        })
    }
}
```

## Guarding versus validating

Guarding and validating both reject bad data, and they are not the same move. Validating filters untrusted input at the
edge of the system. Guarding is the failsafe that keeps an always-true invariant true inside the domain. Knowing which
one you are writing tells you where the code lives and how it fails.

Validation runs on input from the outside world (an HTTP body, a query string) before that input reaches the domain.
Bad external input is ordinary, not exceptional, so validation answers the caller with a readable error. It lives in the
driver layer, as the request validation of the inbound adapter (in this repository the `Json` extractor rejection that
`error_response::invalid_request` answers), never in the aggregate.

Guarding runs inside the always-valid boundary, on data the domain already assumes is good. A value object constructor
rejecting an invalid state, and an aggregate transition asking the status whether the move is allowed, are guards. A
guard firing is not an ordinary event. It means a bad value tried to cross the boundary, so a guard fails fast by
returning a domain error rather than a message.

The two layers overlap on purpose. When a request-validation rule fully covers a domain guard, that guard can never fire
through that entry point, which makes it the boundary's shielded guard (owned by the `rust-hex-driver-http` rule). The
guard still stays in the model as the failsafe for every other caller, and the validation stays at the edge as the first
line that answers the user.
