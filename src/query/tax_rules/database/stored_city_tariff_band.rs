use serde::Deserialize;

use crate::query::tax_rules::CityTariffBand;

#[derive(Deserialize)]
pub(super) struct StoredCityTariffBand {
    amount: u32,
    starts_at: String,
    ends_at: String,
}

impl From<StoredCityTariffBand> for CityTariffBand {
    fn from(stored: StoredCityTariffBand) -> Self {
        Self {
            amount: stored.amount,
            starts_at: stored.starts_at,
            ends_at: stored.ends_at,
        }
    }
}
