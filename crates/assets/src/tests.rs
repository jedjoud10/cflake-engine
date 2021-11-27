#[cfg(test)]
mod test {
    use std::{ops::DerefMut, sync::Mutex};

    use crate::preload_asset;

    // Some tests lol
    #[test]
    fn test() {
        preload_asset!(".\\resources\\test.txt");
        // Load the test text file
        let text = crate::assetc::load_text("test.txt").unwrap();
        assert_eq!(text, "This is a test".to_string());
    }
}