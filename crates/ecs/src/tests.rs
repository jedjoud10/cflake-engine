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
        for i in 0..u16::MAX {
            manager.insert(|_entity, modif| {
                modif.insert(Tag("Hello world tag!")).unwrap();
                modif.insert(SimpleValue(i as i32)).unwrap();
            });
        }

        for _i in 0..u16::MAX {
            manager.insert(|_entity, modif| {
                modif.insert(Tag("Hello world tag!")).unwrap();
                //modif.insert(SimpleValue(i as i32)).unwrap();
            });
        }

        for i in 0..u16::MAX {
            manager.insert(|_entity, modif| {
                modif.insert(Tag("Hello world tag!")).unwrap();
                modif.insert(SimpleValue(i as i32)).unwrap();
                modif.insert(Name("Sususus amogus?", [0; 64])).unwrap();
            });
        }

        for _ in 0..5 {
            manager.prepare();
            let i = std::time::Instant::now();
            type Layout<'a> = (&'a Tag, &'a Entity);
            let filter = or(added::<Tag>(), modified::<Tag>());
            let mut query = manager.query::<Layout>();
            if let Some((tag, entity)) = query.next() {
                println!("{:?}", *entity);
                println!("{:?}", tag);
            }
            dbg!(i.elapsed());
            //dbg!(query);
        }
    }
}
