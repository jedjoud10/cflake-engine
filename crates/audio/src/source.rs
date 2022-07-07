use rodio::Sink;
use world::{Handle, Storage};
use crate::{AudioClip, GLOBAL_LISTENER};

// This component will be attached to entities that can play specific audio clips
pub struct AudioSource {
    clip: Handle<AudioClip>,
    speed: f32,
    volume: f32,
    playing: Option<Sink>,
}

impl AudioSource {
    // Create a new audio source. This will *not* play the audio clip
    pub fn new(clip: Handle<AudioClip>) -> Self {
        Self { clip, volume: 1.0, speed: 1.0, playing: None }
    }

    // Set the master volume
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        if let Some(sink) = &self.playing {
            sink.set_volume(volume);
        }
    }

    // Set the clip speed
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
        if let Some(sink) = &self.playing {
            sink.set_speed(speed);
        }
    }

    // Try to play the inner audio clip (this will fail if we have no listener that is active)
    pub fn try_play(&mut self, clips: &Storage<AudioClip>) -> Option<()> {
        // Fetch the stream handle and the clip data
        let guard = GLOBAL_LISTENER.lock().unwrap();
        let handle = (&*guard).as_ref()?;
        let clip = clips.get(&self.clip);
        let data = clip.0.clone();

        // Create a new sink and play the sound
        let sink = Sink::try_new(handle).unwrap();
        sink.append(data);
        sink.set_volume(self.volume);
        sink.set_speed(self.speed);
        self.playing = Some(sink);
        Some(())
    }
}