use std::collections::HashMap;
use std::sync::{Arc, PoisonError, RwLock};
use std::time::Duration;

use super::MonotonicClock;
use super::cached_tax_rules::CachedTaxRules;
use crate::application::domain::models::tax_rules::{City, TaxRules};

pub(super) struct TaxRulesCache<K> {
    clock: K,
    entries: RwLock<HashMap<City, CachedTaxRules>>,
    time_to_live: Duration,
}

impl<K: MonotonicClock> TaxRulesCache<K> {
    pub(super) fn new(clock: K, time_to_live: Duration) -> Self {
        Self {
            clock,
            entries: RwLock::new(HashMap::new()),
            time_to_live,
        }
    }

    pub(super) fn fresh(&self, city: &City) -> Option<Arc<TaxRules>> {
        let now = self.clock.now();

        self.entries
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(city)
            .filter(|entry| now.duration_since(entry.cached_at()) < self.time_to_live)
            .map(|entry| Arc::clone(entry.rules()))
    }

    pub(super) fn keep(&self, city: &City, rules: &Arc<TaxRules>) {
        let entry = CachedTaxRules::new(Arc::clone(rules), self.clock.now());

        self.entries
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(city.clone(), entry);
    }
}
