use jiff::civil::Time;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("The tariff band starting at <{later_starts_at}> overlaps the one starting at <{earlier_starts_at}>.")]
pub struct TariffBandsOverlap {
    later_starts_at: Time,
    earlier_starts_at: Time,
}

impl TariffBandsOverlap {
    pub(crate) const fn new(later_starts_at: Time, earlier_starts_at: Time) -> Self {
        Self {
            later_starts_at,
            earlier_starts_at,
        }
    }
}
