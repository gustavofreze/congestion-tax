use thiserror::Error;

#[derive(Debug, Error)]
#[error("Currency <{raw}> is not a three-letter ISO 4217 code.")]
pub struct CurrencyFormatNotValid {
    raw: String,
}

impl CurrencyFormatNotValid {
    pub(crate) fn new(raw: &str) -> Self {
        Self { raw: raw.to_owned() }
    }
}
