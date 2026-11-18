use thiserror::Error;

#[derive(Debug, Error)]
#[error("A city cannot be empty.")]
pub struct CityCannotBeEmpty;
