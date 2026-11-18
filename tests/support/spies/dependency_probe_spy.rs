use std::future::ready;
use std::time::Duration;

use congestion_tax::query::health::{DependencyProbe, DependencyStatus};

pub(crate) struct DependencyProbeSpy {
    status: DependencyStatus,
}

impl DependencyProbeSpy {
    pub(crate) fn up(elapsed: Duration) -> Self {
        Self::answering(None, elapsed)
    }

    pub(crate) fn down(detail: &str, elapsed: Duration) -> Self {
        Self::answering(Some(detail.to_owned()), elapsed)
    }

    fn answering(detail: Option<String>, elapsed: Duration) -> Self {
        Self {
            status: DependencyStatus {
                detail,
                elapsed,
                component: "database".to_owned(),
            },
        }
    }
}

impl DependencyProbe for DependencyProbeSpy {
    fn probe(&self) -> impl Future<Output = DependencyStatus> + Send {
        ready(self.status.clone())
    }
}
