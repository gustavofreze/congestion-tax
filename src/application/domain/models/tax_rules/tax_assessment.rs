use super::{Amount, Currency, DailyTax, Money};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxAssessment {
    days: Vec<DailyTax>,
    currency: Currency,
}

impl TaxAssessment {
    #[must_use]
    pub const fn new(days: Vec<DailyTax>, currency: Currency) -> Self {
        Self { days, currency }
    }

    #[must_use]
    pub fn days(&self) -> &[DailyTax] {
        &self.days
    }

    #[must_use]
    pub fn total(&self) -> Money {
        let total = self
            .days
            .iter()
            .map(|day| day.amount())
            .fold(Amount::ZERO, Amount::plus);

        Money::new(total, self.currency.clone())
    }
}
