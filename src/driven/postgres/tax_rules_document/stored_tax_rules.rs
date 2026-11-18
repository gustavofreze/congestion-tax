use serde::Deserialize;

use super::decoded_currency::DecodedCurrency;
use super::decoded_tax_period::DecodedTaxPeriod;

#[derive(Deserialize)]
pub(super) struct StoredTaxRules {
    pub(super) periods: Vec<DecodedTaxPeriod>,
    pub(super) currency: DecodedCurrency,
}
