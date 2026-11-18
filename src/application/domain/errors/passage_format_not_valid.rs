use thiserror::Error;

#[derive(Debug, Error)]
#[error("Passage <{raw}> is not a local date and time in the YYYY-MM-DD HH:MM:SS format.")]
pub struct PassageFormatNotValid {
    raw: String,
}

impl PassageFormatNotValid {
    pub(crate) fn new(raw: &str) -> Self {
        Self { raw: raw.to_owned() }
    }
}
