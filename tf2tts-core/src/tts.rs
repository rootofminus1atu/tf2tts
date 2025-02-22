use tempfile::NamedTempFile;

pub trait Tts {
    type Error: std::error::Error + Send + Sync + 'static;
    async fn get_speech(&self, text: &str) -> Result<NamedTempFile, Self::Error>;
}