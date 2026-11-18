use std::time::Instant;

use sqlx::PgPool;

use crate::query::health::{DependencyProbe, DependencyStatus};

const PROBE_DATABASE: &str = r"SELECT 1";

pub struct PostgresDatabaseProbe {
    pool: PgPool,
}

impl PostgresDatabaseProbe {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl DependencyProbe for PostgresDatabaseProbe {
    async fn probe(&self) -> DependencyStatus {
        let started_at = Instant::now();
        let outcome = sqlx::query(PROBE_DATABASE).execute(&self.pool).await;

        DependencyStatus {
            detail: outcome.err().map(|failure| failure.to_string()),
            elapsed: started_at.elapsed(),
            component: "database".to_owned(),
        }
    }
}
