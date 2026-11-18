use std::time::Duration;

use congestion_tax::query::health::{DependencyProbe, PostgresDatabaseProbe};

use crate::support::fixtures::database_fixtures::{migrated_database, unreachable_database};

#[tokio::test]
async fn probe_when_database_answers_then_reports_it_up() {
    // Given the migrated database
    let database = PostgresDatabaseProbe::new(migrated_database());

    // When the database is probed
    let status = database.probe().await;

    // Then the database is reported up, with no failure
    assert_eq!(status.detail, None);

    // And the probe measured the time the database took to answer
    assert!(status.elapsed > Duration::ZERO);

    // And the probe names the database
    assert_eq!(status.component, "database");
}

#[tokio::test]
async fn probe_when_database_is_unreachable_then_reports_it_down_with_the_failure() {
    // Given a port no database listens on
    let database = PostgresDatabaseProbe::new(unreachable_database());

    // When the database is probed
    let status = database.probe().await;

    // Then the database is reported down, with the failure that made it so
    assert!(status.detail.is_some());
}
