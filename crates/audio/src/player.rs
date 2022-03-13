use crate::{source::AudioSource, AudioSourceTracker};
use rodio::{source::Buffered, Decoder, OutputStream, OutputStreamHandle, Sink};
use std::{cell::RefCell, io::Cursor, sync::Arc};
// A playback cache that contains all the loaded sources
pub struct AudioPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    cache: Vec<Buffered<Decoder<Cursor<Vec<u8>>>>>,
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
            cache: Default::default(),
            sinks: Default::default(),
            left: Default::default(),
            right: Default::default(),
        }
    }
}

impl AudioPlayer {
    // Play a global audio source
    pub fn play(&self, source: &AudioSource) -> Option<AudioSourceTracker> {
        // Check if the index is gud
        let idx = source.idx?;
        let buffered = self.cache.get(idx).unwrap().clone();
        let sink = Sink::try_new(&self.stream_handle).unwrap();
        sink.append(buffered);
        // Arc
        let tracker = AudioSourceTracker::Global(Arc::new(sink));
        let mut sinks = self.sinks.borrow_mut();
        sinks.push(tracker.clone());
        Some(tracker)
    }
    /*
    // Play an audio source at a specified position
    pub fn play_positional(&self, source: &AudioSource, position: vek::Vec3<f32>) -> Option<AudioSourceTracker> {
        // Check if the index is gud
        let idx = source.idx?;
        // Decode then play the sound
        let buffered = self.cache.get(idx).unwrap().clone();
        // Create a new spatial sink
        // Convert positions
        let pos = [position.x, position.y, position.z];
        let spatial_sink = SpatialSink::try_new(&self.stream_handle, pos, self.left, self.right).ok()?;
        spatial_sink.append(buffered);
        // Arc
        let tracker = AudioSourceTracker::Spatial(Arc::new(spatial_sink));
        let mut sinks = self.sinks.borrow_mut();
        sinks.push(tracker.clone());
        Some(tracker)
    }
    // Play a global audio source with modifier
    pub fn play_with_modifiers<T: Source + Send + 'static>(
        &self,
        source: &AudioSource,
        function: impl FnOnce(Buffered<Decoder<Cursor<Vec<u8>>>>) -> T + Send,
    ) -> Option<AudioSourceTracker>
    where
        <T as Iterator>::Item: rodio::Sample + Send,
    {
        // Check if the index is gud
        let idx = source.idx?;
        let buffered = self.cache.get(idx).unwrap().clone();
        let buffered = function(buffered);
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
    pub fn play_positional_with_modifiers<T: Source + Send + 'static>(
        &self,
        source: &AudioSource,
        position: vek::Vec3<f32>,
        function: impl FnOnce(Buffered<Decoder<Cursor<Vec<u8>>>>) -> T + Send,
    ) -> Option<AudioSourceTracker>
    where
        <T as Iterator>::Item: rodio::Sample + Send + Debug,
    {
        // Check if the index is gud
        let idx = source.idx?;
        // Decode then play the sound
        let buffered = self.cache.get(idx).unwrap().clone();
        let buffered = function(buffered);
        // Create a new spatial sink
        // Convert positions
        let pos = [position.x, position.y, position.z];
        let spatial_sink = SpatialSink::try_new(&self.stream_handle, pos, self.left, self.right).ok()?;
        spatial_sink.append(buffered);
        // Arc
        let tracker = AudioSourceTracker::Spatial(Arc::new(spatial_sink));
        let mut sinks = self.sinks.borrow_mut();
        sinks.push(tracker.clone());
        Some(tracker)
    }
    */
    // Update the positions of the spatial ears
    pub fn update_ear_positions(&mut self, left: vek::Vec3<f32>, right: vek::Vec3<f32>) {
        self.left = left;
        self.right = right;
        // Update each spatial sink now
        let borrowed = self.sinks.borrow();
        for sink in borrowed.iter() {
            if let Some(spatial) = sink.as_spatial() {
                spatial.set_left_ear_position(left.as_slice().try_into().unwrap());
                spatial.set_right_ear_position(right.as_slice().try_into().unwrap());
            }
        }
    }
    // Cache a sound to the playback cache (not really; We are just stealing it's temporary bytes)
    pub fn cache(&mut self, mut source: AudioSource) -> Option<AudioSource> {
        // Steal
        source.idx = Some(self.cache.len());
        let compressed = source.temp.take()?;
        self.cache.push(compressed);
        Some(source)
    }
}
