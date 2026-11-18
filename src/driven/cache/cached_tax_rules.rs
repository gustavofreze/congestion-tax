use std::sync::Arc;
use std::time::Instant;

use crate::application::domain::models::tax_rules::TaxRules;

pub(super) struct CachedTaxRules {
    rules: Arc<TaxRules>,
    cached_at: Instant,
}

impl CachedTaxRules {
    pub(super) const fn new(rules: Arc<TaxRules>, cached_at: Instant) -> Self {
        Self { rules, cached_at }
    }

    pub(super) const fn rules(&self) -> &Arc<TaxRules> {
        &self.rules
    }

    pub(super) const fn cached_at(&self) -> Instant {
        self.cached_at
    }
}
