use dotenvy::dotenv;
use std::env;
use tf2tts_core::{app::{App, AppConfig}, audio_player::AudioConfig, log_watcher::LogWatcherConfig, providers::webapi::WebApiTts};


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
