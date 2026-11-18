<!-- Vendored from payment (.claude/skills/php-creating-application/references/handler-branching-antipattern.md) on 2026-11-18. Adapted to this project: the PHP handler bodies became Rust trait method bodies branching on an Option, kept as placeholders because this project persists no aggregate. -->

# Handler create-or-mutate branching (anti-pattern)

Read this when a handler is tempted to branch on a lookup result, showing the prohibited branch next to the
constructor-decides correction.

```rust
// Create-or-mutate branching. A handler that calls find and then chooses between create-new and
// mutate-existing based on whether the result is None is doing domain logic in the wrong layer. A
// constructor accepts the optional existing instance and decides, or a domain service owns the
// decision when it is a cross-aggregate rule, with the handler keeping the input and the output. A
// single not-found guard is allowed in Shape B. Anything beyond that guard is domain logic.

// PROHIBITED. The handler branches on the lookup result.
async fn authorize(&self, command: AuthorizePayment) -> Result<(), AuthorizationFailure> {
    let payment_id = PaymentId::new(&command.payment_id)?;

    let payment = match self.payments.find(&payment_id).await? {
        None => Payment::create(
            &command.charge_method,
            &command.charge_country,
            command.charge_amount_value,
            &command.charge_amount_currency,
            self.clock.now(),
        )?,
        Some(existing) => existing.reauthorize(self.clock.now())?,
    };

    self.payments.save(&payment).await?;

    Ok(())
}

// CORRECT. A constructor accepts the optional existing instance and decides. When the decision is a
// cross-aggregate rule, a domain service owns it instead, with the handler keeping the input and the
// output.
async fn authorize(&self, command: AuthorizePayment) -> Result<(), AuthorizationFailure> {
    let payment_id = PaymentId::new(&command.payment_id)?;
    let existing = self.payments.find(&payment_id).await?;

    let payment = Payment::authorize(
        existing,
        &command.charge_method,
        &command.charge_country,
        command.charge_amount_value,
        &command.charge_amount_currency,
        self.clock.now(),
    )?;

    self.payments.save(&payment).await?;

    Ok(())
}
```
