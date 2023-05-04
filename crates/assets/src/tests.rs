#[cfg(test)]
mod tests {
    use crate::{asset, AssetLoadError, Assets};

    #[test]
    fn read() {
        let loader = Assets::new();
        asset!(loader, "test/text.txt", "src/assets/");
        let string = loader.load::<String>("test/text.txt");
        assert_eq!(string.unwrap(), "this is a test file\n1234567890");
    }

    #[test]
    fn not_found() {
        let loader = Assets::new();
        let string = loader.load::<String>("test/text.txt");
        assert!(string.is_err());
    }

    #[test]
    fn parse_error() {
        let loader = Assets::new();
        asset!(loader, "test/invalid.txt", "src/assets/");
        let string = loader.load::<String>("test/invalid.txt");
        assert!(string.is_err());
    }

    #[test]
    fn read_iter() {
        let loader = Assets::new();
        asset!(loader, "test/text.txt", "src/assets/");
        let mut strings = loader.load_from_iter::<String>(["test/text.txt"]);
        let string = strings.pop().unwrap();
        assert_eq!(string.unwrap(), "this is a test file\n1234567890");
    }

    #[test]
    fn not_found_iter() {
        let loader = Assets::new();
        let mut strings = loader.load_from_iter::<String>(["test/text.txt"]);
        let string = strings.pop().unwrap();
        assert!(matches!(
            string.unwrap_err(),
            AssetLoadError::CachedNotFound(_)
        ));
    }

    #[test]
    fn read_async() {
        let loader = Assets::new();
        asset!(loader, "test/text.txt", "src/assets/");
        let handle = loader.async_load::<String>("test/text.txt");
        let string = loader.wait(handle).unwrap();
        assert_eq!(string, "this is a test file\n1234567890");
    }

    #[test]
    fn not_found_async() {
        let loader = Assets::new();
        let handle = loader.async_load::<String>("test/text.txt");
        let string = loader.wait(handle);
        assert!(matches!(
            string.unwrap_err(),
            AssetLoadError::CachedNotFound(_)
        ));
    }

    #[test]
    fn read_async_iter() {
        let loader = Assets::new();
        asset!(loader, "test/text.txt", "src/assets/");
        let mut handles = loader.async_load_from_iter::<String>(["test/text.txt"]);
        let handle = handles.pop().unwrap();
        let mut vec = loader.wait_from_iter([handle]);
        let last = vec.pop().unwrap();
        assert!(last.is_ok());
        assert_eq!(last.unwrap(), "this is a test file\n1234567890");
    }

    #[test]
    fn not_found_async_iter() {
        let loader = Assets::new();
        let mut handles = loader.async_load_from_iter::<String>(["test/text.txt"]);
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

            fn deserialize(
                data: crate::Data,
                context: Self::Context<'_>,
                settings: Self::Settings<'_>,
            ) -> Result<Self, Self::Err> {
                assert_eq!(*context, 69);
                String::deserialize(data, (), settings).map(Contextual)
            }
        }

        let loader = Assets::new();
        asset!(loader, "test/text.txt", "src/assets/");
        let context = 69u32;
        let string = loader.load::<Contextual>(("test/text.txt", &context));
        assert_eq!(string.unwrap().0, "this is a test file\n1234567890");
    }
}
