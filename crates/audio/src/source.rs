use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{AudioClip, GLOBAL_LISTENER};
use ecs::Component;
use rodio::{source::Spatial, Sink, Source};
use world::{Handle, Storage};

// This component will be attached to entities that can play specific audio clips
#[derive(Component)]
pub struct AudioSource {
    // Main audio clip handle
    clip: Handle<AudioClip>,

    // Playback parameters
    speed: f32,
    paused: bool,
    volume: f32,

    // Inner resources
    playing: Option<Sink>,
    pub(crate) position: Option<Arc<Mutex<vek::Vec3<f32>>>>,
}

impl AudioSource {
    // Create a new positional audio source.
    pub fn positional(clip: Handle<AudioClip>, position: vek::Vec3<f32>) -> Self {
        Self {
            clip,
            speed: 1.0,
            paused: false,
            volume: 1.0,
            playing: None,
            position: Some(Arc::new(Mutex::new(position))),
        }
    }

    // Create a new global audio source
    pub fn global(clip: Handle<AudioClip>) -> Self {
        Self {
            clip,
            speed: 1.0,
            paused: false,
            volume: 1.0,
            playing: None,
            position: None,
        }
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
    pub fn sink(&self) -> Option<&Sink> {
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
        let shared = (*guard).as_ref()?;
        let clip = clips.get(&self.clip);
        let data = clip.0.clone();

        // Create a new sink and play the sound
        let sink = Sink::try_new(&shared.handle).unwrap();

        // Insert the data in the sink
        if let Some(emitter) = self.position.as_ref() {
            // 3D audio source, must create the Spatial modifier
            let emitter = emitter.clone();
            let head = shared.head.clone();
            let emitter_guard = *emitter.lock().unwrap();
            let head_guard = *head.lock().unwrap();

            // Create the source modifier
            let source = Spatial::new(
                data,
                emitter_guard.into_array(),
                head_guard.left.into_array(),
                head_guard.right.into_array(),
            )
            .periodic_access(Duration::from_millis(10), move |i| {
                let emitter_guard = emitter.lock().unwrap();
                let head_guard = head.lock().unwrap();
                i.set_positions(
                    emitter_guard.into_array(),
                    head_guard.left.into_array(),
                    head_guard.right.into_array(),
                );
            });

            sink.append(source);
        } else {
            sink.append(data);
        }

        // Set playback params
        sink.set_volume(self.volume);
        sink.set_speed(self.speed);
        self.playing = Some(sink);
        self.paused = false;

        Some(())
    }
}
