use std::error::Error as StdError;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("The tax rules catalog is unavailable.")]
pub struct TaxRulesUnavailable {
    #[source]
    cause: Box<dyn StdError + Send + Sync>,
}

impl TaxRulesUnavailable {
    pub fn new(cause: impl Into<Box<dyn StdError + Send + Sync>>) -> Self {
        Self { cause: cause.into() }
    }
}
