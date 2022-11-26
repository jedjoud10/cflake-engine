#[cfg(test)]
mod tests {
    use crate::{persistent, Assets};

    #[test]
    fn read() {
        let loader = Assets::new(None);
        persistent!(loader, "test/text.txt");
        let string = loader.load::<String>("test/text.txt").unwrap();
        assert_eq!(string, "this is a test file\n1234567890");
    }

    #[test]
    fn read_iter() {
        let loader = Assets::new(None);
        persistent!(loader, "test/text.txt");
        let mut strings = loader.load_from_iter::<String>(["test/text.txt"]);
        let string = strings.pop().unwrap().unwrap();
        assert_eq!(string, "this is a test file\n1234567890");
    }

    #[test]
    fn read_async() {
        let mut threadpool = world::ThreadPool::default();
        let loader = Assets::new(None);
        persistent!(loader, "test/text.txt");
        let handle = loader.async_load::<String>(
            "test/text.txt",
            &mut threadpool
        ).unwrap();
        let string = loader.wait(handle);
        assert_eq!(string, "this is a test file\n1234567890");
    }

    #[test]
    fn read_async_iter() {
        let mut threadpool = world::ThreadPool::default();
        let loader = Assets::new(None);
        persistent!(loader, "test/text.txt");
        let mut handles = loader.async_load_from_iter::<String>(
            ["test/text.txt"],
            &mut threadpool
        );
        let handle = handles.pop().unwrap().unwrap();
        let mut vec = loader.wait_from_iter([handle]);
        let last = vec.pop();
        assert!(last.is_some());
        assert_eq!(last.unwrap(), "this is a test file\n1234567890");
    }
}
