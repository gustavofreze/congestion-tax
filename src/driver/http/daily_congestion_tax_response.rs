use serde::Serialize;

use crate::application::ports::inbound::DailyCongestionTax;

#[derive(Serialize)]
pub(super) struct DailyCongestionTaxResponse {
    date: String,
    amount: u32,
}

impl From<DailyCongestionTax> for DailyCongestionTaxResponse {
    fn from(day: DailyCongestionTax) -> Self {
        Self {
            date: day.date,
            amount: day.amount,
        }
    }
}
