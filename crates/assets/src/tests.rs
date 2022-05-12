#[cfg(test)]
pub mod tests {
    use crate::{asset, loader::AssetLoader, Asset};

    impl Asset<'static> for String {
        type OptArgs = ();

        fn is_extension_valid(extension: &str) -> bool {
            extension == "txt"
        }

        fn deserialize(bytes: &[u8], args: Self::OptArgs) -> Self {
            String::from_utf8(bytes.to_vec()).unwrap()
        }
    }

    #[test]
    fn test() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/assets");
        let mut loader = AssetLoader::new(path);
        asset!(&mut loader, "./assets/sus/test.txt");
        let val = <String as Asset>::try_load(&mut loader, "sus/test.txt").unwrap();
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
