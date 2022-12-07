#[cfg(test)]
mod storage {
    use crate::Storage;

    #[test]
    fn init() {
        let mut storage: Storage<u32> = Storage::default();
        let zero = storage.insert(0);
        let one = storage.insert(1);
        assert_eq!(0, *storage.get(&zero));
        assert_eq!(1, *storage.get(&one));
    }
}