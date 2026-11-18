use std::sync::Arc;
use std::time::Duration;

use super::MonotonicClock;
use super::tax_rules_cache::TaxRulesCache;
use crate::application::domain::models::tax_rules::{City, TaxRules};
use crate::application::ports::outbound::{TaxRulesCatalog, TaxRulesUnavailable};

pub struct CachedTaxRulesCatalog<C, K> {
    cache: TaxRulesCache<K>,
    origin: C,
}

impl<C: TaxRulesCatalog, K: MonotonicClock> CachedTaxRulesCatalog<C, K> {
    #[must_use]
    pub fn new(clock: K, origin: C, time_to_live: Duration) -> Self {
        Self {
            cache: TaxRulesCache::new(clock, time_to_live),
            origin,
        }
    }
}

impl<C: TaxRulesCatalog, K: MonotonicClock> TaxRulesCatalog for CachedTaxRulesCatalog<C, K> {
    async fn find(&self, city: &City) -> Result<Option<Arc<TaxRules>>, TaxRulesUnavailable> {
        if let Some(rules) = self.cache.fresh(city) {
            return Ok(Some(rules));
        }

        let found = self.origin.find(city).await?;

        if let Some(rules) = &found {
            self.cache.keep(city, rules);
        }

        Ok(found)
    }
}
