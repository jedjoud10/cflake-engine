#[cfg(test)]
pub mod tests {
    use crate::{asset};


    #[test]
    fn test() {
        // Test
        asset!("./assets/sus/test.txt");
        let text = crate::assetc::load::<String>("sus/test.txt").unwrap();
        assert_eq!(text, "Le test ouioui".to_string())
    }
}