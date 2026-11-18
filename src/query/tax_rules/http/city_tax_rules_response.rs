use serde::Serialize;

use super::city_tax_period_response::CityTaxPeriodResponse;
use crate::query::tax_rules::CityTaxRules;

#[derive(Serialize)]
pub(super) struct CityTaxRulesResponse {
    city: String,
    periods: Vec<CityTaxPeriodResponse>,
    currency: String,
}

impl From<CityTaxRules> for CityTaxRulesResponse {
    fn from(rules: CityTaxRules) -> Self {
        Self {
            city: rules.city,
            periods: rules.periods.into_iter().map(CityTaxPeriodResponse::from).collect(),
            currency: rules.currency,
        }
    }
}
