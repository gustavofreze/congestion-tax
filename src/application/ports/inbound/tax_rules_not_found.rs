use thiserror::Error;

#[derive(Debug, Error)]
#[error("No tax rules found for the city.")]
pub struct TaxRulesNotFound;
