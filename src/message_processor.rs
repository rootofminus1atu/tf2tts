use crate::tts::tts_trait::Tts;
use tempfile::NamedTempFile;
use tokio::sync::mpsc::{Sender, Receiver};
use tracing::info;


#[derive(Debug, Clone, thiserror::Error)]
pub enum Error<E> {
    #[error("Could not get the speech: {0}")]
    CouldNotGetSpeech(#[from] E),
    #[error("Audio player receiver closed")]
    ReceiverClosed
}

// TODO: 
// - start fetching as soon as possible, while maintaining FIFO order

pub struct MessageProcessor<T: Tts> {
    receiver: Receiver<String>,
    tts_service: T,
    sender: Sender<NamedTempFile>
}

impl<T: Tts> MessageProcessor<T> {
    pub fn new(receiver: Receiver<String>, tts_service: T, sender: Sender<NamedTempFile>) -> Self {
        Self { 
            receiver, 
            tts_service,
            sender
        }
    }

    pub async fn run(&mut self) -> Result<(), Error<T::Error>> {
        while let Some(message) = self.receiver.recv().await {
            tracing::info!("Processing message: {}", message);

            info!("[VISUALIZE] PROCESSING START {}", message);
            let sound_file = match self.tts_service.get_speech(&message).await {
                Ok(sf) => sf,
                Err(e) => {
                    tracing::error!("{}", e);  // this error is non-critical, so it's just logged
                    continue;
                }
            };
            info!("[VISUALIZE] PROCESSING END {}", message);

            self.sender.send(sound_file).await
                .map_err(|_| Error::ReceiverClosed)?;  // this error is critical, so it's propagated to App
        }

        Ok(())
    }
}