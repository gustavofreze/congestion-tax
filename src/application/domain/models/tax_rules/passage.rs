use std::str::FromStr;

use jiff::SignedDuration;
use jiff::civil::{Date, DateTime, Time};

use crate::application::domain::errors::PassageFormatNotValid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Passage(DateTime);

impl Passage {
    pub(super) fn date(self) -> Date {
        self.0.date()
    }

    pub(super) fn time(self) -> Time {
        self.0.time()
    }

    pub(super) fn elapsed_since(self, earlier: Self) -> SignedDuration {
        self.0.duration_since(earlier.0)
    }
}

impl FromStr for Passage {
    type Err = PassageFormatNotValid;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        raw.parse::<DateTime>()
            .map(Self)
            .map_err(|_| PassageFormatNotValid::new(raw))
    }
}
