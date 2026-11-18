use jiff::civil::Date;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("The tax period starting on <{later_starts_on}> overlaps the one starting on <{earlier_starts_on}>.")]
pub struct TaxPeriodsOverlap {
    later_starts_on: Date,
    earlier_starts_on: Date,
}

impl TaxPeriodsOverlap {
    pub(crate) const fn new(later_starts_on: Date, earlier_starts_on: Date) -> Self {
        Self {
            later_starts_on,
            earlier_starts_on,
        }
    }
}
