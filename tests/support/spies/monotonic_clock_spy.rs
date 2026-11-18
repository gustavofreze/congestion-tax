use std::sync::Mutex;
use std::time::{Duration, Instant};

use congestion_tax::driven::cache::MonotonicClock;

pub(crate) struct MonotonicClockSpy {
    now: Mutex<Instant>,
}

impl MonotonicClockSpy {
    pub(crate) fn starting_now() -> Self {
        Self {
            now: Mutex::new(Instant::now()),
        }
    }

    pub(crate) fn advance(&self, elapsed: Duration) {
        let mut now = self.now.lock().expect("the clock is never poisoned");
        *now += elapsed;
    }
}

impl MonotonicClock for &MonotonicClockSpy {
    fn now(&self) -> Instant {
        *self.now.lock().expect("the clock is never poisoned")
    }
}
