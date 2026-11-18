use jiff::civil::Date;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("No tax period of the city covers the passages of <{date}>.")]
pub struct PassageOutsideTaxPeriods {
    date: Date,
}

impl PassageOutsideTaxPeriods {
    pub(crate) const fn new(date: Date) -> Self {
        Self { date }
    }
}
