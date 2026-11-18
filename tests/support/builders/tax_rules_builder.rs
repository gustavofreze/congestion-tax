use congestion_tax::application::domain::models::tax_rules::{
    Amount, Currency, Month, SingleChargeWindow, TariffBand, TariffSchedule, TaxPeriod, TaxRules, TollFreeCalendar,
    ValidityPeriod, VehicleType,
};
use jiff::civil::{Date, Weekday, date, time};

pub(crate) struct TaxRulesBuilder {
    daily_maximum: u32,
}

impl TaxRulesBuilder {
    pub(crate) const fn gothenburg() -> Self {
        Self { daily_maximum: 60 }
    }

    pub(crate) fn build(self) -> TaxRules {
        TaxRules::new(
            swedish_kronor(),
            [gothenburg_period(
                date(2013, 1, 1),
                date(2013, 12, 31),
                self.daily_maximum,
            )],
        )
        .expect("a single period overlaps no other")
    }

    pub(crate) const fn with_daily_maximum(mut self, daily_maximum: u32) -> Self {
        self.daily_maximum = daily_maximum;
        self
    }
}

pub(crate) fn swedish_kronor() -> Currency {
    Currency::new("SEK").expect("SEK is an ISO 4217 code")
}

pub(crate) fn gothenburg_period(starts_on: Date, ends_on: Date, daily_maximum: u32) -> TaxPeriod {
    TaxPeriod::new(
        gothenburg_tariff(),
        gothenburg_calendar(),
        ValidityPeriod::new(starts_on, ends_on).expect("the period ends on or after it starts"),
        Amount::new(daily_maximum),
        SingleChargeWindow::of_minutes(60),
        [
            VehicleType::Bus,
            VehicleType::Foreign,
            VehicleType::Diplomat,
            VehicleType::Military,
            VehicleType::Emergency,
            VehicleType::Motorcycle,
        ],
    )
}

fn gothenburg_tariff() -> TariffSchedule {
    let bands = [
        (6, 0, 6, 30, 8),
        (6, 30, 7, 0, 13),
        (7, 0, 8, 0, 18),
        (8, 0, 8, 30, 13),
        (8, 30, 15, 0, 8),
        (15, 0, 15, 30, 13),
        (15, 30, 17, 0, 18),
        (17, 0, 18, 0, 13),
        (18, 0, 18, 30, 8),
    ]
    .map(|(start_hour, start_minute, end_hour, end_minute, amount)| {
        TariffBand::new(
            Amount::new(amount),
            time(start_hour, start_minute, 0, 0),
            time(end_hour, end_minute, 0, 0),
        )
        .expect("every Gothenburg band ends after it starts")
    });

    TariffSchedule::new(bands).expect("Gothenburg bands do not overlap")
}

fn gothenburg_calendar() -> TollFreeCalendar {
    let holidays: [Date; 14] = [
        date(2013, 1, 1),
        date(2013, 1, 6),
        date(2013, 3, 29),
        date(2013, 3, 31),
        date(2013, 4, 1),
        date(2013, 5, 1),
        date(2013, 5, 9),
        date(2013, 5, 19),
        date(2013, 6, 6),
        date(2013, 6, 22),
        date(2013, 11, 2),
        date(2013, 12, 25),
        date(2013, 12, 26),
        date(2014, 1, 1),
    ];

    TollFreeCalendar::new(
        true,
        [Month::new(7).expect("July is a month of the year")],
        holidays,
        [Weekday::Saturday, Weekday::Sunday],
    )
}
