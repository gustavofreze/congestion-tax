use std::collections::BTreeSet;

use jiff::civil::Date;

use super::charge_window::ChargeWindow;
use super::{Amount, Passage, SingleChargeWindow, TariffSchedule, TollFreeCalendar, ValidityPeriod, VehicleType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxPeriod {
    tariff: TariffSchedule,
    calendar: TollFreeCalendar,
    validity: ValidityPeriod,
    daily_maximum: Amount,
    exempt_vehicles: BTreeSet<VehicleType>,
    single_charge_window: SingleChargeWindow,
}

impl TaxPeriod {
    #[must_use]
    pub fn new(
        tariff: TariffSchedule,
        calendar: TollFreeCalendar,
        validity: ValidityPeriod,
        daily_maximum: Amount,
        single_charge_window: SingleChargeWindow,
        exempt_vehicles: impl IntoIterator<Item = VehicleType>,
    ) -> Self {
        Self {
            tariff,
            calendar,
            validity,
            daily_maximum,
            exempt_vehicles: exempt_vehicles.into_iter().collect(),
            single_charge_window,
        }
    }

    pub(super) const fn validity(&self) -> ValidityPeriod {
        self.validity
    }

    pub(super) fn daily_tax(&self, date: Date, vehicle: VehicleType, day_passages: &[Passage]) -> Amount {
        if self.exempt_vehicles.contains(&vehicle) || self.calendar.is_toll_free(date) {
            return Amount::ZERO;
        }

        let (charged, open_window) = day_passages
            .iter()
            .map(|passage| (*passage, self.tariff.fee_at(passage.time())))
            .filter(|(_, fee)| !fee.is_zero())
            .fold(
                (Amount::ZERO, None::<ChargeWindow>),
                |(charged, window), (passage, fee)| match window {
                    Some(window) if window.contains(passage) => (charged, Some(window.raised_to(fee))),
                    Some(window) => (
                        window.charged_onto(charged),
                        Some(ChargeWindow::open(fee, self.single_charge_window, passage)),
                    ),
                    None => (
                        charged,
                        Some(ChargeWindow::open(fee, self.single_charge_window, passage)),
                    ),
                },
            );

        open_window
            .map_or(charged, |window| window.charged_onto(charged))
            .min(self.daily_maximum)
    }
}
