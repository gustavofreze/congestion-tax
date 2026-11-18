use jiff::civil::Date;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("The validity period starting on <{starts_on}> must not end before it starts, on <{ends_on}>.")]
pub struct ValidityPeriodEndsBeforeStart {
    starts_on: Date,
    ends_on: Date,
}

impl ValidityPeriodEndsBeforeStart {
    pub(crate) const fn new(starts_on: Date, ends_on: Date) -> Self {
        Self { starts_on, ends_on }
    }
}
