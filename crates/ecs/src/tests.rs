#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test() {
        // Empty manager
        let mut manager = EcsManager::default();

        // Simple component
        #[derive(Component, Debug)]
        struct Name(&'static str, [i32; 64]);

        #[derive(Component, Debug)]
        struct Tag(&'static str);
        #[derive(Component, Debug)]
        struct SimpleValue(i32);

        /*
        let entity = manager.insert(|_, linker| {
            linker.insert(Name("Le Jribi", [0; 64])).unwrap();
            linker.insert(Tag("Jed est cool (trust)")).unwrap();
            linker.insert(SimpleValue(0)).unwrap();
        });
        */

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
        //let mut entry = manager.entry(entity).unwrap();
        //let _name = entry.get_mut::<Name>().unwrap();
        // Get the query
        for x in 0..500 {
            let i = std::time::Instant::now();
            registry::mask::<SimpleValue>();
            dbg!(i.elapsed());
        }

        // Make a new entity
        const COUNT: usize = u16::MAX as usize;
        let i = std::time::Instant::now();
        for x in 0..COUNT {
            let _entity = manager.insert(|_, linker| {
                //linker.insert(Name("Le Jribi", [1; 64])).unwrap();
                linker.insert(Tag("Jed est cool (trust)")).unwrap();
                linker.insert(SimpleValue((x) as i32)).unwrap();
            });
        }
        dbg!(i.elapsed());

        let entity = manager.insert(|_, linker| {
            //linker.insert(Name("Le Jribi", [1; 64])).unwrap();
            linker.insert(Tag("Jed est cool (trust)")).unwrap();
            linker.insert(SimpleValue((0) as i32)).unwrap();
        });

        for x in 0..10 {
            let i = std::time::Instant::now();
            manager.query::<(Write<SimpleValue, true>)>();
            dbg!(i.elapsed());
        }

        let i = std::time::Instant::now();
        manager.query::<(Write<SimpleValue, true>)>();
        dbg!(i.elapsed());

        manager
            .modify(entity, |e, modif| {
                modif.remove::<Tag>().unwrap();
                modif.remove::<SimpleValue>().unwrap();
                modif.insert(Name("Le Jribi", [1; 64])).unwrap();
            })
            .unwrap();

        dbg!(manager.query::<(Write<SimpleValue, true>)>().count());
        dbg!(manager.query::<Read<Name>>().count());

        /*
        for x in 0..COUNT {
            let _entity = manager.insert(|_, modifs| {
                //modifs.insert(Name("Le Jribi", [1; 64])).unwrap();
                //modifs.insert(Tag("Jed est cool (trust)")).unwrap();
                modifs.insert(SimpleValue((x) as i32)).unwrap();
            });
        }
        for x in 0..COUNT {
            let _entity = manager.insert(|_, modifs| {
                modifs.insert(Name("Le Jribi", [1; 64])).unwrap();
                //modifs.insert(Tag("Jed est cool (trust)")).unwrap();
                modifs.insert(SimpleValue((x) as i32)).unwrap();
            });
        }
        */

        // Query
        //let mut query = Query::new::<(&Name, &mut SimpleValue)>(&manager).unwrap().collect::<Vec<_>>();
        for _ in 0..5 {
            manager.prepare();
            let i = std::time::Instant::now();
            //let filters = changed::<SimpleValue>() | state(EntityState::Added);
            /*
            for (value, tag) in manager.query::<(Write<SimpleValue, true>, Read<Tag>)>().unwrap() {
                value.0 += 3;
            }
            */
            eprintln!("{:?}", i.elapsed());
        }
    }
}
