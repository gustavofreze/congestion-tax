use jiff::civil::Time;

use super::Amount;
use crate::application::domain::errors::TariffBandEndsBeforeStart;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TariffBand {
    amount: Amount,
    starts_at: Time,
    ends_at: Time,
}

impl TariffBand {
    pub fn new(amount: Amount, starts_at: Time, ends_at: Time) -> Result<Self, TariffBandEndsBeforeStart> {
        if ends_at <= starts_at {
            return Err(TariffBandEndsBeforeStart::new(starts_at, ends_at));
        }

        Ok(Self {
            amount,
            starts_at,
            ends_at,
        })
    }

    pub(super) const fn amount(self) -> Amount {
        self.amount
    }

    pub(super) const fn starts_at(self) -> Time {
        self.starts_at
    }

    pub(super) const fn ends_at(self) -> Time {
        self.ends_at
    }

    pub(super) fn overlaps_next(self, next: Self) -> bool {
        next.starts_at < self.ends_at
    }
}
