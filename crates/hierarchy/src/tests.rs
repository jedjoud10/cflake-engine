#[cfg(test)]
mod tests {
    #[test]
    fn hierarchy() {
        let mut manager = Scene::default();
        let entity1 = manager.insert(Position::default());
        let entity2 = manager.insert(Position::default());
        manager.attach(entity2, entity1);

        let entry1 = manager.entry(entity1).unwrap();
        let entry2 = manager.entry(entity2).unwrap();

        let child = entry2.get::<Child>().unwrap();
        assert_eq!(child.parent(), entity1);
        assert_eq!(child.depth(), 1);
    }

    #[test]
    fn removed() {
        let mut manager = Scene::default();
        let entity1 = manager.insert(Position::default());
        let entity2 = manager.insert(Position::default());
        let mut entry1 = manager.entry_mut(entity1).unwrap();
        assert!(entry1.remove::<Position>());

        assert_eq!(manager.removed::<Position>().len(), 1);
        assert_eq!(manager.removed_mut::<Position>().len(), 1);
        assert_eq!(manager.removed::<Rotation>().len(), 0);
        assert_eq!(manager.removed_mut::<Rotation>().len(), 0);
    }

}