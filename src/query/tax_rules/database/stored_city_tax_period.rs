use serde::Deserialize;

use super::stored_city_tariff_band::StoredCityTariffBand;
use crate::query::tax_rules::{CityTariffBand, CityTaxPeriod};

#[derive(Deserialize)]
pub(super) struct StoredCityTaxPeriod {
    starts_on: String,
    ends_on: String,
    tariff_bands: Vec<StoredCityTariffBand>,
    exempt_vehicles: Vec<String>,
    public_holidays: Vec<String>,
    toll_free_months: Vec<u8>,
    toll_free_weekdays: Vec<u8>,
    daily_maximum_amount: u32,
    single_charge_window_minutes: u16,
    toll_free_day_before_holiday: bool,
}

impl From<StoredCityTaxPeriod> for CityTaxPeriod {
    fn from(stored: StoredCityTaxPeriod) -> Self {
        Self {
            starts_on: stored.starts_on,
            ends_on: stored.ends_on,
            tariff_bands: stored.tariff_bands.into_iter().map(CityTariffBand::from).collect(),
            exempt_vehicles: stored.exempt_vehicles,
            public_holidays: stored.public_holidays,
            toll_free_months: stored.toll_free_months,
            toll_free_weekdays: stored.toll_free_weekdays,
            daily_maximum_amount: stored.daily_maximum_amount,
            single_charge_window_minutes: stored.single_charge_window_minutes,
            toll_free_day_before_holiday: stored.toll_free_day_before_holiday,
        }
    }
}
