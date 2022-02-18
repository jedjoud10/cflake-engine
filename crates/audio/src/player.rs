use std::{io::Cursor, sync::Arc, cell::RefCell};
use rodio::{OutputStreamHandle, OutputStream, Decoder, SpatialSink};
use crate::{source::AudioSource, AudioSourceTracker};
// A playback cache that contains all the loaded sources
pub struct AudioPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    cache: Vec<Vec<u8>>,
    // Keep track of each sink
    sinks: RefCell<Vec<AudioSourceTracker>>,
    // Position of the left ear and right ear for positional sounds
    left: [f32; 3],
    right: [f32; 3],
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
    // Play a global sound
    pub fn play(&self, source: &AudioSource) -> Option<AudioSourceTracker> {
        // Check if the index is gud
        let idx = source.idx?;
        // Decode then play the sound        
        let compressed = self.cache.get(idx).unwrap().clone();
        let cursor = Cursor::new(compressed);
        let sink = self.stream_handle.play_once(cursor).unwrap();

        // Arc
        let tracker = AudioSourceTracker::Global(Arc::new(sink));
        let mut sinks = self.sinks.borrow_mut();
        sinks.push(tracker.clone());
        Some(tracker)
    }
    // Play a sound at a specified position
    pub fn play_positional(&self, source: &AudioSource, position: veclib::Vector3<f32>) -> Option<AudioSourceTracker> {
        // Check if the index is gud
        let idx = source.idx?;
        // Decode then play the sound        
        let compressed = self.cache.get(idx).unwrap().clone();
        let cursor = Cursor::new(compressed);
        // Create a new spatial sink
        // Convert positions        
        let pos = [position.x, position.y, position.z];
        let spatial_sink = SpatialSink::try_new(&self.stream_handle, pos, self.left, self.right).ok()?;
        // Decode and play
        let decoded = Decoder::new(cursor).unwrap();
        spatial_sink.append(decoded);
        spatial_sink.play();
        
        // Arc
        let tracker = AudioSourceTracker::Spatial(Arc::new(spatial_sink));
        let mut sinks = self.sinks.borrow_mut();
        sinks.push(tracker.clone());
        Some(tracker)
    }
    // Update the positions of the spatial ears
    pub fn update_ear_positions(&mut self, left: veclib::Vector3<f32>, right: veclib::Vector3<f32>) {
        let left = [left.x, left.y, left.z];
        let right = [right.x, right.y, right.z];
        self.left = left;
        self.right = right;
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