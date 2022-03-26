#[cfg(test)]
mod tests {
    use std::{cell::RefCell, mem::size_of, rc::Rc};

    use rayon::iter::{
        IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
    };

    use crate::prelude::*;

    // Test
    #[test]
    fn test() {
        // Create a manager
        let mut manager = EcsManager::default();

        // Create a new archetype
        let bits = [f32::register()];
        let layout = ComponentLayout::new(&bits);
        let id = manager.register(layout);
        
        let bits2 = [f32::register(), i32::register()];
        let layout2 = ComponentLayout::new(&bits2);
        let id2 = manager.register(layout2);
        // Add an entity
        for _ in 0..(8196) {
            let entity = manager
                .insert_with(id, |s| {
                    //s.insert(0i32);
                    //s.insert(0u8);
                    //s.insert(0u64);
                    s.insert(5.0f32).unwrap();
                })
                .unwrap();
            let entity = manager
                .insert_with(id2, |s| {
                    s.insert(0i32).unwrap();
                    //s.insert(0u8);
                    //s.insert(0u64);
                    s.insert(5.0f32).unwrap();
                })
                .unwrap();

            let mut linked = manager.entry(entity, layout2);
            *linked.get_mut::<f32>().unwrap() += 5.0;
            *linked.get_mut::<i32>().unwrap() += 5;
        }

        rayon::ThreadPoolBuilder::new().num_threads(6).build_global().unwrap();

        // Loop through all the components of type i32
        for x in 0..100 {            
            let i = std::time::Instant::now();
            
            manager
                .query(layout)
                .into_iter()
                .for_each(|mut linked| {
                    //let y = linked.get_mut::<i32>().unwrap();
                    //*y += 1;
                });

            dbg!(i.elapsed().as_micros());
        }
    }

    // Execution test
    #[test]
    fn execution_test() {
        // World
        #[derive(Default)]
        struct World {
            manager: EcsManager,
            systems: SystemSet<Self>,
        }

        // Create an empty world
        let mut world = World::default();

        // Register f32
        f32::register();
        
        // Add a system (event)
        world.manager.system::<World>(|world| {
            println!("Hello world!");
            
            // Simple query
            let layout = ComponentLayout::new(&[f32::bits().unwrap()]);
        }, &mut world.systems);

        // Run the systems
        world.manager.prepare::<World>();
        let systems = world.systems.clone();
        EcsManager::execute(&mut world, systems)
    }
}
