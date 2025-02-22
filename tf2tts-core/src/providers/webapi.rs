use std::{fs::File, io::Cursor};
use std::io::copy;

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use tracing::info;

use crate::tts::Tts;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to fetch TTS audio: {0}")]
    FetchError(#[from] reqwest::Error),
    #[error("Failed to create temporary file: {0}")]
    TempFileError(#[from] tempfile::PersistError),
    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid text input: {0}")]
    InvalidInput(String),
}

pub struct WebApiTts {
    client: reqwest::Client
}

impl WebApiTts {
    pub fn new() -> Self {
        Self::new_with_client(reqwest::Client::new())
    }

    pub fn new_with_client(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl Tts for WebApiTts {
    type Error = Error;

    async fn get_speech(&self, text: &str) -> Result<NamedTempFile, Self::Error> {
        // TODO: more input validation
        if text.is_empty() {
            return Err(Error::InvalidInput("Empty text provided".into()));
        }

        let body = Body {
            text: text.into()
        };
    
        
        info!("[VISUALIZE] REQUESTING START {}", text);
        let mp3_link = self.client.post("https://api.getchipbot.com/api/v1/utility/text-to-speech")
            .json(&body)
            .send()
            .await?
            .json::<Resp>()
            .await?
            .data
            .location;
    
        let mp3 = self.client.get(mp3_link)
            .send()
            .await?
            .bytes()
            .await?;
        info!("[VISUALIZE] REQUESTING END {}", text);
    
        info!("[VISUALIZE] FILECREATION START {}", text);
        let temp_file = tempfile::Builder::new()
            .suffix(".mp3")
            .tempfile()?;
    
        // TODO: tokio blocking spawn
        let mut file = File::create(temp_file.path())?;
        copy(&mut Cursor::new(mp3), &mut file)?;
        info!("[VISUALIZE] FILECREATION END {}", text);
    
        Ok(temp_file)
    }
}

#[derive(Debug, Clone, Serialize)]
struct Body {
    text: String
}

#[derive(Debug, Clone, Deserialize)]
struct Resp {
    data: RespData
}

#[derive(Debug, Clone, Deserialize)]
struct RespData {
    #[serde(rename = "Location")]
    location: String
}