use super::CityTariffBand;

#[derive(Debug)]
pub struct CityTaxPeriod {
    pub starts_on: String,
    pub ends_on: String,
    pub tariff_bands: Vec<CityTariffBand>,
    pub exempt_vehicles: Vec<String>,
    pub public_holidays: Vec<String>,
    pub toll_free_months: Vec<u8>,
    pub toll_free_weekdays: Vec<u8>,
    pub daily_maximum_amount: u32,
    pub single_charge_window_minutes: u16,
    pub toll_free_day_before_holiday: bool,
}
