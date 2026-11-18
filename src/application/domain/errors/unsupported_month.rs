use thiserror::Error;

#[derive(Debug, Error)]
#[error("Month <{number}> is not a month of the year, which runs from 1 to 12.")]
pub struct UnsupportedMonth {
    number: i8,
}

impl UnsupportedMonth {
    pub(crate) const fn new(number: i8) -> Self {
        Self { number }
    }
}
