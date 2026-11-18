use serde::Deserialize;

use super::decoded_time::DecodedTime;

#[derive(Deserialize)]
pub(super) struct StoredTariffBand {
    pub(super) amount: u32,
    pub(super) starts_at: DecodedTime,
    pub(super) ends_at: DecodedTime,
}
