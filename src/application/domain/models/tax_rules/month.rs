use jiff::civil::Date;

use crate::application::domain::errors::UnsupportedMonth;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Month(i8);

impl Month {
    pub(super) fn of(date: Date) -> Self {
        Self(date.month())
    }

    pub fn new(number: i8) -> Result<Self, UnsupportedMonth> {
        if !(1..=12).contains(&number) {
            return Err(UnsupportedMonth::new(number));
        }

        Ok(Self(number))
    }
}
