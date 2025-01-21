use std::time::Instant;
use crate::SteamConfig;
use tokio::sync::mpsc::Sender;


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Message processor receiver closed")]
    ReceiverClosed
}

pub struct LogWatcher {
    config: SteamConfig,
    sender: Sender<String>,
}

impl LogWatcher {
    pub fn new(config: SteamConfig, sender: Sender<String>) -> Self {
        Self { config, sender }
    }

    fn extract_message(&self, line: &str) -> Option<String> {
        let prefixes = [
            format!("(TEAM) {} :", self.config.user_name),
            format!("*DEAD*(TEAM) {} :", self.config.user_name),
            format!("{} :", self.config.user_name),
            format!("*DEAD* {} :", self.config.user_name),
        ];

        for prefix in prefixes.iter() {
            if line.starts_with(prefix) {
                return Some(line.replacen(prefix, "", 1).trim().to_string());
            }
        }

        None
    }

    pub async fn watch_tf2_log(&self) -> Result<(), Error> {
        let log_path = &self.config.log_path;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(200));
        
        let last_len = std::fs::metadata(log_path)?.len();
        println!("starting to watch from position: {}", last_len);
        let mut last_len = last_len;

        loop {
            interval.tick().await;
            let read_start = Instant::now();

            if let Ok(metadata) = std::fs::metadata(log_path) {
                let new_len = metadata.len();
                if new_len > last_len {
                    if let Ok(mut file) = std::fs::File::open(log_path) {
                        use std::io::{Seek, SeekFrom, Read};
                        file.seek(SeekFrom::Start(last_len))?;
                        let mut buffer = vec![0; (new_len - last_len) as usize];
                        file.read_exact(&mut buffer)?;
                        println!("Read took: {:?}", read_start.elapsed());
                        let content = String::from_utf8_lossy(&buffer);

                        for line in content.lines() {
                            if let Some(message) = self.extract_message(line) {
                                self.sender.send(message).await.map_err(|_| Error::ReceiverClosed)?;
                            }
                        }
                        
                        last_len = new_len;
                    }
                }
            }
        }
    }
}

