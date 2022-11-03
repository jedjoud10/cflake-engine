#[cfg(test)]
mod tests {
    use world::ThreadPool;

    use crate::*;

    #[derive(Component, Debug, PartialEq, Eq)]
    struct Name(&'static str);
    #[derive(Component, Debug, PartialEq, Eq)]
    struct Health(i32);
    #[derive(Component, Debug, Clone, Copy)]
    struct Ammo(u32);

    fn cleanup(ecs: &mut Scene) {
        for (_, archetype) in ecs.archetypes_mut() {
            for state in archetype.states_mut().iter_mut() {
                state.update(|added, removed, mutated| {
                    *added = Mask::zero();
                    *mutated = Mask::zero();
                    *removed = Mask::zero();
                });
            }
        }
    }

    #[test]
    fn entries() {
        let mut manager = Scene::default();

        let entity = manager.insert((Name("Basic"), Health(100)));
        let mut entry = manager.entry_mut(entity).unwrap();
        assert_eq!(entry.get_mut::<Name>(), Some(&mut Name("Basic")));
        assert!(entry.get_mut::<Ammo>().is_none());

        let mask = registry::mask::<Name>() | registry::mask::<Health>();
        let archetype = manager.archetypes().get(&mask).unwrap();
        let states = archetype.states();
        let state = states.get(0).unwrap();
        assert_eq!(state.mutated(), mask);

        let entry = manager.entry(entity).unwrap();
        assert_eq!(entry.get::<Name>(), Some(&Name("Basic")));
        assert!(entry.get::<Ammo>().is_none());
    }

    #[test]
    fn queries() {
        let mut manager = Scene::default();
        let iter = (0..128).map(|_| (Name("Person"), Health(100)));
        manager.extend_from_iter(iter);

        let query = manager.query_mut::<(&Name, &mut Health)>();
        for (name, health) in query {
            assert_eq!(name.0, "Person");
            assert_eq!(health.0, 100);
            health.0 -= 100;
        }

        let query = manager.query_mut::<(&Name, &mut Health)>();
        for (name, health) in query {
            assert_eq!(name.0, "Person");
            assert_eq!(health.0, 0);
            health.0 += 100;
        }

        let mut threadpool = ThreadPool::new();
        let query = manager.query_mut::<(&Name, &mut Health)>();
        query.for_each(
            &mut threadpool,
            |(_, health)| {
                health.0 += 100;
            },
            32,
        );

        for health in manager.query_mut::<&Health>() {
            assert_eq!(health.0, 200)
        }
    }

    #[test]
    fn filter() {
        let mut manager = Scene::default();
        let e1 = manager.insert(Health(100));
        let e2 = manager.insert((Health(100), Ammo(30)));
        let query = manager.query_with::<&Health>(contains::<Ammo>());
        assert_eq!(query.len(), 1);
        let query = manager.query::<&Health>();
        assert_eq!(query.len(), 2);
        cleanup(&mut manager);

        let query = manager.query_with::<&Health>(modified::<Health>());
        assert_eq!(query.len(), 0);

        let mut entry = manager.entry_mut(e1).unwrap();
        entry.get_mut::<Health>().unwrap();

        let query = manager.query_with::<&Health>(modified::<Health>());
        assert_eq!(query.len(), 1);
    }
}
