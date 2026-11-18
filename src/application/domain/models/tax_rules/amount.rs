#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount(u32);

impl Amount {
    pub const ZERO: Self = Self(0);

    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }

    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub const fn plus(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }
}
