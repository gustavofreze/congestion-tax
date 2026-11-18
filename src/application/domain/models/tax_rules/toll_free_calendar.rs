use std::collections::{BTreeSet, HashSet};

use jiff::civil::{Date, Weekday};

use super::Month;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TollFreeCalendar {
    months: BTreeSet<Month>,
    holidays: BTreeSet<Date>,
    weekdays: HashSet<Weekday>,
    day_before_holiday: bool,
}

impl TollFreeCalendar {
    #[must_use]
    pub fn new(
        day_before_holiday: bool,
        months: impl IntoIterator<Item = Month>,
        holidays: impl IntoIterator<Item = Date>,
        weekdays: impl IntoIterator<Item = Weekday>,
    ) -> Self {
        Self {
            months: months.into_iter().collect(),
            holidays: holidays.into_iter().collect(),
            weekdays: weekdays.into_iter().collect(),
            day_before_holiday,
        }
    }

    pub(super) fn is_toll_free(&self, date: Date) -> bool {
        self.weekdays.contains(&date.weekday())
            || self.months.contains(&Month::of(date))
            || self.holidays.contains(&date)
            || (self.day_before_holiday && date.tomorrow().is_ok_and(|next_day| self.holidays.contains(&next_day)))
    }
}
