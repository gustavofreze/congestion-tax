use jiff::civil::Date;

use super::Amount;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DailyTax {
    date: Date,
    amount: Amount,
}

impl DailyTax {
    #[must_use]
    pub const fn new(date: Date, amount: Amount) -> Self {
        Self { date, amount }
    }

    #[must_use]
    pub const fn date(self) -> Date {
        self.date
    }

    #[must_use]
    pub const fn amount(self) -> Amount {
        self.amount
    }
}
