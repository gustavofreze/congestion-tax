use crate::application::domain::errors::CurrencyFormatNotValid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Currency(String);

impl Currency {
    pub fn new(code: &str) -> Result<Self, CurrencyFormatNotValid> {
        let is_iso_code = code.len() == 3 && code.bytes().all(|letter| letter.is_ascii_uppercase());

        if !is_iso_code {
            return Err(CurrencyFormatNotValid::new(code));
        }

        Ok(Self(code.to_owned()))
    }

    #[must_use]
    pub fn code(&self) -> &str {
        &self.0
    }
}
