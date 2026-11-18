use std::sync::Arc;

use congestion_tax::application::domain::models::tax_rules::TaxRules;

pub(crate) enum TaxRulesCatalogOutcome {
    Found(Arc<TaxRules>),
    Missing,
    Unavailable,
}
