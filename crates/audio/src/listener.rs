use std::sync::{Mutex, atomic::{AtomicBool, Ordering}, Arc};

use rodio::{OutputStream, OutputStreamHandle};
use world::Resource;

pub(crate) static GLOBAL_LISTENER: Mutex<Option<OutputStreamHandle>> = Mutex::new(None);

// An audio listener component that will hear all of the audio source entities that are in the world
#[derive(Resource)]
pub struct Listener {
    active: bool,
    stream: OutputStream,
    handle: OutputStreamHandle,

    // Position of the left ear and right ear for positional sounds
    /*
    left: vek::Vec3<f32>,
    right: vek::Vec3<f32>,
    */
}

impl Listener {
    // Try to create a new listener and return Some. If there is already a new listener that is active, this will simply return None
    pub fn try_new() -> Option<Self> {
        let mut guard = GLOBAL_LISTENER.lock().unwrap();
        if let None = *guard {
            let (stream, handle) = OutputStream::try_default().unwrap();
            *guard = Some(handle.clone());
            Some(Self {
                active: true,
                stream,
                handle,
            })
        } else {
            None
        }
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        let mut guard = GLOBAL_LISTENER.lock().unwrap();
        *guard = None;
    }
}

/*
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
*/
