#[cfg(test)]
pub mod tests {
    use crate::{asset, init};

    #[test]
    fn test() {
        // Test
        asset!("./assets/sus/test.txt");
        init!("/src/assets");
        let text = crate::assetc::load::<String>("sus/test.txt").unwrap();
        assert_eq!(text, "Le test ouioui".to_string())
    }
}
