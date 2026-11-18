use serde::Deserialize;

use crate::application::commands::CalculateCongestionTax;

#[derive(Deserialize)]
pub(crate) struct CalculateCongestionTaxRequest {
    city: String,
    vehicle: String,
    passages: Vec<String>,
}

impl From<CalculateCongestionTaxRequest> for CalculateCongestionTax {
    fn from(request: CalculateCongestionTaxRequest) -> Self {
        Self {
            city: request.city,
            vehicle: request.vehicle,
            passages: request.passages,
        }
    }
}
