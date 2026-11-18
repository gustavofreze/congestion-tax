use jiff::civil::Weekday;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(try_from = "i8")]
pub(super) struct DecodedWeekday(pub(super) Weekday);

impl TryFrom<i8> for DecodedWeekday {
    type Error = jiff::Error;

    fn try_from(number: i8) -> Result<Self, Self::Error> {
        Weekday::from_monday_one_offset(number).map(Self)
    }
}
