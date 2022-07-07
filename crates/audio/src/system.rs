use ecs::{EcsManager, modified, added, or};
use math::Transform;
use world::{Events, World, Update, Stage};
use crate::{Listener, AudioSource, AudioHead, GLOBAL_LISTENER};

// This will update the audio listener ear positions and audio sources emitter positions
fn update(world: &mut World) {
    let ecs = world.get_mut::<&mut EcsManager>().unwrap();
    
    let head = ecs.try_view::<(&Transform, &Listener)>().unwrap().next().map(|(transform, _)| {
        AudioHead {
            left: -transform.right(),
            right: transform.right(),
        }
    });

    if let Some(new) = head {
        // Update ear locations
        let global = GLOBAL_LISTENER.lock().unwrap();
        let mut head = global.as_ref().unwrap().head.lock().unwrap();
        head.left = new.left;
        head.right = new.right;

        // Update emitter locations
        let filter = or(modified::<Transform>(), added::<Transform>());
        let sources = ecs.try_query_with::<(&mut AudioSource, &Transform)>(filter).unwrap();
        for (source, transform) in sources {
            if let Some(pos) = &source.position {
                *pos.lock().unwrap() = transform.position;
            }
        }
    }  
    
    
}

// Main audio system
pub fn system(events: &mut Events) {
    events.registry::<Update>().insert_with(update, Stage::new("audio update").after("post user")).unwrap();
}