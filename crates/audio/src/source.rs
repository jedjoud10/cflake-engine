// A single audio source that can be loaded
#[derive(Default)]
pub struct AudioSource {
    // The source's ID in the playback
    pub(crate) idx: Option<usize>,

    // A temporary location for our loaded rodio decoder before we add it to the playback cache
    pub(crate) temp: Option<Vec<u8>>,
}

// Each audio source is loadable
impl assets::Asset for AudioSource {
    fn load_medadata(mut self, data: &assets::AssetMetadata) -> Option<Self>
    where
        Self: Sized {
        // Pass the compressed bytes
        self.temp = Some(data.bytes.clone());
        Some(self)
    }
}