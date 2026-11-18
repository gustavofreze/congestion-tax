use serde::Deserialize;

use super::stored_tariff_band::StoredTariffBand;
use crate::application::domain::errors::TariffBandEndsBeforeStart;
use crate::application::domain::models::tax_rules::{Amount, TariffBand};

#[derive(Deserialize)]
#[serde(try_from = "StoredTariffBand")]
pub(super) struct DecodedTariffBand(pub(super) TariffBand);

impl TryFrom<StoredTariffBand> for DecodedTariffBand {
    type Error = TariffBandEndsBeforeStart;

    fn try_from(stored: StoredTariffBand) -> Result<Self, Self::Error> {
        TariffBand::new(Amount::new(stored.amount), stored.starts_at.0, stored.ends_at.0).map(Self)
    }
}
