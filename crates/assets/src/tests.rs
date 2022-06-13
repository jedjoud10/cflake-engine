#[cfg(test)]
pub mod tests {
    use std::{path::PathBuf, str::FromStr};

    use crate::{asset, loader::Assets, Asset};
    impl Asset<'static> for String {
        type Args = ();

        fn extensions() -> &'static [&'static str] {
            &["txt"]
        }

        fn deserialize(bytes: crate::loader::CachedSlice, _args: Self::Args) -> Self {
            String::from_utf8(bytes.as_ref().to_vec()).unwrap()
        }
    }

    #[test]
    fn test() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/assets");
        let mut loader = Assets::new(Some(PathBuf::from_str(path).unwrap()));
        asset!(&mut loader, "./assets/sus/test.txt");
        let val = loader.load::<String>("sus/test.txt").unwrap();
        dbg!(val);

        /*

                // Test
        asset!("./assets/sus/test.txt");
        init!("/src/assets");
        let text = crate::load::<String>("sus/test.txt").unwrap();
        assert_eq!(text, "Le test ouioui".to_string())

        */
    }
}
