use serde::Deserialize;

use crate::application::domain::errors::CurrencyFormatNotValid;
use crate::application::domain::models::tax_rules::Currency;

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub(super) struct DecodedCurrency(pub(super) Currency);

impl TryFrom<String> for DecodedCurrency {
    type Error = CurrencyFormatNotValid;

    fn try_from(code: String) -> Result<Self, Self::Error> {
        Currency::new(&code).map(Self)
    }
}
