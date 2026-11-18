<!-- Vendored from payment (.claude/skills/php-creating-application/references/state-transition.md) on 2026-11-18. Adapted to this project: the in-place eventual shape became the immutable shape that consumes self, the event emission step was dropped because this project has no domain events, and the exception became a Result. -->

# State transition (absorb, guard, move)

Use this when adding a transition to an aggregate with a lifecycle: a method that absorbs re-application of its own
resulting state, guards itself through a status predicate, and answers the aggregate in the new status. The premises
are owned by `rust-hex-application-domain` § Aggregate method contract.

This project's `TaxRules` has no lifecycle. The shape below applies when an aggregate with statuses enters, so its names
are placeholders.

```rust
// State-transition shape. Each transition is a method that consumes the aggregate and:
//   1. answers the aggregate unchanged when it is already in the transition's resulting state, so
//      a redelivered command is absorbed by the model itself, never by the handler.
//   2. guards itself by asking the status enum a predicate (cannot_be_paid), never comparing variants.
//   3. answers the aggregate in the new status, built through struct update syntax.
// The method answers Result<Self, <Error>>: the error names the refused transition, and no flow
// signal beyond it crosses the boundary. A transition takes only the primitives or domain value
// objects it genuinely needs (Reason) and builds any other value object internally. The aggregate
// never reads the clock: the instant of the transition arrives as an argument (paid_at,
// refunded_at), read once by the handler from an outbound clock port.

pub fn pay(self, paid_at: Timestamp) -> Result<Self, InvalidPaymentStatusTransition> {
    if self.status == PaymentStatus::Paid {
        return Ok(self);
    }

    if self.status.cannot_be_paid() {
        return Err(InvalidPaymentStatusTransition);
    }

    Ok(Self {
        status: PaymentStatus::Paid,
        paid_at: Some(paid_at),
        ..self
    })
}

pub fn refund(self, reason: Reason, refunded_at: Timestamp) -> Result<Self, InvalidPaymentStatusTransition> {
    if self.status == PaymentStatus::Refunded {
        return Ok(self);
    }

    if self.status.cannot_be_refunded() {
        return Err(InvalidPaymentStatusTransition);
    }

    Ok(Self {
        status: PaymentStatus::Refunded,
        refund: Some(Refund::new(reason, refunded_at)),
        ..self
    })
}
```
