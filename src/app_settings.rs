use std::net::SocketAddr;
use std::time::Duration;

use crate::environment_variable::EnvironmentVariable;

pub(crate) struct AppSettings {
    pub(crate) http_address: SocketAddr,
    pub(crate) tax_rules_cache_ttl: Duration,
}

impl AppSettings {
    pub(crate) fn from_environment() -> anyhow::Result<Self> {
        Ok(Self {
            http_address: EnvironmentVariable::from("HTTP_ADDRESS")?.parsed()?,
            tax_rules_cache_ttl: Duration::from_secs(
                EnvironmentVariable::from("TAX_RULES_CACHE_TTL_SECONDS")?.parsed()?,
            ),
        })
    }
}
