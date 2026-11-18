use super::{Amount, Currency};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money {
    amount: Amount,
    currency: Currency,
}

impl Money {
    #[must_use]
    pub const fn new(amount: Amount, currency: Currency) -> Self {
        Self { amount, currency }
    }

    #[must_use]
    pub const fn amount(&self) -> Amount {
        self.amount
    }

    #[must_use]
    pub const fn currency(&self) -> &Currency {
        &self.currency
    }
}
