use regex::Regex;
use tokio::sync::mpsc;

use crate::{audio_player::AudioPlayer, log_watcher::LogWatcher, message_processor::MessageProcessor, tts::{tts_impl::TtsService, tts_trait::Tts}};



pub type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct SteamConfig {
    pub user_id: String,
    pub user_name: String,
    pub steam_folder: String,
    pub log_path: String
}

impl SteamConfig {
    pub fn new(user_id: &str, steam_folder: &str) -> Result<Self, GenericError> {
        let users_content = std::fs::read_to_string(format!(r"{}\config\loginusers.vdf", steam_folder))?;

        let re = Regex::new(&format!(r#""{}"\s*\{{[^}}]*"PersonaName"\s*"([^"]*)""#, user_id))?;
        let caps = re.captures(&users_content).ok_or("User ID not found")?;
        let persona_name = caps.get(1).ok_or("PersonaName not found")?.as_str();

        Ok(Self {
            user_id: user_id.into(),
            user_name: persona_name.into(),
            steam_folder: steam_folder.into(),
            log_path: format!(r"{}\steamapps\common\Team Fortress 2\tf\tf2consoleoutput.log", steam_folder),
        })
    }
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

pub struct App<T: Tts = TtsService> {
    watcher: LogWatcher,
    message_processor: MessageProcessor<T>,
    audio_player: AudioPlayer
}

impl<T: Tts> App<T> {
    pub fn new(config: SteamConfig, tts_service: T) -> Self {
        let (log_sender, log_receiver) = mpsc::channel(100);
        let (audio_sender, audio_receiver) = mpsc::channel(100);

        let watcher = LogWatcher::new(config, log_sender);
        let message_processor = MessageProcessor::new(log_receiver, tts_service, audio_sender);
        let audio_player = AudioPlayer::new(audio_receiver);

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

