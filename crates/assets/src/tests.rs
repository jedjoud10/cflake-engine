#[cfg(test)]
pub mod tests {
    use crate::{asset, loader::AssetLoader, Asset};
    impl Asset<'static> for String {
        type Args = ();

        fn extensions() -> &'static [&'static str] {
            &["txt"]
        }

        fn deserialize(bytes: &[u8], path: std::path::PathBuf, args: Self::Args, ctx: crate::loader::LoadingContext) -> Self {
            todo!()
        }
    }

    #[test]
    fn test() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/assets");
        let mut loader = AssetLoader::new(path);
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
