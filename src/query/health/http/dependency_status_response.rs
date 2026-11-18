use serde::Serialize;

use crate::query::health::DependencyStatus;

#[derive(Serialize)]
pub(super) struct DependencyStatusResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    status: &'static str,
    critical: bool,
    component: String,
    duration_in_milliseconds: f64,
}

impl DependencyStatusResponse {
    pub(super) const fn is_down(&self) -> bool {
        self.detail.is_some()
    }
}

impl From<DependencyStatus> for DependencyStatusResponse {
    fn from(dependency: DependencyStatus) -> Self {
        let status = match dependency.detail {
            Some(_) => "DOWN",
            None => "UP",
        };

        Self {
            detail: dependency.detail,
            status,
            critical: true,
            component: dependency.component,
            duration_in_milliseconds: (dependency.elapsed.as_secs_f64() * 100_000.0).round() / 100.0,
        }
    }
}
