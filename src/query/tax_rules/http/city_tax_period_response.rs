use serde::Serialize;

use super::city_tariff_band_response::CityTariffBandResponse;
use crate::query::tax_rules::CityTaxPeriod;

#[derive(Serialize)]
pub(super) struct CityTaxPeriodResponse {
    starts_on: String,
    ends_on: String,
    tariff_bands: Vec<CityTariffBandResponse>,
    exempt_vehicles: Vec<String>,
    public_holidays: Vec<String>,
    toll_free_months: Vec<u8>,
    toll_free_weekdays: Vec<u8>,
    daily_maximum_amount: u32,
    single_charge_window_minutes: u16,
    toll_free_day_before_holiday: bool,
}

impl From<CityTaxPeriod> for CityTaxPeriodResponse {
    fn from(period: CityTaxPeriod) -> Self {
        Self {
            starts_on: period.starts_on,
            ends_on: period.ends_on,
            tariff_bands: period
                .tariff_bands
                .into_iter()
                .map(CityTariffBandResponse::from)
                .collect(),
            exempt_vehicles: period.exempt_vehicles,
            public_holidays: period.public_holidays,
            toll_free_months: period.toll_free_months,
            toll_free_weekdays: period.toll_free_weekdays,
            daily_maximum_amount: period.daily_maximum_amount,
            single_charge_window_minutes: period.single_charge_window_minutes,
            toll_free_day_before_holiday: period.toll_free_day_before_holiday,
        }
    }
}
