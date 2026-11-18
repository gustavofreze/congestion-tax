use jiff::civil::Date;

use super::Passage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Passages(Vec<Passage>);

impl Passages {
    #[must_use]
    pub fn new(passages: impl IntoIterator<Item = Passage>) -> Self {
        let mut passages: Vec<Passage> = passages.into_iter().collect();
        passages.sort_unstable();

        Self(passages)
    }

    pub(super) fn by_day(&self) -> impl Iterator<Item = (Date, &[Passage])> {
        self.0
            .chunk_by(|current, next| current.date() == next.date())
            .filter_map(|day_passages| day_passages.first().map(|first| (first.date(), day_passages)))
    }
}

impl FromIterator<Passage> for Passages {
    fn from_iter<I: IntoIterator<Item = Passage>>(passages: I) -> Self {
        Self::new(passages)
    }
}
