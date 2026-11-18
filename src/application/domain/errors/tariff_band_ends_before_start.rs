use jiff::civil::Time;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("The tariff band starting at <{starts_at}> must end after it starts, not at <{ends_at}>.")]
pub struct TariffBandEndsBeforeStart {
    starts_at: Time,
    ends_at: Time,
}

impl TariffBandEndsBeforeStart {
    pub(crate) const fn new(starts_at: Time, ends_at: Time) -> Self {
        Self { starts_at, ends_at }
    }
}
