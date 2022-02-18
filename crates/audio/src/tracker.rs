use std::sync::Arc;

use enum_as_inner::EnumAsInner;
use rodio::{Sink, SpatialSink};

// An audio source tracker that we can use to update an audio source that already started playing
#[derive(Clone, EnumAsInner)]
pub enum AudioSourceTracker {
    Global(Arc<Sink>),
    Spatial(Arc<SpatialSink>),
}