use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};

use crate::application::ports::inbound::CalculatingCongestionTax;
use crate::driver::http::{calculate_congestion_tax, method_not_allowed, route_not_found};
use crate::query::health::{DependencyProbe, liveness, readiness};
use crate::query::tax_rules::{TaxRulesFinding, find_tax_rules_by_city};

pub fn router<P, D, F>(readiness_probe: Arc<D>, tax_rules_finding: Arc<F>, calculating_congestion_tax: Arc<P>) -> Router
where
    P: CalculatingCongestionTax + 'static,
    D: DependencyProbe + 'static,
    F: TaxRulesFinding + 'static,
{
    let calculation = Router::new()
        .route("/congestion-taxes", post(calculate_congestion_tax::<P>))
        .with_state(calculating_congestion_tax);

    let tax_rules = Router::new()
        .route("/tax-rules/{city}", get(find_tax_rules_by_city::<F>))
        .with_state(tax_rules_finding);

    let health = Router::new()
        .route("/health/liveness", get(liveness))
        .route("/health/readiness", get(readiness::<D>))
        .with_state(readiness_probe);

    calculation
        .merge(tax_rules)
        .merge(health)
        .fallback(route_not_found)
        .method_not_allowed_fallback(method_not_allowed)
}
