// FIXME: Why in the EVERLIVING FUCK is it NOT DETERMINISTIC

use crate::prelude::*;

#[derive(Component, Debug, PartialEq, Eq, Clone, Default)]
struct Name(&'static str);
#[derive(Component, Debug, PartialEq, Eq, Clone, Default)]
struct Health(i32);
#[derive(Component, Debug, PartialEq, Eq, Clone, Copy, Default)]
struct Ammo(u32);
#[derive(Component, Debug, PartialEq, Eq, Clone, Copy, Default)]
struct Placeholder();

#[allow(dead_code)]
fn cleanup(ecs: &mut Scene) {
    for (_, archetype) in ecs.archetypes_mut() {
        for (_, column) in archetype.table_mut().iter_mut() {
            column.delta_frame_states_mut().reset();
        }
    }

    ecs.removed.clear();
}


fn test() {
    fn consume<T>(a: &T, b: &T) {
    }

    let mut scene = Scene::default();
    let entity = scene.insert(Health(0));
    let entry = scene.entry_mut(entity).unwrap();
    let layout = entry.as_query::<&Health>().unwrap();
    let layout2 = entry.as_query::<&Health>().unwrap();
    consume(layout, layout2)
}

/*
#[test]
fn entries() {
    let mut manager = Scene::default();

    let entity = manager.insert(Name("Basic"));
    let mut entry = manager.entry_mut(entity).unwrap();

    assert_eq!(entry.get_mut::<Name>(), Some(&mut Name("Basic")));
    assert!(entry.get_mut::<Ammo>().is_none());

    let mask = registry::mask::<Name>();
    let archetype = manager.archetypes().get(&mask).unwrap();
    let states = archetype.states::<Name>().unwrap();
    assert!(states.get(0).unwrap().modified);

    let entry = manager.entry(entity).unwrap();
    assert_eq!(entry.get::<Name>(), Some(&Name("Basic")));
    assert!(entry.get::<Ammo>().is_none());

    let mut entry = manager.entry_mut(entity).unwrap();
    entry.insert(Health(100)).unwrap();
    assert!(!entry.contains::<Ammo>());
    /*
    assert!(!entry.remove::<Ammo>());
    entry.insert(Ammo(100)).unwrap();
    assert!(entry.remove::<Ammo>());
    assert!(!entry.contains::<Ammo>());
    assert!(entry.contains::<Health>());
    assert!(entry.contains::<Name>());
    */
}
*/

#[test]
fn mask() {
    let mask1 = Mask::from(0b0100u64);
    let mask2 = Mask::from(0b1111u64);
    assert!(mask2.contains(mask1));
}

/*
#[test]
fn proto() {
    let mut manager = Scene::default();
    let entity = manager.insert((Name(""), Health(100)));
    let mut entry = manager.entry_mut(entity).unwrap();
    assert_eq!(entry.archetype().len(), 1);
    entry.insert::<Ammo>(Ammo(0)).unwrap();
}

#[test]
fn moving() {
    let mut manager = Scene::default();
    let entity = manager.insert((Name(""), Health(100)));
    let mut entry = manager.entry_mut(entity).unwrap();
    //assert!(entry.remove::<Health>());
    assert_eq!(entry.archetype().len(), 1);
    entry.insert::<Ammo>(Ammo(0)).unwrap();
    assert!(entry.insert::<Ammo>(Ammo(0)).is_none());
    assert!(entry.insert::<Ammo>(Ammo(0)).is_none());
    assert_eq!(entry.archetype().len(), 1);
}
*/

/*
#[test]
fn states() {
    let mut manager = Scene::default();

    manager.extend_from_iter(std::iter::repeat(Name("Test")).take(32));
    let mask = Mask::from_bundle::<Name>();
    let archetype = manager.archetypes().get(&mask).unwrap();
    let states = archetype.states::<Name>().unwrap();
    let chunk = states.chunks()[0];
    println!("{:b}", chunk.added);
    println!("{:b}", (1usize << 32) - 1);
    assert_eq!(chunk.added, (1 << 32) - 1);
    assert_eq!(chunk.modified, (1 << 32) - 1);
    assert_eq!(chunk.added.count_ones(), 32);
    assert_eq!(chunk.modified.count_ones(), 32);
    cleanup(&mut manager);

    manager.extend_from_iter(std::iter::repeat((Name("Test 2"), Health(100))).take(64));
    let query = manager.query_with::<&Entity>(added::<Name>() & added::<Health>());
    assert_eq!(query.len(), 64);

    let mask = Mask::from_bundle::<(Name, Health)>();
    let archetype = manager.archetypes().get(&mask).unwrap();
    let states = archetype.states::<Name>().unwrap();
    let sum1: u32 = states.chunks().iter().map(|c| c.added.count_ones()).sum();
    let states2 = archetype.states::<Health>().unwrap();
    let sum2: u32 = states2.chunks().iter().map(|c| c.added.count_ones()).sum();
    assert_eq!(sum1, 64);
    assert_eq!(sum2, 64);

    assert_eq!(states.chunks()[0].added, usize::MAX);
    assert_eq!(states2.chunks()[0].added, usize::MAX);

    cleanup(&mut manager);
    manager.extend_from_iter(std::iter::repeat((Name("Test 2"), Health(100))).take(64));
    let query = manager.query_with::<&Entity>(contains::<Name>() & contains::<Health>());
    assert_eq!(query.len(), 128);

    cleanup(&mut manager);
    manager.extend_from_iter(std::iter::repeat((Name("Test 2"), Health(100))).take(100));
    let query = manager.query_with::<&Entity>(contains::<Name>() & contains::<Health>());
    assert_eq!(query.len(), 228);
}



#[test]
fn moving_batch() {
    let mut scene = Scene::default();
    let entities = scene
        .extend_from_iter(
            std::iter::repeat((Name::default(), Health(50), Ammo(100))).take(5000),
        )
        .to_vec();

    cleanup(&mut scene);
    for (i, id) in entities.iter().enumerate() {
        if i % 10 == 0 {
            let mut entry = scene.entry_mut(*id).unwrap();
            assert!(entry.remove::<Name>());
            entry.insert::<Placeholder>(Placeholder()).unwrap();
        }
    }

    /*
    let filter = added::<Placeholder>();
    for (health, ammo) in
        scene.query_mut_with::<(&mut Health, &Ammo)>(filter)
    {
        health.0 += ammo.0 as i32;
    }

    for (i, id) in entities.iter().enumerate() {
        let entry = scene.entry_mut(*id).unwrap();
        let data = entry.get::<Health>();
        if i % 10 == 0 {
            assert_eq!(data, Some(&Health(150)));
        } else {
            assert_eq!(data, Some(&Health(50)));
        }
    }
    */
}
*/

/*
#[test]
fn threaded() {
    let mut scene = Scene::default();

    scene
        .extend_from_iter(
            std::iter::repeat((Name::default(), Health(50), Ammo(100))).take(4096),
        )
        .to_vec();

    todo!();
    /*
    scene.query_mut::<(&mut Ammo, &mut Health)>().for_each(
        &mut threadpool,
        |(ammo, health)| {
            ammo.0 += 100;
            health.0 -= 50;
        },
        512,
    );
    */

    for (ammo, health) in scene.query_mut::<(&Ammo, &Health)>() {
        assert_eq!(ammo.0, 200);
        assert_eq!(health.0, 0);
    }
}

#[test]
fn queries() {
    let mut manager = Scene::default();
    let iter = (0..4096).map(|_| (Name("Person"), Health(100)));
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

    let query = manager.query_mut::<(&Name, &mut Health)>();
    /*
    query.for_each(
        &mut threadpool,
        |(_, health)| {
            health.0 += 100;
        },
        32,
    );
    */
    todo!();

    assert_eq!(manager.query_mut::<&Health>().len(), 4096);
    for health in manager.query_mut::<&Health>() {
        assert_eq!(health.0, 200)
    }
}

#[test]
fn optional_queries() {
    let mut manager = Scene::default();
    let iter = (0..4096).map(|_| (Name("Person"), Health(100)));
    manager.extend_from_iter(iter);
    assert_eq!(manager.query_mut::<&Health>().len(), 4096);
    assert_eq!(manager.query_mut::<&Entity>().len(), 4096);
    assert_eq!(manager.query_mut::<Option<&Health>>().len(), 4096);
    assert_eq!(
        manager.query_mut::<(&Entity, Option<&Health>)>().len(),
        4096
    );
    assert_eq!(manager.query_mut::<&Health>().into_iter().count(), 4096);
    assert_eq!(
        manager.query_mut::<Option<&Health>>().into_iter().count(),
        4096
    );
    assert_eq!(manager.query_mut::<&Entity>().into_iter().count(), 4096);

    let query = manager.query_mut::<(&Name, &mut Health, Option<&Ammo>)>();
    assert_eq!(query.len(), 4096);
    for (name, health, ammo) in query {
        assert_eq!(name.0, "Person");
        assert_eq!(health.0, 100);
        assert_eq!(ammo, None);
        health.0 -= 100;
    }

    let query = manager.query_mut::<(&Name, &mut Health, Option<&Ammo>)>();
    assert_eq!(query.len(), 4096);
    for (name, health, ammo) in query {
        assert_eq!(name.0, "Person");
        assert_eq!(health.0, 0);
        assert_eq!(ammo, None);
        health.0 += 100;
    }

    let query = manager.query_mut::<(&Name, &mut Health, Option<&Ammo>)>();
    /*
    query.for_each(
        &mut threadpool,
        |(_, health, ammo)| {
            health.0 += 100;
            assert_eq!(ammo, None);
        },
        32,
    );
    */
    todo!();

    for health in manager.query_mut::<&Health>() {
        assert_eq!(health.0, 200)
    }
}
*/

/*/
#[test]
fn filter_ref() {
    let mut manager = Scene::default();
    let e1 = manager.insert(Health(100));
    let _e2 = manager.insert((Health(100), Ammo(30)));
    let _e3 = manager.insert((Health(100), Ammo(30)));
    let query = manager.query_with::<&Health>(contains::<Ammo>());
    assert_eq!(query.len(), 2);
    assert_eq!(query.into_iter().count(), 2);
    let query = manager.query::<&Health>();
    assert_eq!(query.len(), 3);
    assert_eq!(query.into_iter().count(), 3);
    cleanup(&mut manager);
    let query = manager.query_with::<&Health>(modified::<Health>());
    assert_eq!(query.len(), 0);
    assert_eq!(query.into_iter().count(), 0);

    let mut entry = manager.entry_mut(e1).unwrap();
    entry.get_mut::<Health>().unwrap();

    let query = manager.query_with::<&Health>(modified::<Health>());
    assert_eq!(query.len(), 1);
    assert_eq!(query.into_iter().count(), 1);

    let query = manager.query_with::<&Entity>(contains::<Health>() & !contains::<Ammo>());
    assert_eq!(query.len(), 1);
    assert_eq!(query.into_iter().count(), 1);

    let query = manager.query_with::<&Entity>(contains::<Health>() & contains::<Ammo>());
    assert_eq!(query.len(), 2);
    assert_eq!(query.into_iter().count(), 2);
}

#[test]
fn filter_mut() {
    let mut manager = Scene::default();
    let e1 = manager.insert(Health(100));
    let _e2 = manager.insert((Health(100), Ammo(30)));
    let _e3 = manager.insert((Health(100), Ammo(30)));
    let query = manager.query_mut_with::<&mut Health>(contains::<Ammo>());
    assert_eq!(query.len(), 2);
    assert_eq!(query.into_iter().count(), 2);
    let query = manager.query_mut::<&mut Health>();
    // WTF sometimes this is false??? maybe UB?
    assert_eq!(query.len(), 3);
    assert_eq!(query.into_iter().count(), 3);
    cleanup(&mut manager);

    let query = manager.query_mut_with::<&mut Health>(modified::<Health>());
    assert_eq!(query.len(), 0);
    assert_eq!(query.into_iter().count(), 0);

    let mut entry = manager.entry_mut(e1).unwrap();
    entry.get_mut::<Health>().unwrap();

    let query = manager.query_mut_with::<&Health>(modified::<Health>());
    assert_eq!(query.len(), 1);
    assert_eq!(query.into_iter().count(), 1);

    let query = manager.query_mut_with::<&Entity>(contains::<Health>() & !contains::<Ammo>());
    assert_eq!(query.len(), 1);
    assert_eq!(query.into_iter().count(), 1);

    let query = manager.query_mut_with::<&Entity>(contains::<Health>() & contains::<Ammo>());
    assert_eq!(query.len(), 2);
    assert_eq!(query.into_iter().count(), 2);

    let query = manager.query_mut_with::<&Entity>(
        contains::<Health>() & contains::<Ammo>() & contains::<Ammo>(),
    );
    assert_eq!(query.len(), 2);
    assert_eq!(query.into_iter().count(), 2);

    let query = manager.query_mut_with::<&Entity>(
        (contains::<Health>() & !contains::<Ammo>()) & contains::<Ammo>(),
    );
    assert_eq!(query.len(), 0);
    assert_eq!(query.into_iter().count(), 0);
}
*/

/*
#[test]
fn filter_mut() {
    let mut manager = Scene::default();
    let e1 = manager.insert(Health(100));
    let _e2 = manager.insert((Health(100), Ammo(30)));
    let _e3 = manager.insert((Health(100), Ammo(30)));
    let query = manager.query_mut_with::<&mut Health>(contains::<Ammo>());
    assert_eq!(query.len(), 2);
    assert_eq!(query.into_iter().count(), 2);
    let query = manager.query_mut::<&mut Health>();
    // WTF sometimes this is false??? maybe UB?
    assert_eq!(query.len(), 3);
    assert_eq!(query.into_iter().count(), 3);
    cleanup(&mut manager);

    let query = manager.query_mut_with::<&mut Health>(modified::<Health>());
    assert_eq!(query.len(), 0);
    assert_eq!(query.into_iter().count(), 0);

    let mut entry = manager.entry_mut(e1).unwrap();
    entry.get_mut::<Health>().unwrap();

    let query = manager.query_mut_with::<&Health>(modified::<Health>());
    assert_eq!(query.len(), 1);
    assert_eq!(query.into_iter().count(), 1);

    let query = manager.query_mut_with::<&Entity>(contains::<Health>() & !contains::<Ammo>());
    assert_eq!(query.len(), 1);
    assert_eq!(query.into_iter().count(), 1);

    let query = manager.query_mut_with::<&Entity>(contains::<Health>() & contains::<Ammo>());
    assert_eq!(query.len(), 2);
    assert_eq!(query.into_iter().count(), 2);

    let query = manager.query_mut_with::<&Entity>(
        contains::<Health>() & contains::<Ammo>() & contains::<Ammo>(),
    );
    assert_eq!(query.len(), 2);
    assert_eq!(query.into_iter().count(), 2);

    let query = manager.query_mut_with::<&Entity>(
        (contains::<Health>() & !contains::<Ammo>()) & contains::<Ammo>(),
    );
    assert_eq!(query.len(), 0);
    assert_eq!(query.into_iter().count(), 0);
}
*/
