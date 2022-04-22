// Default asset implementations
impl crate::Asset for String {
    type OptArgs = ();
    fn deserialize(_meta: &crate::metadata::AssetMetadata, bytes: &[u8], _input: Self::OptArgs) -> Option<Self>
    where
        Self: Sized,
    {
        Self::from_utf8(bytes.to_vec()).ok()
    }
}
