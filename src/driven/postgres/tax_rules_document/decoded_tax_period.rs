use std::error::Error;

use serde::Deserialize;

use super::stored_tax_period::StoredTaxPeriod;
use crate::application::domain::models::tax_rules::{
    Amount, SingleChargeWindow, TariffSchedule, TaxPeriod, TollFreeCalendar, ValidityPeriod,
};

#[derive(Deserialize)]
#[serde(try_from = "StoredTaxPeriod")]
pub(super) struct DecodedTaxPeriod(pub(super) TaxPeriod);

impl TryFrom<StoredTaxPeriod> for DecodedTaxPeriod {
    type Error = Box<dyn Error + Send + Sync>;

    fn try_from(stored: StoredTaxPeriod) -> Result<Self, Self::Error> {
        let validity = ValidityPeriod::new(stored.starts_on.0, stored.ends_on.0)?;
        let tariff = TariffSchedule::new(stored.tariff_bands.into_iter().map(|band| band.0))?;
        let calendar = TollFreeCalendar::new(
            stored.toll_free_day_before_holiday,
            stored.toll_free_months.into_iter().map(|month| month.0),
            stored.public_holidays.into_iter().map(|holiday| holiday.0),
            stored.toll_free_weekdays.into_iter().map(|weekday| weekday.0),
        );

        Ok(Self(TaxPeriod::new(
            tariff,
            calendar,
            validity,
            Amount::new(stored.daily_maximum_amount),
            SingleChargeWindow::of_minutes(stored.single_charge_window_minutes),
            stored.exempt_vehicles.into_iter().map(|vehicle| vehicle.0),
        )))
    }
}
