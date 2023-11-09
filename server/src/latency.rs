use std::{fmt, time::Duration};

pub(crate) struct Latency {
    duration: Duration,
}

impl Latency {
    pub(crate) fn new(duration: Duration) -> Self {
        Self { duration }
    }
}

impl fmt::Display for Latency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.duration.as_micros() < 1000 {
            return write!(f, "{} μs", self.duration.as_micros());
        }

        if self.duration.as_millis() < 1000 {
            return write!(
                f,
                "{}.{} ms",
                self.duration.as_millis(),
                self.duration.subsec_micros() % 1000
            );
        }

        write!(
            f,
            "{}.{} s",
            self.duration.as_secs(),
            self.duration.subsec_millis()
        )
    }
}
