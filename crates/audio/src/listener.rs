use ecs::Component;
use math::{Location, Rotation};
use rodio::{OutputStream, OutputStreamHandle};
use std::sync::{Arc, Mutex};

// TODO: Rewrite all of this with our own custom logic sheize
#[derive(Clone, Copy)]
// The global audio head of the main audio listener
pub(crate) struct AudioHead {
    pub(crate) left: vek::Vec3<f32>,
    pub(crate) right: vek::Vec3<f32>,
}

// Shared listener that is available to all audio sources in the world
pub(crate) struct SharedListener {
    pub(crate) handle: OutputStreamHandle,
    pub(crate) head: Arc<Mutex<AudioHead>>,
}

pub(crate) static GLOBAL_LISTENER: Mutex<Option<SharedListener>> = Mutex::new(None);

// An audio listener component that will hear all of the audio source entities that are in the world
#[derive(Component)]
pub struct Listener {
    stream: OutputStream,
}

impl Listener {
    // Try to create a new listener and return Some. If there is already a new listener that is active, this will simply return None
    pub fn try_new(location: &Location, rotation: &Rotation) -> Option<Self> {
        let mut guard = GLOBAL_LISTENER.lock().unwrap();
        if let None = *guard {
            let (stream, handle) = OutputStream::try_default().unwrap();
            *guard = Some(SharedListener {
                handle,
                head: Arc::new(Mutex::new(AudioHead {
                    left: **location - rotation.right(),
                    right: **location + rotation.right(),
                })),
            });
            Some(Self { stream })
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
