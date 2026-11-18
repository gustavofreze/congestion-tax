use super::{CityTaxRules, TaxRulesFindingUnavailable};

/// Finding of the tax rules a city keeps, as the editors of each city maintain them.
pub trait TaxRulesFinding: Send + Sync {
    /// Finds the tax rules of the city, named in any case and with surrounding spaces, or none when the city keeps no
    /// tax rules.
    ///
    /// Fails with `TaxRulesFindingUnavailable` when the tax rules cannot be read.
    fn find(&self, city: &str)
    -> impl Future<Output = Result<Option<CityTaxRules>, TaxRulesFindingUnavailable>> + Send;
}
