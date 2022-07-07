use std::sync::Arc;

use ecs::Component;
use rodio::{Sink, SpatialSink};
use world::{Handle, Storage};
use crate::{AudioClip, GLOBAL_LISTENER};

// Audio sources can be either Global or Positional
enum SinkType {
    Global(Sink),
    Positional(SpatialSink),
}

// This component will be attached to entities that can play specific audio clips
#[derive(Component)]
pub struct AudioEmitter {
    clip: Handle<AudioClip>,
    speed: f32,
    paused: bool,
    volume: f32,
    playing: Option<SinkType>,
    position: vek::Vec3<f32>
}

impl AudioEmitter {
    // Create a new audio source. This will *not* play the audio clip
    pub fn new(clip: Handle<AudioClip>, position: vek::Vec3<f32>) -> Self {
        Self { clip, volume: 1.0, speed: 1.0, playing: None, paused: false, position }
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

    // Try to get the inner sink
    pub fn sink(&self) -> Option<&SpatialSink> {
        self.playing.as_ref()
    }

    // Set the paused state
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
        if let Some(sink) = &self.playing {
            if paused {
                sink.pause();
            } else {
                sink.play();
            }
        }
    }

    // Get the paused state
    pub fn paused(&self) -> bool {
        self.paused
    }

    // Get the clip volume
    pub fn volume(&self) -> f32 {
        self.volume
    }

    // Get the clip speed
    pub fn speed(&self) -> f32 {
        self.speed
    }

    // Try to play the inner audio clip (this will fail if we have no listener that is active)
    pub fn try_play(&mut self, clips: &Storage<AudioClip>) -> Option<()> {
        // Fetch the global listener and the clip data
        let guard = GLOBAL_LISTENER.lock().unwrap();
        let shared = (&*guard).as_ref()?;
        let clip = clips.get(&self.clip);
        let data = clip.0.clone();

        // Decompose the shared listener
        let handle = &shared.handle;
        let head = shared.head.lock().unwrap();
        let left = head.left;
        let right = head.right;

        // Create a new sink and play the sound
        let sink = SpatialSink::try_new(&handle, self.position.into_array(), left.into_array(), right.into_array()).unwrap();
        sink.append(data);
        sink.set_volume(self.volume);
        sink.set_speed(self.speed);         
        self.playing = Some(sink);
        self.paused = false;
        Some(())
    }
}