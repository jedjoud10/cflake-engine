#[cfg(test)]
mod tests {
    use crate::{persistent, Assets};

    #[test]
    fn read() {
        let loader = Assets::new(None);
        persistent!(loader, "test/text.txt");
        let string = loader.load::<String>("test/text.txt");
        assert_eq!(string.unwrap(), "this is a test file\n1234567890");
    }

    #[test]
    fn not_found() {
        let loader = Assets::new(None);
        let string = loader.load::<String>("test/text.txt");
        assert!(string.is_err());
    }

    #[test]
    fn read_iter() {
        let loader = Assets::new(None);
        persistent!(loader, "test/text.txt");
        let mut strings =
            loader.load_from_iter::<String>(["test/text.txt"]);
        let string = strings.pop().unwrap();
        assert_eq!(string.unwrap(), "this is a test file\n1234567890");
    }

    #[test]
    fn not_found_iter() {
        let loader = Assets::new(None);
        let mut strings =
        loader.load_from_iter::<String>(["test/text.txt"]);
        let string = strings.pop().unwrap();
        assert!(string.is_err());
    }

    #[test]
    fn read_async() {
        let mut threadpool = utils::ThreadPool::default();
        let loader = Assets::new(None);
        persistent!(loader, "test/text.txt");
        let handle = loader
            .async_load::<String>("test/text.txt", &mut threadpool);
        let string = loader.wait(handle).unwrap();
        assert_eq!(string, "this is a test file\n1234567890");
    }

    #[test]
    fn not_found_async() {
        let mut threadpool = utils::ThreadPool::default();
        let loader = Assets::new(None);
        let handle = loader
            .async_load::<String>("test/text.txt", &mut threadpool);
        let string = loader.wait(handle);
        assert!(string.is_err());
    }

    #[test]
    fn read_async_iter() {
        let mut threadpool = utils::ThreadPool::default();
        let loader = Assets::new(None);
        persistent!(loader, "test/text.txt");
        let mut handles = loader.async_load_from_iter::<String>(
            ["test/text.txt"],
            &mut threadpool,
        );
        let handle = handles.pop().unwrap();
        let mut vec = loader.wait_from_iter([handle]);
        let last = vec.pop().unwrap();
        assert!(last.is_ok());
        assert_eq!(last.unwrap(), "this is a test file\n1234567890");
    }

    #[test]
    fn not_found_async_iter() {
        let mut threadpool = utils::ThreadPool::default();
        let loader = Assets::new(None);
        let mut handles = loader.async_load_from_iter::<String>(
            ["test/text.txt"],
            &mut threadpool,
        );
        let handle = handles.pop().unwrap();
        let mut vec = loader.wait_from_iter([handle]);
        let last = vec.pop().unwrap();
        assert!(last.is_err());
    }
}
