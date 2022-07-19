use crate::{AudioClip, AudioHead, AudioSource, Listener, GLOBAL_LISTENER};
use ecs::{added, modified, or, EcsManager};
use math::Transform;
use world::{Events, Init, Stage, Storage, Update, World};

// This will insert the default audio clip storage
fn init(world: &mut World) {
    world.insert(Storage::<AudioClip>::default());
}

// This will update the audio listener ear positions and audio sources emitter positions
fn update(world: &mut World) {
    let mut ecs = world.get_mut::<EcsManager>().unwrap();

    // Get the audio listener's ear locations
    let head = ecs
        .view::<(&Transform, &Listener)>()
        .unwrap()
        .next()
        .map(|(transform, _)| AudioHead {
            left: -transform.right(),
            right: transform.right(),
        });

    if let Some(new) = head {
        // Update ear locations
        let global = GLOBAL_LISTENER.lock().unwrap();
        let mut head = global.as_ref().unwrap().head.lock().unwrap();
        head.left = new.left;
        head.right = new.right;

        // Update emitter locations
        let filter = or(modified::<Transform>(), added::<Transform>());
        let sources = ecs
            .query_with::<(&mut AudioSource, &Transform)>(filter)
            .unwrap();
        for (source, transform) in sources {
            if let Some(pos) = &source.position {
                *pos.lock().unwrap() = transform.position;
            }
        }
    }
}

// Main audio system
pub fn system(events: &mut Events) {
    events
        .registry::<Update>()
        .insert_with(update, Stage::new("audio update").after("post user"))
        .unwrap();

    events
        .registry::<Init>()
        .insert_with(init, Stage::new("insert storage audio").before("user"));
}
