use world::{
    ecs::{
        component::{ComponentKey, RefComponentFetcher},
        event::EventKey,
    },
    rendering::{
        basics::lights::{StoredLight, StoredLightTransform},
        pipeline::{RenderedModel, RenderingCamera, RenderingSettings, ShadowedModel},
    },
    World,
};

// The lights system update loop
fn run(world: &mut World, mut data: EventKey) {
    // Update all the light sources
    let query = data.as_query().unwrap();
    for (_, components) in query.iter() {
        let light = components.get::<crate::components::Light>().unwrap();
        let transform = components.get::<crate::components::Transform>().unwrap();
        let source = world.pipeline.lights.get_mut(&light.handle).unwrap();
        source._type = light._type.clone();
        source.transform = StoredLightTransform {
            position: transform.position,
            rotation: transform.rotation,
        };
        source.strength = light.strength;
        source.color = light.color;
    }
}

// An event fired whenever we add multiple new light sources
fn added_entities(world: &mut World, mut data: EventKey) {
    let query = data.as_query_mut().unwrap();
    for (_, components) in query.iter_mut() {
        let light = components.get::<crate::components::Light>().unwrap();
        let transform = components.get::<crate::components::Transform>().unwrap();

        // Add the light source to the pipeline
        let handle = world.pipeline.lights.insert(StoredLight {
            _type: light._type.clone(),
            transform: StoredLightTransform {
                position: transform.position,
                rotation: transform.rotation,
            },
            strength: light.strength,
            color: light.color,
        });

        // Update the component
        let light = components.get_mut::<crate::components::Light>().unwrap();
        light.handle = handle;
    }
}

// An event fired whenever we remove multiple light sources
fn removed_entities(world: &mut World, mut data: EventKey) {
    // The light would automatically get removed from the pipeline since the Handle<StoredLight> would get dropped
}

// Create the light system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder()
        .with_run_event(run)
        .with_added_entities_event(added_entities)
        .with_removed_entities_event(removed_entities)
        .link::<crate::components::Light>()
        .link::<crate::components::Transform>()
        .build();
}
