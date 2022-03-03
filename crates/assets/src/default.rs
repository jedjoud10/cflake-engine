impl crate::Asset for String {
    fn load_raw(_meta: &crate::metadata::AssetMetadata, bytes: &[u8]) -> Option<String>
    where
        String: Sized {
        String::from_utf8(bytes.to_vec()).ok()
    }
}