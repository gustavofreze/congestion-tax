mod app_settings;
mod database_settings;
mod environment_variable;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use congestion_tax::application::handlers::CalculatingCongestionTaxHandler;
use congestion_tax::driven::cache::{CachedTaxRulesCatalog, SystemMonotonicClock};
use congestion_tax::driven::postgres::PostgresTaxRulesCatalog;
use congestion_tax::query::health::PostgresDatabaseProbe;
use congestion_tax::query::tax_rules::PostgresTaxRulesFinding;
use congestion_tax::routes::router;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use tokio::net::TcpListener;
use tokio::signal::unix::{SignalKind, signal};

use crate::app_settings::AppSettings;
use crate::database_settings::DatabaseSettings;

const DATABASE_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(3);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();

    let app = AppSettings::from_environment()?;
    let database = DatabaseSettings::from_environment()?;

    let pool = PgPoolOptions::new()
        .acquire_timeout(DATABASE_ACQUIRE_TIMEOUT)
        .connect_lazy_with(
            PgConnectOptions::new()
                .host(&database.host)
                .port(database.port)
                .database(&database.name)
                .password(&database.password)
                .username(&database.user),
        );
    let readiness_probe = Arc::new(PostgresDatabaseProbe::new(pool.clone()));
    let tax_rules_finding = Arc::new(PostgresTaxRulesFinding::new(pool.clone()));
    let catalog = CachedTaxRulesCatalog::new(
        SystemMonotonicClock,
        PostgresTaxRulesCatalog::new(pool),
        app.tax_rules_cache_ttl,
    );
    let calculating_congestion_tax = Arc::new(CalculatingCongestionTaxHandler::new(catalog));

    let mut interrupt = signal(SignalKind::interrupt()).context("cannot listen for SIGINT")?;
    let mut terminate = signal(SignalKind::terminate()).context("cannot listen for SIGTERM")?;
    let listener = TcpListener::bind(app.http_address)
        .await
        .with_context(|| format!("cannot listen on {}", app.http_address))?;

    tracing::info!(address = %app.http_address, "Congestion tax service listening.");

    axum::serve(
        listener,
        router(readiness_probe, tax_rules_finding, calculating_congestion_tax),
    )
    .with_graceful_shutdown(async move {
        tokio::select! {
            _ = interrupt.recv() => {},
            _ = terminate.recv() => {},
        }
    })
    .await
    .context("HTTP server stopped unexpectedly")
}
