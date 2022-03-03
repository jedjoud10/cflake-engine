#[cfg(test)]
pub mod tests {
    use crate::{asset, init, Asset};

    #[test]
    fn test() {
        // Test
        asset!("./assets/sus/test.txt");
        init!("/src/assets");
        let text: String = Asset::load("sus/test.txt").unwrap();
        assert_eq!(text, "Le test ouioui".to_string())
    }
}
