use std::error::Error as StdError;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("The tax rules cannot be read.")]
pub struct TaxRulesFindingUnavailable {
    #[source]
    cause: Box<dyn StdError + Send + Sync>,
}

impl TaxRulesFindingUnavailable {
    pub fn new(cause: impl Into<Box<dyn StdError + Send + Sync>>) -> Self {
        Self { cause: cause.into() }
    }
}
