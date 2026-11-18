<!-- Vendored from payment (.claude/skills/php-creating-application/references/domain-exception.md) on 2026-11-18. Adapted to this project: renamed from domain-exception.md, the PHP exception classes became thiserror types answered through Result, and the dedicated and input-rejection shapes are worked from src/application/domain/errors/. -->

# Domain error shapes

Use this when choosing where a failure type lives and how it is shaped, across the four cases (prohibited ad hoc
failure, dedicated domain type, application not-found, and input-rejection carrying the raw value).

Every failure is a value answered through `Result`, never a panic. Each type lives in its own file named after it, and
each derives `Debug` and `thiserror::Error` with an `#[error("...")]` message that is a full sentence.

```rust
// Domain error shapes. Four cases, each labeled below.

// PROHIBITED. An ad hoc failure: a String error, an anyhow error, or a panic, inside the domain.
if amount == 0 {
    return Err("Amount must be positive.".to_owned());
}

// CORRECT. A dedicated type named after the invariant, a unit struct when no context is carried.
// src/application/domain/errors/city_cannot_be_empty.rs
#[derive(Debug, Error)]
#[error("A city cannot be empty.")]
pub struct CityCannotBeEmpty;

// NOT A DOMAIN ERROR. A not-found lives beside the inbound port, under
// src/application/ports/inbound/, and is answered by the handler when the outbound port finds
// nothing. The placement decision tree (which directory a failure type lives in) and its shape are
// owned by the rust-hex-application-domain rule, read it.
// src/application/ports/inbound/<aggregate>_not_found.rs
#[derive(Debug, Error)]
#[error("The <aggregate> was not found.")]
pub struct <Aggregate>NotFound;

// at the call site
return Err(<Aggregate>NotFound.into());

// CORRECT. An input-rejection error carrying the raw offending value as a primitive, because the
// input failed promotion to a value object or enum and no value object exists yet to carry it.
// src/application/domain/errors/unsupported_vehicle_type.rs
#[derive(Debug, Error)]
#[error("Vehicle type <{raw}> is not supported.")]
pub struct UnsupportedVehicleType {
    raw: String,
}

impl UnsupportedVehicleType {
    pub(crate) fn new(raw: &str) -> Self {
        Self { raw: raw.to_owned() }
    }
}

// at the call site, where the raw input could not be promoted to the VehicleType enum
_ => Err(UnsupportedVehicleType::new(raw)),
```
