use super::CityTaxPeriod;

#[derive(Debug)]
pub struct CityTaxRules {
    pub city: String,
    pub periods: Vec<CityTaxPeriod>,
    pub currency: String,
}
