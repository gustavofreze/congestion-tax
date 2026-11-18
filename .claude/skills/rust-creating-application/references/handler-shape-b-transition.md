<!-- Vendored from payment (.claude/skills/php-creating-application/references/handler-shape-b-transition.md) on 2026-11-18. Adapted to this project: the PHP handler became a Rust trait implementation, the throwing null guard became a let-else not-found, and the load-then-apply worked example is CalculatingCongestionTaxHandler beside the placeholder transition. -->

# Handler shape B (single aggregate load then apply)

Use this when wiring a handler that loads one aggregate through an outbound port, answers not-found when the port finds
nothing, and applies the aggregate's behavior: a transition that is then persisted, or a computation whose result the
handler answers.

A tenant scope, where a project has one, is carried into the port beside the id, and passing it is the isolation
invariant: a load that ignores the scope can return another tenant's aggregate. In a single-tenant service, or where
the aggregate's table carries no tenant discriminator, the command has no scope field, the port takes the key alone,
and the gate is skipped rather than failed. This project is single-tenant. Everything else in the shape (promote, load,
not-found guard, apply, answer) is unchanged either way.

## Load then compute (this project)

```rust
// src/application/handlers/calculating_congestion_tax_handler.rs
//
// Shape B, computing variant. Promote every command primitive to its value object first, so a
// rejected input fails before the port is reached. Load the aggregate through the outbound port,
// answer the not-found when it finds nothing (the one guard Shape B allows, written as let-else),
// apply the aggregate's behavior, and map the domain result to the read model the inbound port
// promises. Every failure propagates with ? into the inbound port's failure enum.

pub struct CalculatingCongestionTaxHandler<C> {
    catalog: C,
}

impl<C: TaxRulesCatalog> CalculatingCongestionTaxHandler<C> {
    #[must_use]
    pub const fn new(catalog: C) -> Self {
        Self { catalog }
    }
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
        let total = assessment.total();

        Ok(CongestionTax {
            days: assessment
                .days()
                .iter()
                .map(|day| DailyCongestionTax {
                    date: day.date().to_string(),
                    amount: day.amount().value(),
                })
                .collect(),
            amount: total.amount().value(),
            currency: total.currency().code().to_owned(),
        })
    }
}
```

## Load then transition

```rust
// Shape B, transition variant. Load, answer the not-found, call the transition, persist. The
// transition receives a domain value object built through its intent-named for_<origin>_<action>
// constructor, sourced from the command primitive fields (promoting a primitive to its enum at the
// boundary when the constructor takes one), never from a single catch-all field. It also receives
// the instant it happened, read from the outbound clock port the handler takes through new(),
// because the aggregate never reads the clock.

async fn <operation>(&self, command: <Operation><Aggregate>) -> Result<(), <Failure>> {
    let <aggregate>_id = <Aggregate>Id::new(&command.<aggregate>_id)?;

    let Some(<aggregate>) = self.<aggregates>.find(&<aggregate>_id).await? else {
        return Err(<Aggregate>NotFound.into());
    };

    let code = command.code.as_deref().map(str::parse::<Code>).transpose()?;
    let reason = Reason::for_<origin>_<action>(code, command.additional_information);

    let <aggregate> = <aggregate>.<operation>(reason, self.clock.now())?;

    self.<aggregates>.save(&<aggregate>).await?;

    Ok(())
}
```
