use serde::Deserialize;

use super::decoded_date::DecodedDate;
use super::decoded_month::DecodedMonth;
use super::decoded_tariff_band::DecodedTariffBand;
use super::decoded_vehicle_type::DecodedVehicleType;
use super::decoded_weekday::DecodedWeekday;

#[derive(Deserialize)]
pub(super) struct StoredTaxPeriod {
    pub(super) starts_on: DecodedDate,
    pub(super) ends_on: DecodedDate,
    pub(super) tariff_bands: Vec<DecodedTariffBand>,
    pub(super) exempt_vehicles: Vec<DecodedVehicleType>,
    pub(super) public_holidays: Vec<DecodedDate>,
    pub(super) toll_free_months: Vec<DecodedMonth>,
    pub(super) toll_free_weekdays: Vec<DecodedWeekday>,
    pub(super) daily_maximum_amount: u32,
    pub(super) single_charge_window_minutes: u16,
    pub(super) toll_free_day_before_holiday: bool,
}
