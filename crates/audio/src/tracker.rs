use std::sync::Arc;

use enum_as_inner::EnumAsInner;
use rodio::{Sink, SpatialSink};

// An audio source tracker that we can use to update an audio source that already started playing
#[derive(Clone, EnumAsInner)]
pub enum AudioSourceTracker {
    Global(Arc<Sink>),
    Spatial(Arc<SpatialSink>),
}

impl AudioSourceTracker {
    // Update the position of the tracker if it is a spatial tracker
    pub fn update_position(&self, pos: vek::Vec3<f32>) {
        if let Self::Spatial(spatial) = self {
            spatial.set_emitter_position(pos.as_slice().try_into().unwrap())
        }
    }
}
