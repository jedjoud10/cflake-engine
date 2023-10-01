use std::time::Duration;

/// Persistent system timings for systems that get called more than one time
/// This assumes that the samples are sorted before we calculate any of the following methods
#[derive(Clone, Copy)]
pub struct SystemTimings {
    samples: [Duration; 8],
}

impl SystemTimings {
    /// Get the median time
    pub fn median(&self) -> Duration {
        self.samples[3]
    }

    /// Get the average time of execution for this system
    pub fn average(&self) -> Duration {
        let nanos = self.samples.iter().map(|x| x.as_nanos()).sum::<u128>();
        Duration::from_nanos(nanos as u64 / 8)
    }

    /// Get the minimum time
    pub fn min(&self) -> Duration {
        self.samples[0]
    }

    /// Get the maximum time
    pub fn max(&self) -> Duration {
        self.samples[7]
    }
}