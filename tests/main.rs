mod support {
    pub(crate) mod builders {
        pub(crate) mod tax_rules_builder;
    }

    pub(crate) mod spies {
        pub(crate) mod calculating_congestion_tax_spy;
        pub(crate) mod dependency_probe_spy;
        pub(crate) mod monotonic_clock_spy;
        pub(crate) mod tax_rules_catalog_outcome;
        pub(crate) mod tax_rules_catalog_spy;
    }

    pub(crate) mod fixtures {
        pub(crate) mod database_fixtures;
    }
}

mod application {
    mod domain {
        mod models {
            mod tax_rules;
        }
    }

    mod handlers {
        mod calculating_congestion_tax_handler;
    }
}

mod driven {
    mod cache {
        mod cached_tax_rules_catalog;
    }

    mod postgres {
        mod postgres_tax_rules_catalog;
    }
}

mod driver {
    mod http {
        mod calculate_congestion_tax_endpoint;
    }
}

mod query {
    mod health {
        mod database {
            mod postgres_database_probe;
        }

        mod http {
            mod liveness_endpoint;
            mod readiness_endpoint;
        }
    }

    mod tax_rules {
        mod database {
            mod postgres_tax_rules_finding;
        }

        mod http {
            mod find_tax_rules_by_city_endpoint;
        }
    }
}

mod routes;
