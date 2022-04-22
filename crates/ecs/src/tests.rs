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

        // Le entities
        /*
        for i in 0..u16::MAX {
            manager.insert(|_entity, modif| {
                modif.insert(Tag("Hello world tag!")).unwrap();
                modif.insert(SimpleValue(i as i32)).unwrap();
            });
        }

        for _i in 0..u16::MAX {
            manager.insert(|_entity, modif| {
                modif.insert(Tag("Hello world tag!")).unwrap();
                modif.insert(SimpleValue(_i as i32)).unwrap();
            });
        }
        */

        for i in 0..1 {
            let _entity = manager.insert(|_entity, modif| {
                modif.insert(Tag("Hello world tag!")).unwrap();
                modif.insert(SimpleValue(i as i32)).unwrap();
                modif.insert(Name("Sususus amogus?", [0; 64])).unwrap();
            });

            //let mut entry = manager.entry(entity).unwrap();
            //let tag = entry.get_mut::<Tag>().unwrap();
            //tag.0 = "AMOGUS STACK OVERFLOW";
        }

        for _ in 0..5 {
            manager.prepare();
            let _i = std::time::Instant::now();
            type Layout<'a> = (&'a Tag, &'a SimpleValue);
            let filter = added::<Tag>();
            let query = manager.query_with::<Layout, _>(filter);
            dbg!(query.count());
            //dbg!(query);
        }
    }
}
