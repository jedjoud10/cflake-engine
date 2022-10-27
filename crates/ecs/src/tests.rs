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
        let mut threadpool = ThreadPool::new();
        let iter = (0..4096).map(|_| (Name("Person"), Health(100)));
        let entity = manager.extend_from_iter(iter);
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
}
