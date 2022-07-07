use ecs::EcsManager;
use math::Transform;
use world::{Events, World, Update, Stage};
use crate::{Listener, AudioEmitter, AudioHead};

// This will update the audio listener ear positions and audio sources emitter positions
fn update(world: &mut World) {
    let ecs = world.get_mut::<&mut EcsManager>().unwrap();
    
    // Get the main audio listener
    let head = ecs.try_view::<(&Transform, &Listener)>().unwrap().next().map(|(transform, _)| {
        AudioHead {
            left: -transform.right(),
            right: transform.right(),
        }
    });

    // Update the audio listener ear positions
    if let Some(head) = head {
        let sources = ecs.try_query::<&mut AudioEmitter>().unwrap();
        for (source, transform) in sources {
            source
        }
    }

    // Automatically update the position of audio sources that contain the transform component
    if let Some(head) = head {
        let sources = ecs.try_query::<(&mut AudioEmitter, &Transform)>().unwrap();
        for (source, transform) in sources {
            let sink = source.sink().unwrap();
            sink.set_emitter_position(transform.position.into_array());
            sink.set_left_ear_position(head.left.into_array());
            sink.set_right_ear_position(head.right.into_array());
        }
    }  
    
    
}

fn system(events: &mut Events) {
    events.registry::<Update>().insert_with(update, Stage::new("audio update").after("user"))
}