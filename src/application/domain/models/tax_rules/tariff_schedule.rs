use jiff::civil::Time;

use super::{Amount, TariffBand};
use crate::application::domain::errors::TariffBandsOverlap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TariffSchedule(Vec<TariffBand>);

impl TariffSchedule {
    pub fn new(bands: impl IntoIterator<Item = TariffBand>) -> Result<Self, TariffBandsOverlap> {
        let mut bands: Vec<TariffBand> = bands.into_iter().collect();
        bands.sort_by_key(|band| band.starts_at());

        let overlap = bands.windows(2).find_map(|pair| match pair {
            [earlier, later] if earlier.overlaps_next(*later) => {
                Some(TariffBandsOverlap::new(later.starts_at(), earlier.starts_at()))
            }
            _ => None,
        });

        overlap.map_or(Ok(Self(bands)), Err)
    }

    pub(super) fn fee_at(&self, time: Time) -> Amount {
        let bands_started = self.0.partition_point(|band| band.starts_at() <= time);

        bands_started
            .checked_sub(1)
            .and_then(|latest| self.0.get(latest))
            .filter(|band| time < band.ends_at())
            .map_or(Amount::ZERO, |band| band.amount())
    }
}
