<!-- Vendored from payment (.claude/skills/php-creating-application/references/handler-shape-a-create.md) on 2026-11-18. Adapted to this project: the PHP handler became a Rust struct generic over its outbound ports implementing the inbound port trait, and the shape is kept as placeholders because this project has no create use case. -->

# Handler shape A (single aggregate create)

Use this when wiring the default thin create handler that passes command primitives straight to the aggregate
constructor.

This project has no create use case yet, so the names below are placeholders.

```rust
// Shape A. Single aggregate, create. The default thin handler. Pass command primitives straight to
// the aggregate constructor, persist via the outbound port, answer the read model the inbound port
// promises. The constructor builds its own value objects and generates the identity, so the handler
// builds none, generates no id, and pre-checks nothing. Beyond the command fields, the handler
// supplies only the creation instant, read from the outbound clock port it takes through new(),
// because the aggregate never reads the clock. Every failure propagates with ? into the inbound
// port's failure enum.

pub struct <Operation>ing<Aggregate>Handler<R, K> {
    clock: K,
    <aggregates>: R,
}

impl<R: <Aggregates>, K: Clock> <Operation>ing<Aggregate> for <Operation>ing<Aggregate>Handler<R, K> {
    // Correct. Primitives go straight to the constructor, which builds its own value objects and
    // generates the id. No pre-check, because uniqueness is a schema constraint the driven adapter
    // translates.
    async fn <operation>(&self, command: <Operation><Aggregate>) -> Result<<ReadModel>, <Failure>> {
        let <aggregate> = <Aggregate>::create(
            &command.<field>,
            command.<amount>,
            &command.<currency>,
            self.clock.now(),
        )?;

        self.<aggregates>.save(&<aggregate>).await?;

        Ok(<ReadModel>::from(&<aggregate>))
    }
}

// PROHIBITED. A value object is built up front to feed the constructor, and the id is generated
// outside it and passed in. The constructor takes primitives and owns id generation.
//
// async fn <operation>(&self, command: <Operation><Aggregate>) -> Result<<ReadModel>, <Failure>> {
//     let money = Money::new(Amount::new(command.<amount>), Currency::new(&command.<currency>)?);
//     let id = <Aggregate>Id::generate();
//
//     let <aggregate> = <Aggregate>::create(id, money, &command.<field>, self.clock.now())?;
//
//     self.<aggregates>.save(&<aggregate>).await?;
//
//     Ok(<ReadModel>::from(&<aggregate>))
// }
```
