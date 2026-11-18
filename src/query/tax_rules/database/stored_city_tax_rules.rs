use serde::Deserialize;

use super::stored_city_tax_period::StoredCityTaxPeriod;
use crate::query::tax_rules::{CityTaxPeriod, CityTaxRules};

#[derive(Deserialize)]
pub(super) struct StoredCityTaxRules {
    city: String,
    periods: Vec<StoredCityTaxPeriod>,
    currency: String,
}

impl From<StoredCityTaxRules> for CityTaxRules {
    fn from(stored: StoredCityTaxRules) -> Self {
        Self {
            city: stored.city,
            periods: stored.periods.into_iter().map(CityTaxPeriod::from).collect(),
            currency: stored.currency,
        }
    }
}
