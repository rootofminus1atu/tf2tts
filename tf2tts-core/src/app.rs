use tokio::sync::mpsc;

use crate::{audio_player::{AudioConfig, AudioPlayer}, log_watcher::{LogWatcher, LogWatcherConfig}, message_processor::MessageProcessor, providers::webapi::WebApiTts, tts::Tts};



pub type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub struct AppConfig {
    pub steam_config: LogWatcherConfig,
    pub audio_config: AudioConfig
}


#[derive(Debug, thiserror::Error)]
pub enum Error<E> {
    #[error("Log watcher error: {0}")]
    LogWatcherError(#[from] crate::log_watcher::Error),
    #[error("Message processor error: {0}")]
    MessageProcessorError(#[from] crate::message_processor::Error<E>), 
    #[error("Audio player error: {0}")]
    AudioPlayerError(#[from] crate::audio_player::Error)
}

pub struct App<T: Tts = WebApiTts> {
    watcher: LogWatcher,
    message_processor: MessageProcessor<T>,
    audio_player: AudioPlayer
}

impl<T: Tts> App<T> {
    pub fn new(config: AppConfig, tts_service: T) -> Self {
        let (log_sender, log_receiver) = mpsc::channel(100);
        let (audio_sender, audio_receiver) = mpsc::channel(100);

        let watcher = LogWatcher::new(config.steam_config, log_sender);
        let message_processor = MessageProcessor::new(log_receiver, tts_service, audio_sender);
        let audio_player = AudioPlayer::new(config.audio_config, audio_receiver);

        Self {
            watcher,
            message_processor,
            audio_player
        }
    } 

    pub async fn run(self) -> Result<(), Error<T::Error>>  {
        let watcher = self.watcher;
        let mut message_processor = self.message_processor;
        let mut audio_player = self.audio_player;

        tokio::select! {
            result = watcher.watch_tf2_log() => {
                result?;
                Ok(())
            }
            result = message_processor.run() => {
                result?;
                Ok(())
            }
            result = audio_player.run() => {
                result?;
                Ok(())
            }
        }
    }
}

