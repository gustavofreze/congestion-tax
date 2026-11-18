use congestion_tax::application::domain::errors::{
    CurrencyFormatNotValid, PassageOutsideTaxPeriods, TariffBandEndsBeforeStart, TariffBandsOverlap, TaxPeriodsOverlap,
    UnsupportedMonth, ValidityPeriodEndsBeforeStart,
};
use congestion_tax::application::domain::models::tax_rules::{
    Amount, Currency, DailyTax, Money, Month, Passage, Passages, TariffBand, TariffSchedule, TaxRules, ValidityPeriod,
    VehicleType,
};
use jiff::civil::{date, time};
use rstest::rstest;

use crate::support::builders::tax_rules_builder::{TaxRulesBuilder, gothenburg_period, swedish_kronor};

fn passages(raw: &[&str]) -> Passages {
    raw.iter()
        .map(|passage| {
            passage
                .parse::<Passage>()
                .expect("every passage is a local date and time")
        })
        .collect()
}

fn kronor(amount: u32) -> Money {
    Money::new(Amount::new(amount), swedish_kronor())
}

#[test]
fn tax_for_when_car_passes_on_the_dates_of_the_challenge_example_then_owes_89_kronor() {
    // Given the Gothenburg tax rules
    let rules = TaxRulesBuilder::gothenburg().build();

    // And the passages of the challenge example, out of chronological order
    let challenge_example = passages(&[
        "2013-01-14 21:00:00",
        "2013-01-15 21:00:00",
        "2013-02-07 06:23:27",
        "2013-02-07 15:27:00",
        "2013-02-08 06:27:00",
        "2013-02-08 06:20:27",
        "2013-02-08 14:35:00",
        "2013-02-08 15:29:00",
        "2013-02-08 15:47:00",
        "2013-02-08 16:01:00",
        "2013-02-08 16:48:00",
        "2013-02-08 17:49:00",
        "2013-02-08 18:29:00",
        "2013-02-08 18:35:00",
        "2013-03-26 14:25:00",
        "2013-03-28 14:07:27",
    ]);

    // When the tax of a car is calculated
    let assessment = rules
        .tax_for(VehicleType::Car, &challenge_example)
        .expect("every passage falls within a tax period");

    // Then the car owes 89 kronor
    assert_eq!(assessment.total(), kronor(89));

    // And each day of the passages carries its own tax, in chronological order
    assert_eq!(
        assessment.days(),
        [
            DailyTax::new(date(2013, 1, 14), Amount::new(0)),
            DailyTax::new(date(2013, 1, 15), Amount::new(0)),
            DailyTax::new(date(2013, 2, 7), Amount::new(21)),
            DailyTax::new(date(2013, 2, 8), Amount::new(60)),
            DailyTax::new(date(2013, 3, 26), Amount::new(8)),
            DailyTax::new(date(2013, 3, 28), Amount::new(0)),
        ]
    );
}

#[test]
fn tax_for_when_vehicle_made_no_passages_then_owes_nothing() {
    // Given the Gothenburg tax rules
    let rules = TaxRulesBuilder::gothenburg().build();

    // When the tax of a car with no passages is calculated
    let assessment = rules
        .tax_for(VehicleType::Car, &passages(&[]))
        .expect("no passage falls outside a tax period");

    // Then the car owes nothing
    assert_eq!(assessment.total(), kronor(0));
}

#[rstest]
#[case::before_the_first_band("2013-02-05 05:59:59", 0)]
#[case::start_of_the_first_band("2013-02-05 06:00:00", 8)]
#[case::end_of_the_first_band("2013-02-05 06:29:59", 8)]
#[case::start_of_the_morning_shoulder("2013-02-05 06:30:00", 13)]
#[case::end_of_the_morning_shoulder("2013-02-05 06:59:59", 13)]
#[case::start_of_the_morning_peak("2013-02-05 07:00:00", 18)]
#[case::end_of_the_morning_peak("2013-02-05 07:59:59", 18)]
#[case::after_the_morning_peak("2013-02-05 08:00:00", 13)]
#[case::start_of_the_day("2013-02-05 08:30:00", 8)]
#[case::first_half_of_a_day_hour("2013-02-05 09:15:00", 8)]
#[case::end_of_the_day("2013-02-05 14:59:59", 8)]
#[case::start_of_the_afternoon_shoulder("2013-02-05 15:00:00", 13)]
#[case::start_of_the_afternoon_peak("2013-02-05 15:30:00", 18)]
#[case::end_of_the_afternoon_peak("2013-02-05 16:59:59", 18)]
#[case::start_of_the_evening_shoulder("2013-02-05 17:00:00", 13)]
#[case::start_of_the_last_band("2013-02-05 18:00:00", 8)]
#[case::end_of_the_last_band("2013-02-05 18:29:59", 8)]
#[case::after_the_last_band("2013-02-05 18:30:00", 0)]
#[case::night("2013-02-05 23:00:00", 0)]
fn tax_for_when_car_passes_once_on_a_weekday_then_owes_the_amount_of_its_band(
    #[case] passage: &str,
    #[case] amount: u32,
) {
    // Given the Gothenburg tax rules
    let rules = TaxRulesBuilder::gothenburg().build();

    // When the tax of a car passing once on a Tuesday is calculated
    let assessment = rules
        .tax_for(VehicleType::Car, &passages(&[passage]))
        .expect("the passage falls within a tax period");

    // Then the car owes the amount of the band its passage falls in
    assert_eq!(assessment.total(), kronor(amount));
}

#[rstest]
#[case::saturday("2013-02-09 07:00:00")]
#[case::sunday("2013-02-10 07:00:00")]
#[case::public_holiday("2013-05-01 07:00:00")]
#[case::day_before_a_public_holiday("2013-04-30 07:00:00")]
#[case::day_before_a_public_holiday_of_the_next_year("2013-12-31 07:00:00")]
#[case::july("2013-07-15 07:00:00")]
fn tax_for_when_car_passes_on_a_toll_free_day_then_owes_nothing(#[case] passage: &str) {
    // Given the Gothenburg tax rules
    let rules = TaxRulesBuilder::gothenburg().build();

    // When the tax of a car passing in the morning peak of a toll free day is calculated
    let assessment = rules
        .tax_for(VehicleType::Car, &passages(&[passage]))
        .expect("the passage falls within a tax period");

    // Then the car owes nothing
    assert_eq!(assessment.total(), kronor(0));
}

#[rstest]
#[case::bus(VehicleType::Bus)]
#[case::foreign(VehicleType::Foreign)]
#[case::diplomat(VehicleType::Diplomat)]
#[case::military(VehicleType::Military)]
#[case::emergency(VehicleType::Emergency)]
#[case::motorcycle(VehicleType::Motorcycle)]
fn tax_for_when_vehicle_is_exempt_then_owes_nothing(#[case] vehicle: VehicleType) {
    // Given the Gothenburg tax rules
    let rules = TaxRulesBuilder::gothenburg().build();

    // When the tax of an exempt vehicle passing in the morning peak of a weekday is calculated
    let assessment = rules
        .tax_for(vehicle, &passages(&["2013-02-05 07:00:00"]))
        .expect("the passage falls within a tax period");

    // Then the vehicle owes nothing
    assert_eq!(assessment.total(), kronor(0));
}

#[rstest]
#[case::one_second_before_the_window_closes("2013-02-05 06:59:59", 13)]
#[case::when_the_window_closes("2013-02-05 07:00:00", 18)]
#[case::one_second_after_the_window_closes("2013-02-05 07:00:01", 26)]
fn tax_for_when_car_passes_twice_around_the_end_of_the_window_then_owes_the_single_charge_rule_amount(
    #[case] second_passage: &str,
    #[case] amount: u32,
) {
    // Given the Gothenburg tax rules
    let rules = TaxRulesBuilder::gothenburg().build();

    // When the tax of a car passing at 06:00 and passing again around 07:00 is calculated
    let assessment = rules
        .tax_for(VehicleType::Car, &passages(&["2013-02-05 06:00:00", second_passage]))
        .expect("every passage falls within a tax period");

    // Then the car owes one highest fee within the 60 minutes, and every fee after them
    assert_eq!(assessment.total(), kronor(amount));
}

#[test]
fn tax_for_when_passages_chain_beyond_60_minutes_then_window_stays_anchored_at_its_first_passage() {
    // Given the Gothenburg tax rules
    let rules = TaxRulesBuilder::gothenburg().build();

    // When the tax of a car passing at 06:00, 06:50 and 07:40 is calculated
    let assessment = rules
        .tax_for(
            VehicleType::Car,
            &passages(&["2013-02-05 06:00:00", "2013-02-05 06:50:00", "2013-02-05 07:40:00"]),
        )
        .expect("every passage falls within a tax period");

    // Then the car owes 13 kronor for the window opened at 06:00 and 18 kronor for the one opened at 07:40
    assert_eq!(assessment.total(), kronor(31));
}

#[test]
fn tax_for_when_a_free_passage_precedes_charged_ones_then_window_opens_at_the_first_charged_passage() {
    // Given the Gothenburg tax rules
    let rules = TaxRulesBuilder::gothenburg().build();

    // When the tax of a car passing at 05:50, which is free, and then at 06:20 and 06:55 is calculated
    let assessment = rules
        .tax_for(
            VehicleType::Car,
            &passages(&["2013-02-05 05:50:00", "2013-02-05 06:20:00", "2013-02-05 06:55:00"]),
        )
        .expect("every passage falls within a tax period");

    // Then the car owes the highest fee of the window opened at 06:20
    assert_eq!(assessment.total(), kronor(13));
}

#[rstest]
#[case::below_the_charges_of_the_day(25, 25)]
#[case::equal_to_the_charges_of_the_day(26, 26)]
#[case::above_the_charges_of_the_day(27, 26)]
fn tax_for_when_daily_maximum_is_around_the_charges_of_the_day_then_owes_at_most_the_daily_maximum(
    #[case] daily_maximum: u32,
    #[case] amount: u32,
) {
    // Given tax rules with a daily maximum around 26 kronor
    let rules = TaxRulesBuilder::gothenburg().with_daily_maximum(daily_maximum).build();

    // When the tax of a car charged 8 kronor at 06:00 and 18 kronor at 07:30 is calculated
    let assessment = rules
        .tax_for(
            VehicleType::Car,
            &passages(&["2013-02-05 06:00:00", "2013-02-05 07:30:00"]),
        )
        .expect("every passage falls within a tax period");

    // Then the car owes the charges of the day, capped at the daily maximum
    assert_eq!(assessment.total(), kronor(amount));
}

#[test]
fn tax_for_when_passages_span_two_tax_periods_then_each_day_follows_the_rules_of_its_period() {
    // Given a 2013 period capped at 60 kronor a day
    let year_2013 = gothenburg_period(date(2013, 1, 1), date(2013, 12, 31), 60);

    // And a 2014 period capped at 20 kronor a day
    let year_2014 = gothenburg_period(date(2014, 1, 1), date(2014, 12, 31), 20);

    // And the tax rules of a city holding both periods
    let rules = TaxRules::new(swedish_kronor(), [year_2014, year_2013]).expect("the periods do not overlap");

    // When the tax of a car charged 8 and 18 kronor on a Tuesday of each year is calculated
    let assessment = rules
        .tax_for(
            VehicleType::Car,
            &passages(&[
                "2013-02-05 06:00:00",
                "2013-02-05 07:30:00",
                "2014-02-04 06:00:00",
                "2014-02-04 07:30:00",
            ]),
        )
        .expect("every passage falls within a tax period");

    // Then the 2013 day owes its 26 kronor and the 2014 day is capped at 20 kronor
    assert_eq!(
        assessment.days(),
        [
            DailyTax::new(date(2013, 2, 5), Amount::new(26)),
            DailyTax::new(date(2014, 2, 4), Amount::new(20)),
        ]
    );
}

#[rstest]
#[case::first_day_of_the_period("2013-01-02 07:00:00", 18)]
#[case::last_day_of_the_period("2013-12-30 07:00:00", 18)]
fn tax_for_when_passage_falls_on_the_edge_of_a_tax_period_then_follows_that_period(
    #[case] passage: &str,
    #[case] amount: u32,
) {
    // Given tax rules with one period from 2013-01-02 to 2013-12-30
    let rules = TaxRules::new(
        swedish_kronor(),
        [gothenburg_period(date(2013, 1, 2), date(2013, 12, 30), 60)],
    )
    .expect("a single period overlaps no other");

    // When the tax of a car passing in the morning peak of an edge day of the period is calculated
    let assessment = rules
        .tax_for(VehicleType::Car, &passages(&[passage]))
        .expect("the passage falls within the period");

    // Then the car owes the fee of the period
    assert_eq!(assessment.total(), kronor(amount));
}

#[rstest]
#[case::day_before_the_period_starts("2013-01-01 07:00:00")]
#[case::day_after_the_period_ends("2013-12-31 07:00:00")]
fn tax_for_when_passage_falls_outside_every_tax_period_then_fails(#[case] passage: &str) {
    // Given tax rules with one period from 2013-01-02 to 2013-12-30
    let rules = TaxRules::new(
        swedish_kronor(),
        [gothenburg_period(date(2013, 1, 2), date(2013, 12, 30), 60)],
    )
    .expect("a single period overlaps no other");

    // When the tax of a car passing on a day next to the period is calculated
    let assessment = rules.tax_for(VehicleType::Car, &passages(&[passage]));

    // Then the calculation is refused because no tax period covers the passage
    assert!(matches!(assessment, Err(PassageOutsideTaxPeriods { .. })));
}

#[test]
fn new_when_tax_periods_share_a_day_then_fails() {
    // Given a period ending on 2013-12-31
    let year_2013 = gothenburg_period(date(2013, 1, 1), date(2013, 12, 31), 60);

    // And a period starting on that same day
    let overlapping = gothenburg_period(date(2013, 12, 31), date(2014, 12, 31), 60);

    // When the tax rules are created from both periods
    let rules = TaxRules::new(swedish_kronor(), [overlapping, year_2013]);

    // Then the tax rules are refused because the periods overlap
    assert!(matches!(rules, Err(TaxPeriodsOverlap { .. })));
}

#[test]
fn new_when_validity_period_ends_before_it_starts_then_fails() {
    // Given a period that ends the day before it starts
    // When the validity period is created
    let validity = ValidityPeriod::new(date(2013, 1, 2), date(2013, 1, 1));

    // Then the validity period is refused because it ends before it starts
    assert!(matches!(validity, Err(ValidityPeriodEndsBeforeStart { .. })));
}

#[test]
fn new_when_validity_period_starts_and_ends_on_the_same_day_then_covers_that_day() {
    // Given tax rules with one period of a single Tuesday
    let rules = TaxRules::new(
        swedish_kronor(),
        [gothenburg_period(date(2013, 2, 5), date(2013, 2, 5), 60)],
    )
    .expect("a single period overlaps no other");

    // When the tax of a car passing in the morning peak of that Tuesday is calculated
    let assessment = rules
        .tax_for(VehicleType::Car, &passages(&["2013-02-05 07:00:00"]))
        .expect("the passage falls within the period");

    // Then the car owes the fee of the period
    assert_eq!(assessment.total(), kronor(18));
}

#[rstest]
#[case::ends_before_it_starts(7, 6)]
#[case::ends_when_it_starts(7, 7)]
fn new_when_tariff_band_does_not_end_after_it_starts_then_fails(#[case] start_hour: i8, #[case] end_hour: i8) {
    // Given a tariff band that does not end after it starts
    // When the band is created
    let band = TariffBand::new(Amount::new(8), time(start_hour, 0, 0, 0), time(end_hour, 0, 0, 0));

    // Then the band is refused because it ends before it starts
    assert!(matches!(band, Err(TariffBandEndsBeforeStart { .. })));
}

#[test]
fn new_when_tariff_bands_overlap_then_fails() {
    // Given a band from 06:00 to 07:00
    let morning = TariffBand::new(Amount::new(8), time(6, 0, 0, 0), time(7, 0, 0, 0)).expect("band ends after start");

    // And a band from 06:30 to 07:30
    let overlapping =
        TariffBand::new(Amount::new(13), time(6, 30, 0, 0), time(7, 30, 0, 0)).expect("band ends after start");

    // When the schedule is created from both bands
    let schedule = TariffSchedule::new([overlapping, morning]);

    // Then the schedule is refused because the bands overlap
    assert!(matches!(schedule, Err(TariffBandsOverlap { .. })));
}

#[rstest]
#[case::lowercase("sek")]
#[case::too_short("SE")]
#[case::too_long("SEKK")]
#[case::with_a_digit("S3K")]
fn new_when_currency_is_not_an_iso_code_then_fails(#[case] code: &str) {
    // Given a currency code that is not three uppercase letters
    // When the currency is created
    let currency = Currency::new(code);

    // Then the currency is refused because its format is not valid
    assert!(matches!(currency, Err(CurrencyFormatNotValid { .. })));
}

#[rstest]
#[case::below_january(0)]
#[case::above_december(13)]
fn new_when_month_is_outside_the_year_then_fails(#[case] number: i8) {
    // Given a month number outside 1 to 12
    // When the month is created
    let month = Month::new(number);

    // Then the month is refused because it is not supported
    assert!(matches!(month, Err(UnsupportedMonth { .. })));
}
