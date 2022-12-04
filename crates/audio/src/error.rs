use thiserror::Error;

// An error that might occur during clip deserialization
#[derive(Debug, Error)]
pub enum AudioClipError {
    #[error("MiniMP3 deserialization error {0}")]
    MP3(minimp3::Error),
}