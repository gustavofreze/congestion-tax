use serde::Deserialize;

use crate::application::domain::errors::UnsupportedMonth;
use crate::application::domain::models::tax_rules::Month;

#[derive(Deserialize)]
#[serde(try_from = "i8")]
pub(super) struct DecodedMonth(pub(super) Month);

impl TryFrom<i8> for DecodedMonth {
    type Error = UnsupportedMonth;

    fn try_from(number: i8) -> Result<Self, Self::Error> {
        Month::new(number).map(Self)
    }
}
