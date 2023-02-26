#[cfg(test)]
mod tests {
    use crate::{persistent, AssetLoadError, Assets};

    #[test]
    fn read() {
        let mut loader = Assets::new(None);
        persistent!(loader, "test/text.txt");
        let string = loader.load::<String>("test/text.txt");
        assert_eq!(
            string.unwrap(),
            "this is a test file\n1234567890"
        );
    }

    #[test]
    fn not_found() {
        let mut loader = Assets::new(None);
        let string = loader.load::<String>("test/text.txt");
        assert!(string.is_err());
    }

    #[test]
    fn parse_error() {
        let mut loader = Assets::new(None);
        persistent!(loader, "test/invalid.txt");
        let string = loader.load::<String>("test/invalid.txt");
        assert!(string.is_err());
    }

    #[test]
    fn read_iter() {
        let mut loader = Assets::new(None);
        persistent!(loader, "test/text.txt");
        let mut strings =
            loader.load_from_iter::<String>(["test/text.txt"]);
        let string = strings.pop().unwrap();
        assert_eq!(
            string.unwrap(),
            "this is a test file\n1234567890"
        );
    }

    #[test]
    fn not_found_iter() {
        let mut loader = Assets::new(None);
        let mut strings =
            loader.load_from_iter::<String>(["test/text.txt"]);
        let string = strings.pop().unwrap();
        assert!(matches!(
            string.unwrap_err(),
            AssetLoadError::CachedNotFound(_)
        ));
    }

    #[test]
    fn read_async() {
        let mut threadpool = utils::ThreadPool::default();
        let mut loader = Assets::new(None);
        persistent!(loader, "test/text.txt");
        let handle = loader
            .async_load::<String>("test/text.txt", &mut threadpool);
        let string = loader.wait(handle).unwrap();
        assert_eq!(string, "this is a test file\n1234567890");
    }

    #[test]
    fn not_found_async() {
        let mut threadpool = utils::ThreadPool::default();
        let mut loader = Assets::new(None);
        let handle = loader
            .async_load::<String>("test/text.txt", &mut threadpool);
        let string = loader.wait(handle);
        assert!(matches!(
            string.unwrap_err(),
            AssetLoadError::CachedNotFound(_)
        ));
    }

    #[test]
    fn read_async_iter() {
        let mut threadpool = utils::ThreadPool::default();
        let mut loader = Assets::new(None);
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
        let mut loader = Assets::new(None);
        let mut handles = loader.async_load_from_iter::<String>(
            ["test/text.txt"],
            &mut threadpool,
        );
        let handle = handles.pop().unwrap();
        let mut vec = loader.wait_from_iter([handle]);
        let string = vec.pop().unwrap();
        assert!(matches!(
            string.unwrap_err(),
            AssetLoadError::CachedNotFound(_)
        ));
    }

    #[test]
    fn context() {
        struct Contextual(String);

        impl crate::Asset for Contextual {
            type Context<'ctx> = &'ctx u32;
            type Settings<'stg> = ();
            type Err = std::string::FromUtf8Error;

            fn extensions() -> &'static [&'static str] {
                &["txt"]
            }

            fn deserialize<'c, 's>(
                data: crate::Data,
                context: Self::Context<'c>,
                settings: Self::Settings<'s>,
            ) -> Result<Self, Self::Err> {
                assert_eq!(*context, 69);
                String::deserialize(data, (), settings)
                    .map(Contextual)
            }
        }

        let mut loader = Assets::new(None);
        persistent!(loader, "test/text.txt");
        let context = 69u32;
        let string =
            loader.load::<Contextual>(("test/text.txt", &context));
        assert_eq!(
            string.unwrap().0,
            "this is a test file\n1234567890"
        );
    }
}
