use std::time::Duration;

/// Number of samples to use for event timings
pub const EVENT_TIMINGS_SAMPLE_COUNT: usize = 8;

/// Persistent system timings for systems that get called more than one time
#[derive(Clone, Copy)]
pub struct SystemTimings {
    samples: [Duration; 8],
    min: Duration,
    max: Duration,
}

impl SystemTimings {
    // Get the median time
    pub fn median(&self) -> Duration {
        todo!()
    }

    /// Get the average time of execution for this system
    pub fn average(&self) -> Duration {
        let nanos = self.samples.iter().map(|x| x.as_nanos()).sum::<u128>();
        Duration::from_nanos(nanos as u64 / 8)
    }

    /// Get the minimum time
    pub fn min(&self) -> Duration {
        self.min
    }

    /// Get the maximum time
    pub fn max(&self) -> Duration {
        self.max
    }
}