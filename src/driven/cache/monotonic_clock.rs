use std::time::Instant;

/// Clock whose instants never go backwards.
pub trait MonotonicClock: Send + Sync {
    /// Returns the current instant.
    fn now(&self) -> Instant;
}
