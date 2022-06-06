use crate::{source::AudioSource, AudioSourceTracker};
use rodio::{
    source::Buffered, Decoder, OutputStream, OutputStreamHandle, Sink, Source, SpatialSink,
};
use std::{cell::RefCell, fmt::Debug, io::Cursor, sync::Arc};
// A playback cache that contains all the loaded sources
pub struct AudioPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    // Keep track of each sink
    sinks: RefCell<Vec<AudioSourceTracker>>,
    // Position of the left ear and right ear for positional sounds
    left: vek::Vec3<f32>,
    right: vek::Vec3<f32>,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        // Get the stream handle
        let (_stream, handle) = OutputStream::try_default().unwrap();
        Self {
            _stream,
            stream_handle: handle,
            sinks: Default::default(),
            left: Default::default(),
            right: Default::default(),
        }
    }
}

impl AudioPlayer {
    // Play a global audio source with modifier
    pub fn play<T: Source + Send + 'static>(
        &self,
        source: &AudioSource,
        map: impl FnOnce(Buffered<Decoder<Cursor<Vec<u8>>>>) -> T + Send,
    ) -> Option<AudioSourceTracker>
    where
        <T as Iterator>::Item: rodio::Sample + Send,
    {
        let buffered = map(source.buffered.clone());
        let sink = Sink::try_new(&self.stream_handle).unwrap();
        // Run the modifiers
        sink.append(buffered);
        // Arc
        let tracker = AudioSourceTracker::Global(Arc::new(sink));
        let mut sinks = self.sinks.borrow_mut();
        sinks.push(tracker.clone());
        Some(tracker)
    }
    // Play an audio source at a specified locatio, with modifier
    pub fn play_positional<T: Source + Send + 'static>(
        &self,
        source: &AudioSource,
        position: vek::Vec3<f32>,
        map: impl FnOnce(Buffered<Decoder<Cursor<Vec<u8>>>>) -> T + Send,
    ) -> Option<AudioSourceTracker>
    where
        <T as Iterator>::Item: rodio::Sample + Send + Debug,
    {
        let buffered = map(source.buffered.clone());
        // Create a new spatial sink
        // Convert positions
        let pos = [position.x, position.y, position.z];
        let left = self.left.as_slice().try_into().unwrap();
        let right = self.right.as_slice().try_into().unwrap();
        let spatial_sink = SpatialSink::try_new(&self.stream_handle, pos, left, right).ok()?;
        spatial_sink.append(buffered);
        // Arc
        let tracker = AudioSourceTracker::Spatial(Arc::new(spatial_sink));
        let mut sinks = self.sinks.borrow_mut();
        sinks.push(tracker.clone());
        Some(tracker)
    }
    // Update the positions of the spatial ears
    pub fn update(&mut self, left: vek::Vec3<f32>, right: vek::Vec3<f32>) {
        self.left = left;
        self.right = right;
        // Update each spatial sink now
        let mut borrowed = self.sinks.borrow_mut();
        for sink in borrowed.iter() {
            if let Some(spatial) = sink.as_spatial() {
                spatial.set_left_ear_position(left.as_slice().try_into().unwrap());
                spatial.set_right_ear_position(right.as_slice().try_into().unwrap());
            }
        }

        // Remove the sinks that finished playing
        borrowed.retain(|sink| match sink {
            AudioSourceTracker::Global(g) => !g.empty(),
            AudioSourceTracker::Spatial(s) => !s.empty(),
        });
    }
}
