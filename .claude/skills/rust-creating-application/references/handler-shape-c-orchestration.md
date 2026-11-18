<!-- Vendored from payment (.claude/skills/php-creating-application/references/handler-shape-c-orchestration.md) on 2026-11-18. Adapted to this project: the try/catch fallback became a match on the port outcome, and the shape is kept as placeholders because this project has no orchestrating use case. -->

# Handler shape C (compensating orchestration)

Use this when a write use case needs multi-aggregate coordination or a compensating fallback, the one shape where
branching and a fallback on a port error are allowed. The fallback translates a port outcome into the aggregate's own
transition, so the aggregate is still triggered at most once per execution path, and the external interaction happens
only because a prior use case committed the transition that authorizes it (`rust-hex-application-handlers` § One
aggregate trigger per handler).

A tenant scope, where a project has one, is carried into the port beside the id. In a single-tenant service, or where
the aggregate's table carries no tenant discriminator, the command has no scope field, the port takes the id alone, and
the gate is skipped rather than failed. This project is single-tenant and has no orchestrating use case, so the names
below are placeholders. The compensating fallback is the point of this shape and it is unaffected either way.

```rust
// Shape C. Compensating orchestration. The named exception to the thin default: a match on a port
// outcome translating each error into the aggregate's own transition, allowed in this shape only.
// Any other port error still propagates with ?. The thin-handler, one-aggregate-trigger, and
// domain-service invariants are owned by the rust-hex-application-handlers rule, read it. Each
// transition receives the instant it happened, read from the outbound clock port, because the
// aggregate never reads the clock.

async fn accept(&self, command: AcceptCharge) -> Result<(), AcceptanceFailure> {
    let payment_id = PaymentId::new(&command.payment_id)?;

    let Some(payment) = self.payments.find(&payment_id).await? else {
        return Err(PaymentNotFound.into());
    };

    let payment = match self.charge_submitting.submit(&payment).await {
        Ok(charged) => payment.accept(charged, self.clock.now())?,
        Err(ChargeSubmissionFailure::NoApplicableRoutingRule) => {
            payment.fail(Reason::for_no_routing_rule(), self.clock.now())?
        }
        Err(ChargeSubmissionFailure::ProviderChainExhausted) => {
            payment.fail(Reason::for_provider_chain_exhaustion(), self.clock.now())?
        }
    };

    self.payments.save(&payment).await?;

    Ok(())
}
```
