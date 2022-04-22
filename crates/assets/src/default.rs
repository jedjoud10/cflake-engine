// Default asset implementations
impl crate::Asset for String {
    type Input = ();
    fn deserialize(_meta: &crate::metadata::AssetMetadata, bytes: &[u8], input: Self::Input) -> Option<Self>
    where
        Self: Sized,
    {
        Self::from_utf8(bytes.to_vec()).ok()
    }
}
