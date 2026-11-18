use std::sync::Arc;

use super::TaxRulesUnavailable;
use crate::application::domain::models::tax_rules::{City, TaxRules};

/// The tax rules of every city, kept as content outside the application and maintained by the editors of each city.
pub trait TaxRulesCatalog: Send + Sync {
    /// Finds the tax rules the city applies, or none when the city has no rules in the catalog.
    ///
    /// Fails with `TaxRulesUnavailable` when the catalog cannot be read, or when the rules it holds break an invariant
    /// of the domain.
    fn find(&self, city: &City) -> impl Future<Output = Result<Option<Arc<TaxRules>>, TaxRulesUnavailable>> + Send;
}
