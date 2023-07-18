use atomic_float::AtomicF32;
use cpal::traits::{DeviceTrait, HostTrait};
use ecs::Component;
use parking_lot::RwLock;
use std::sync::{atomic::Ordering, Arc};

// This is a component that will be able to playback any type of audio to a specific cpal device
// We can technically have multiple audio listenenrs in the same scene, although that would be pretty pointless
#[derive(Component)]
pub struct AudioListener {
    // CPAL stuff
    pub device: cpal::Device,
    pub host: cpal::Host,
    pub supported_output_configs: Vec<cpal::SupportedStreamConfigRange>,

    // Global audio listener volume
    pub volume: Arc<AtomicF32>,

    // Distance between the ears of the listener in meters
    pub ear_distance: f32,

    // Ear positions
    pub(crate) ear_positions: [Arc<RwLock<vek::Vec3<f32>>>; 2],
}

impl AudioListener {
    // Create an audio player that uses the default host device
    pub fn new(ear_distance: f32) -> Option<Self> {
        // Fetch the CPAL device
        let host = cpal::default_host();
        let device = host.default_output_device()?;
        log::debug!(
            "Using device {:?} as the default device for the audio player",
            device.name().unwrap()
        );

        // Fetch the cpal stream config and save them in a vec
        let supported_output_configs = device.supported_output_configs().ok()?.collect::<Vec<_>>();

        // Can't have shit in Ohio
        if supported_output_configs.is_empty() {
            panic!("No supported output configs!");
        }

        for x in supported_output_configs.iter() {
            log::debug!(
                "Min sample rate: {}, max sample rate: {}",
                x.min_sample_rate().0,
                x.max_sample_rate().0
            );
        }

        Some(Self {
            host,
            device,
            volume: Arc::new(AtomicF32::new(1.0)),
            supported_output_configs,
            ear_distance,
            ear_positions: std::array::from_fn(|_| Arc::new(RwLock::new(vek::Vec3::zero()))),
        })
    }

    // Set the volume of the audio player as a percentage
    pub fn set_volume(&mut self, volume: f32) {
        self.volume.store(volume, Ordering::Relaxed);
    }

    // Get the volume of the audio player
    pub fn volume(&self) -> f32 {
        self.volume.load(Ordering::Relaxed)
    }
}
