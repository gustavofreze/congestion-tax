use super::{Amount, Passage, SingleChargeWindow};

#[derive(Clone, Copy)]
pub(super) struct ChargeWindow {
    length: SingleChargeWindow,
    highest: Amount,
    opened_at: Passage,
}

impl ChargeWindow {
    pub(super) const fn open(fee: Amount, length: SingleChargeWindow, opened_at: Passage) -> Self {
        Self {
            length,
            highest: fee,
            opened_at,
        }
    }

    pub(super) fn contains(self, passage: Passage) -> bool {
        self.length.contains(passage, self.opened_at)
    }

    pub(super) const fn charged_onto(self, charged: Amount) -> Amount {
        charged.plus(self.highest)
    }

    pub(super) fn raised_to(self, fee: Amount) -> Self {
        Self {
            highest: self.highest.max(fee),
            ..self
        }
    }
}
