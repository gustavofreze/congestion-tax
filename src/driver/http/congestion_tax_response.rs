use serde::Serialize;

use super::daily_congestion_tax_response::DailyCongestionTaxResponse;
use crate::application::ports::inbound::CongestionTax;

#[derive(Serialize)]
pub(super) struct CongestionTaxResponse {
    days: Vec<DailyCongestionTaxResponse>,
    amount: u32,
    currency: String,
}

impl From<CongestionTax> for CongestionTaxResponse {
    fn from(tax: CongestionTax) -> Self {
        Self {
            days: tax.days.into_iter().map(DailyCongestionTaxResponse::from).collect(),
            amount: tax.amount,
            currency: tax.currency,
        }
    }
}
