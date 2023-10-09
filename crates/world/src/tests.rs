#[allow(dead_code)]
#[cfg(test)]
mod test {
    use crate::prelude::*;
    use std::{sync::atomic::{AtomicU32, Ordering}, any::{Any, TypeId}};

    #[test]
    fn simple() {
        fn init(_world: &mut World, _init: &Init) {}
        fn init2(_world: &mut World, _init: &Init) {}

        let mut registry = Registry::<Init>::default();
        registry.insert(init).after(post_user);
        registry.insert(init2).before(init).after(post_user);

        registry.insert(move |_world: &mut World, _init: &Init| {
        }).before(init).before(pre_user); 

        registry.sort().unwrap();

        // TODO: Assert this pls
        let slice = registry.sorted_systems();
    }

    #[test]
    fn plugin() {
        static COUNTER: AtomicU32 = AtomicU32::new(0);

        fn init(_world: &mut World, _init: &Init) {
            let x = COUNTER.fetch_add(1, Ordering::Relaxed);
            assert_eq!(x, 0);
        }

        fn init2(_world: &mut World, _init: &Init) {
            let x = COUNTER.fetch_add(1, Ordering::Relaxed);
            assert_eq!(x, 1);
        }

        fn plugin(registries: &mut Registries) {
            registries.init.insert(init);
            registries.init.insert(init2).after(init);
        }

        let mut registries = Registries::default();
        plugin(&mut registries);
        registries.init.sort().unwrap();
        registries.init.execute(&mut World::default(), &Init);
        let x = COUNTER.load(Ordering::Relaxed);
        assert_eq!(x, 2);
    }

    #[test]
    fn world() {
        let mut world = World::default();

        assert_eq!(world.contains::<u32>(), false);
        assert_eq!(world.get::<u32>().is_some(), false);
        assert_eq!(world.get_mut::<u32>().is_some(), false);

        world.insert::<u32>(69420);
        
        assert_eq!(world.contains::<u32>(), true);
        assert_eq!(world.get::<u32>().is_some(), true);
        assert_eq!(world.get_mut::<u32>().is_some(), true);
        let x = world.remove::<u32>().unwrap();
        assert_eq!(x, 69420);
    }

    #[test]
    fn resource() {
        let resource = 10u32;
        let boxed: Box<dyn Any> = Box::new(resource);
        let downcasted = boxed.downcast_ref::<u32>().unwrap();
        assert_eq!(*downcasted, 10);
    }

    #[test]
    fn different_marker_event_type_id() {
        fn type_id<T: 'static>(_: T) -> TypeId {
            TypeId::of::<T>()
        }

        let a = type_id(pre_user::<Init>);
        let b = type_id(pre_user::<Update>);
        
        let c = type_id(post_user::<Init>);
        let d = type_id(post_user::<Update>);
        assert_ne!(a, b);
        assert_ne!(a, c);
    }
}