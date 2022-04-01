#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use rayon::iter::{IndexedParallelIterator, ParallelIterator};

    use crate::*;

    #[test]
    fn test() {
        // Empty manager
        let mut manager = EcsManager::default();

        // Simple component
        #[derive(Component, Debug)]
        struct Name(&'static str);
        registry::register::<Name>();

        #[derive(Component, Debug)]
        struct Tag(&'static str);
        registry::register::<Tag>();

        #[derive(Component, Debug)]
        struct SimpleValue(usize);
        registry::register::<SimpleValue>();


        let entity = manager.insert_with(|_, linker| {
            linker.insert(Name("Le Jribi")).unwrap();
            linker.insert(Tag("Jed est cool (trust)")).unwrap();
            linker.insert(SimpleValue(0)).unwrap();
        });

        manager.modify(entity, |_, modifier| {
            modifier.remove::<Name>().unwrap();
        });

        /*
        // Make a new entity
        const COUNT: usize = 10;
        let entity = Entity::default();
        for x in 0..COUNT {
            let _ = manager.insert_with(|_, modifs| {
                modifs.insert(Name("Le Jribi")).unwrap();
                modifs.insert(Tag("Jed est cool (trust)")).unwrap();
                modifs.insert(SimpleValue(x)).unwrap();
            });
        }

        // Create a mask layout
        let mask = crate::layout!(Name, Tag, SimpleValue);

        // Query
        let i = std::time::Instant::now();
        dbg!(size_of::<QueryBuilder>());
        dbg!(size_of::<EntityEntry>());
        dbg!(size_of::<EcsManager>());

        while i.elapsed().as_secs() < 5 {
            manager.prepare();
            let _h = std::time::Instant::now();
            let _entry = EntityEntry::new(&mut manager, entity);
            //dbg!(entry.get::<Tag>().unwrap().0);
            //dbg!(entry.state());=
            let builder = QueryBuilder::new(&mut manager, mask);
            let values = builder.get_mut::<SimpleValue>().unwrap();
            let tags = builder.get::<(Tag, ComponentState)>().unwrap();
            let names = builder.get::<Name>().unwrap();
            values.into_iter().zip(tags.into_iter()).zip(names.into_iter()).for_each(|((value, _tag), _name)| {
                println!("{}", value.0);
            });

            //panic!("remove");
            manager.remove(entity);

            /*vec.par_iter().for_each(|value| {
                let x = value.0;
                let y = value.0 + 6;
                value.0 += 2 - y;
            });
            */
            /*
            vec.par_iter_mut().for_each(|value| {
                let x = value.0;
                let y = value.0 + 6;
                value.0 += 2 - y;
            });
            */
            /*
            vec2.par_iter_mut().for_each(|linked| {
                //let name = linked.get::<SimpleValue>().unwrap();
                let val = &mut linked.0;
                *val += 1;
                //linked.get_component_bits::<Name>().unwrap();
                //dbg!(name.0);
                //dbg!(linked.was_mutated::<Name>().unwrap());
            });
            vec2.par_iter_mut().for_each(|linked| {
                //let name = linked.get::<SimpleValue>().unwrap();
                let val = &mut linked.0;
                *val += 1;
                *val *= 3;
                //linked.get_component_bits::<Name>().unwrap();
                //dbg!(name.0);
                //dbg!(linked.was_mutated::<Name>().unwrap());
            });
            */
            //dbg!(h.elapsed().as_micros());
        }
        */
    }
}
