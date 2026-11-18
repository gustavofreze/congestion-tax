use std::future::ready;
use std::sync::{Arc, Mutex};

use congestion_tax::application::domain::models::tax_rules::{City, TaxRules};
use congestion_tax::application::ports::outbound::{TaxRulesCatalog, TaxRulesUnavailable};

use super::tax_rules_catalog_outcome::TaxRulesCatalogOutcome;

pub(crate) struct TaxRulesCatalogSpy {
    outcome: TaxRulesCatalogOutcome,
    searched_cities: Mutex<Vec<City>>,
}

impl TaxRulesCatalogSpy {
    pub(crate) fn empty() -> Self {
        Self::answering(TaxRulesCatalogOutcome::Missing)
    }

    pub(crate) fn holding(rules: TaxRules) -> Self {
        Self::answering(TaxRulesCatalogOutcome::Found(Arc::new(rules)))
    }

    pub(crate) fn unavailable() -> Self {
        Self::answering(TaxRulesCatalogOutcome::Unavailable)
    }

    pub(crate) fn searched_cities(&self) -> Vec<City> {
        self.searched_cities
            .lock()
            .expect("searched cities are never poisoned")
            .clone()
    }

    fn answering(outcome: TaxRulesCatalogOutcome) -> Self {
        Self {
            outcome,
            searched_cities: Mutex::new(Vec::new()),
        }
    }
}

impl TaxRulesCatalog for &TaxRulesCatalogSpy {
    fn find(&self, city: &City) -> impl Future<Output = Result<Option<Arc<TaxRules>>, TaxRulesUnavailable>> + Send {
        self.searched_cities
            .lock()
            .expect("searched cities are never poisoned")
            .push(city.clone());

        ready(match &self.outcome {
            TaxRulesCatalogOutcome::Found(rules) => Ok(Some(Arc::clone(rules))),
            TaxRulesCatalogOutcome::Missing => Ok(None),
            TaxRulesCatalogOutcome::Unavailable => Err(TaxRulesUnavailable::new("connection refused")),
        })
    }
}
