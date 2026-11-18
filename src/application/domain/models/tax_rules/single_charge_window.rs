use jiff::SignedDuration;

use super::Passage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SingleChargeWindow(SignedDuration);

impl SingleChargeWindow {
    #[must_use]
    pub fn of_minutes(minutes: u16) -> Self {
        Self(SignedDuration::from_mins(i64::from(minutes)))
    }

    pub(super) fn contains(self, passage: Passage, opened_at: Passage) -> bool {
        passage.elapsed_since(opened_at) <= self.0
    }
}
