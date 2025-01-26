use app::{App, AppConfig};
use audio_player::AudioConfig;
use log_watcher::LogWatcherConfig;
use dotenvy::dotenv;
use providers::webapi::WebApiTts;
use std::env;

mod tts;
mod message_processor;
mod audio_player;
mod log_watcher;
mod app;
mod providers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let user_id = env::var("STEAM_USER_ID")
        .expect("STEAM_USER_ID must be set in .env file");

    let steam_folder = r"C:\Program Files (x86)\Steam";

    let config = AppConfig {
        steam_config: LogWatcherConfig::new(&user_id, steam_folder)?,
        audio_config: AudioConfig::new(None, Some("CABLE Input (VB-Audio Virtual Cable)".into()))?
    };

    let tts = WebApiTts::new();

    let app = App::new(config, tts);
    app.run().await?;

    Ok(())
}
