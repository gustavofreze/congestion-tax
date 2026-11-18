use serde::Deserialize;

use super::stored_tax_rules::StoredTaxRules;
use crate::application::domain::errors::TaxPeriodsOverlap;
use crate::application::domain::models::tax_rules::TaxRules;

#[derive(Deserialize)]
#[serde(try_from = "StoredTaxRules")]
pub(in crate::driven::postgres) struct DecodedTaxRules(pub(in crate::driven::postgres) TaxRules);

impl TryFrom<StoredTaxRules> for DecodedTaxRules {
    type Error = TaxPeriodsOverlap;

    fn try_from(stored: StoredTaxRules) -> Result<Self, Self::Error> {
        TaxRules::new(stored.currency.0, stored.periods.into_iter().map(|period| period.0)).map(Self)
    }
}
