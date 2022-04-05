#[cfg(test)]
mod tests {
    use rayon::iter::ParallelIterator;

    use crate::*;

    use crate::*;
    #[test]
    fn test() {
        // Empty manager
        let mut manager = EcsManager::new();

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

        let entity = manager.insert(|_, linker| {
            linker.insert(Name("Le Jribi")).unwrap();
            linker.insert(Tag("Jed est cool (trust)")).unwrap();
            linker.insert(SimpleValue(0)).unwrap();
        });

        /*
        manager.modify(entity, |_, modifier| {
            modifier.remove::<Name>().unwrap();
            modifier.insert(Name("Trustrutrst")).unwrap();
            modifier.insert(SimpleValue(0)).unwrap();
            modifier.remove::<SimpleValue>().unwrap();
        });
        */

        /*
        dbg!(entry.get::<Name>().unwrap());
        dbg!(entry.get::<Tag>().unwrap());
        dbg!(entry.get::<SimpleValue>());
        */
        let mut entry = manager.entry(entity).unwrap();
        let _name = entry.get_mut::<Name>().unwrap();
        // Get the query

        // Make a new entity
        const COUNT: usize = u16::MAX as usize * 1;
        for x in 0..COUNT {
            let _entity = manager.insert(|_, modifs| {
                modifs.insert(Name("Le Jribi")).unwrap();
                modifs.insert(Tag("Jed est cool (trust)")).unwrap();
                modifs.insert(SimpleValue(x)).unwrap();
            });
        }

        // Query
        let _i = std::time::Instant::now();

        for _ in 0..5 {
            manager.prepare();
            let h = std::time::Instant::now();
            //dbg!(entry.get::<Tag>().unwrap().0);
            //dbg!(entry.state());=

            let builder = Query::par_new::<(&Name, &mut SimpleValue, &Entity)>(&mut manager).unwrap();
            builder.for_each(|(_x, _, _entity)| {});
            /*
            for (name, val) in .unwrap() {
                //dbg!(name);
            }
            */
            //panic!("remove");

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
            dbg!(h.elapsed().as_micros());
        }
    }
}
