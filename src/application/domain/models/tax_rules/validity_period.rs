use jiff::civil::Date;

use crate::application::domain::errors::ValidityPeriodEndsBeforeStart;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidityPeriod {
    starts_on: Date,
    ends_on: Date,
}

impl ValidityPeriod {
    pub fn new(starts_on: Date, ends_on: Date) -> Result<Self, ValidityPeriodEndsBeforeStart> {
        if ends_on < starts_on {
            return Err(ValidityPeriodEndsBeforeStart::new(starts_on, ends_on));
        }

        Ok(Self { starts_on, ends_on })
    }

    pub(super) const fn starts_on(self) -> Date {
        self.starts_on
    }

    pub(super) const fn ends_on(self) -> Date {
        self.ends_on
    }

    pub(super) fn overlaps_next(self, next: Self) -> bool {
        next.starts_on <= self.ends_on
    }
}
