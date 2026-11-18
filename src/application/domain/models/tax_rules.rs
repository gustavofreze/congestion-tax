mod amount;
mod charge_window;
mod city;
mod currency;
mod daily_tax;
mod money;
mod month;
mod passage;
mod passages;
mod single_charge_window;
mod tariff_band;
mod tariff_schedule;
mod tax_assessment;
mod tax_period;
mod toll_free_calendar;
mod validity_period;
mod vehicle_type;

use jiff::civil::Date;

pub use amount::Amount;
pub use city::City;
pub use currency::Currency;
pub use daily_tax::DailyTax;
pub use money::Money;
pub use month::Month;
pub use passage::Passage;
pub use passages::Passages;
pub use single_charge_window::SingleChargeWindow;
pub use tariff_band::TariffBand;
pub use tariff_schedule::TariffSchedule;
pub use tax_assessment::TaxAssessment;
pub use tax_period::TaxPeriod;
pub use toll_free_calendar::TollFreeCalendar;
pub use validity_period::ValidityPeriod;
pub use vehicle_type::VehicleType;

use crate::application::domain::errors::{PassageOutsideTaxPeriods, TaxPeriodsOverlap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxRules {
    periods: Vec<TaxPeriod>,
    currency: Currency,
}

impl TaxRules {
    pub fn new(currency: Currency, periods: impl IntoIterator<Item = TaxPeriod>) -> Result<Self, TaxPeriodsOverlap> {
        let mut periods: Vec<TaxPeriod> = periods.into_iter().collect();
        periods.sort_by_key(|period| period.validity().starts_on());

        let overlap = periods.windows(2).find_map(|pair| match pair {
            [earlier, later] if earlier.validity().overlaps_next(later.validity()) => Some(TaxPeriodsOverlap::new(
                later.validity().starts_on(),
                earlier.validity().starts_on(),
            )),
            _ => None,
        });

        overlap.map_or_else(|| Ok(Self { periods, currency }), Err)
    }

    pub fn tax_for(
        &self,
        vehicle: VehicleType,
        passages: &Passages,
    ) -> Result<TaxAssessment, PassageOutsideTaxPeriods> {
        let period_on = |date: Date| {
            let periods_started = self
                .periods
                .partition_point(|period| period.validity().starts_on() <= date);

            periods_started
                .checked_sub(1)
                .and_then(|latest| self.periods.get(latest))
                .filter(|period| date <= period.validity().ends_on())
        };

        let days = passages
            .by_day()
            .map(|(date, day_passages)| {
                let period = period_on(date).ok_or_else(|| PassageOutsideTaxPeriods::new(date))?;
                Ok(DailyTax::new(date, period.daily_tax(date, vehicle, day_passages)))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(TaxAssessment::new(days, self.currency.clone()))
    }
}
