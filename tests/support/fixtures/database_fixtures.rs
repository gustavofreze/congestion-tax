use std::env;
use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(3);

pub(crate) fn migrated_database() -> PgPool {
    let options = PgConnectOptions::new()
        .host(&variable("DATABASE_HOST"))
        .port(
            variable("DATABASE_PORT")
                .parse()
                .expect("DATABASE_PORT is a port number"),
        )
        .database(&variable("DATABASE_NAME"))
        .password(&variable("DATABASE_PASSWORD"))
        .username(&variable("DATABASE_USER"));

    PgPoolOptions::new()
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect_lazy_with(options)
}

pub(crate) fn unreachable_database() -> PgPool {
    let options = PgConnectOptions::new()
        .host("127.0.0.1")
        .port(1)
        .database("none")
        .password("none")
        .username("none");

    PgPoolOptions::new()
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect_lazy_with(options)
}

pub(crate) async fn store_tax_rules_with_overlapping_bands(database: &PgPool) {
    sqlx::raw_sql(
        r"
        INSERT INTO tax_rules (id, city, currency)
        VALUES ('01a1e6cd-a800-7000-8000-0000000000f1', 'overlapping-bands', 'SEK')
        ON CONFLICT DO NOTHING;

        INSERT INTO tax_periods (id, tax_rule_id, toll_free_months, toll_free_weekdays, toll_free_day_before_holiday,
                                 daily_maximum_amount, single_charge_window_minutes, starts_on, ends_on)
        VALUES ('01a1e6cd-a800-7000-8000-0000000000f2', '01a1e6cd-a800-7000-8000-0000000000f1',
                '{}', '{}', FALSE, 60, 60, '2013-01-01', '2013-12-31')
        ON CONFLICT DO NOTHING;

        INSERT INTO tariff_bands (id, amount, tax_period_id, starts_at, ends_at)
        VALUES ('01a1e6cd-a800-7000-8000-0000000000f3', 8, '01a1e6cd-a800-7000-8000-0000000000f2', '06:00', '07:00'),
               ('01a1e6cd-a800-7000-8000-0000000000f4', 13, '01a1e6cd-a800-7000-8000-0000000000f2', '06:30', '07:30')
        ON CONFLICT DO NOTHING;
        ",
    )
    .execute(database)
    .await
    .expect("the overlapping bands are stored");
}

fn variable(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("the variable <{name}> points at the migrated test database"))
}
