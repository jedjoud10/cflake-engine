mod test {
    use std::{ops::DerefMut, sync::Mutex};

    use crate::preload_asset;

    // Some tests lol
    #[test]
    fn test() {
        preload_asset!(".\\resources\\test.txt");
        // Load the test text file
        println!("EE");
        let text = crate::assetc::load_text("\\resources\\test.txt").unwrap();
    }
}