// Default asset implementations
impl crate::Asset for String {
    fn deserialize(self, _meta: &crate::metadata::AssetMetadata, bytes: &[u8]) -> Option<Self>
    where
        Self: Sized {
        Self::from_utf8(bytes.to_vec()).ok()
    }
}