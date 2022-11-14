use crate::*;
use world::ThreadPool;

#[derive(Component, Debug, PartialEq, Eq, Clone)]
struct Name(&'static str);
#[derive(Component, Debug, PartialEq, Eq, Clone)]
struct Health(i32);
#[derive(Component, Debug, Clone, Copy)]
struct Ammo(u32);

fn cleanup(ecs: &mut Scene) {
    for (_, archetype) in ecs.archetypes_mut() {
        for (_, column) in archetype.state_table_mut().iter_mut() {
            column.clear();
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
    let column = archetype.states::<Name>().unwrap();
    assert!(column.get(0).unwrap().modified == 1);

    let entry = manager.entry(entity).unwrap();
    assert_eq!(entry.get::<Name>(), Some(&Name("Basic")));
    assert!(entry.get::<Ammo>().is_none());
}

#[test]
fn moving() {
    let mut manager = Scene::default();
    let entity = manager.insert((Name(""), Health(100)));
    let mut entry = manager.entry_mut(entity).unwrap();
    entry.remove_bundle::<Health>().unwrap();
    entry.insert_bundle::<Ammo>(Ammo(0)).unwrap();
}

#[test]
fn queries() {
    let mut manager = Scene::default();
    let iter = (0..130).map(|_| (Name("Person"), Health(100)));
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
fn filter_ref() {
    // TODO: Fix infinite loop
    let mut manager = Scene::default();
    let e1 = manager.insert(Health(100));
    let e2 = manager.insert((Health(100), Ammo(30)));
    let query = manager.query_with::<&Health>(contains::<Ammo>());
    assert_eq!(query.len(), 1);
    assert_eq!(query.into_iter().count(), 1);
    let query = manager.query::<&Health>();
    assert_eq!(query.len(), 2);
    assert_eq!(query.into_iter().count(), 2);
    cleanup(&mut manager);

    let query = manager.query_with::<&Health>(modified::<Health>());
    assert_eq!(query.len(), 0);
    assert_eq!(query.into_iter().count(), 0);

    let mut entry = manager.entry_mut(e1).unwrap();
    entry.get_mut::<Health>().unwrap();

    
    let query = manager.query_with::<&Health>(modified::<Health>());
    assert_eq!(query.len(), 1);
    assert_eq!(query.into_iter().count(), 1);
}


#[test]
fn filter_mut() {
    let mut manager = Scene::default();
    let e1 = manager.insert(Health(100));
    let e2 = manager.insert((Health(100), Ammo(30)));
    let query = manager.query_mut_with::<&mut Health>(contains::<Ammo>());
    assert_eq!(query.len(), 1);
    assert_eq!(query.into_iter().count(), 1);
    let query = manager.query_mut::<&mut Health>();
    assert_eq!(query.len(), 2);
    assert_eq!(query.into_iter().count(), 2);
    cleanup(&mut manager);

    let query = manager.query_mut_with::<&mut Health>(modified::<Health>());
    assert_eq!(query.len(), 0);
    assert_eq!(query.into_iter().count(), 0);

    let mut entry = manager.entry_mut(e1).unwrap();
    entry.get_mut::<Health>().unwrap();

    
    let query = manager.query_mut_with::<&mut Health>(modified::<Health>());
    assert_eq!(query.len(), 1);
    assert_eq!(query.into_iter().count(), 1);
}
