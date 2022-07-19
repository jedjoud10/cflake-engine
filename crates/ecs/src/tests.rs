#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test() {
        let _manager = EcsManager::default();

        #[derive(Component, Debug)]
        struct Name(&'static str);
        #[derive(Component, Debug)]
        struct Health(i32);
        #[derive(Component, Debug)]
        struct Ammo(u32);

        //manager.insert((Name("Red"), Health(100)));

        /*


        manager.insert((Name("Red"), Health(100))).unwrap();
        manager.insert((Name("Green"), Health(100))).unwrap();
        manager.insert((Name("Blue"), Health(100))).unwrap();

        let modifier = manager.modify(entity);
        modifier.insert::<>()

        let success = manager.query::<(&mut Name, &Health)>();
        assert_eq!(success.is_some(), true);
        assert_eq!(success.unwrap().len(), 3);
        let fail = manager.query::<(&mut Name, &mut Name)>();
        assert_eq!(fail.is_some(), false);
        drop(fail);
        let success2 = manager.view::<(&Name, &Health, &Name)>();
        assert_eq!(success2.len(), 3);
        */
    }
}
