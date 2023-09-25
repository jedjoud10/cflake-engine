use std::{marker::PhantomData, time::Duration};

use crate::{Caller, StageId};

// Persistent event timings for events that get called more than one time
pub struct PersistentEventTimings<C: Caller> {
    samples: [Duration; 8],
    min: Duration,
    max: Duration,
    _phantom: PhantomData<C>,
}

impl<C: Caller> Clone for PersistentEventTimings<C> {
    fn clone(&self) -> Self {
        Self {
            samples: self.samples,
            min: self.min,
            max: self.max,
            _phantom: self._phantom,
        }
    }
}

impl<C: Caller> PersistentEventTimings<C> {
    // Get the median time
    pub fn median(&self) -> Duration {
        todo!()
    }

    // Get the average time
    pub fn average(&self) -> Duration {
        let nanos = self.samples.iter().map(|x| x.as_nanos()).sum::<u128>();
        Duration::from_nanos(nanos as u64 / 8)
    }

    // Get the minimum time
    pub fn min(&self) -> Duration {
        self.min
    }

    // Get the maximum time
    pub fn max(&self) -> Duration {
        self.max
    }
}

// Timings for a single event of a system registry that has executed
pub struct EventTimings<C: Caller> {
    id: StageId,
    elapsed: Duration,

    // Only for events that get called multiple times
    persistent: Option<PersistentEventTimings<C>>,
}

impl<C: Caller> Clone for EventTimings<C> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            elapsed: self.elapsed,
            persistent: self.persistent.clone(),
        }
    }
}

impl<C: Caller> EventTimings<C> {
    // Create a new default event timings with a specific ID
    pub(crate) fn new(id: StageId, persistent: bool) -> Self {
        Self {
            id,
            elapsed: Duration::ZERO,
            persistent: persistent.then(|| PersistentEventTimings {
                samples: [Duration::ZERO; 8],
                min: Duration::ZERO,
                max: Duration::ZERO,
                _phantom: Default::default(),
            }),
        }
    }

    // Get the stage ID of the recorded event
    pub fn id(&self) -> StageId {
        self.id
    }

    // Get the time it took to execute the event
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    // Get persistent timing data
    pub fn persistent(&self) -> Option<&PersistentEventTimings<C>> {
        self.persistent.as_ref()
    }

    // Record a new timing for this event
    pub(crate) fn record(&mut self, timing: Duration) {
        self.elapsed = timing;
        if let Some(persistent) = self.persistent.as_mut() {
            persistent.samples.rotate_right(1);
            persistent.samples[0] = timing;
            persistent.max = persistent.max.max(timing);
            persistent.min = persistent.min.min(timing);
        }
    }
}
