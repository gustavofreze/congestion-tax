use serde::Serialize;

use crate::query::tax_rules::CityTariffBand;

#[derive(Serialize)]
pub(super) struct CityTariffBandResponse {
    amount: u32,
    starts_at: String,
    ends_at: String,
}

impl From<CityTariffBand> for CityTariffBandResponse {
    fn from(band: CityTariffBand) -> Self {
        Self {
            amount: band.amount,
            starts_at: band.starts_at,
            ends_at: band.ends_at,
        }
    }
}
