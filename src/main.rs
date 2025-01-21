use app::{App, SteamConfig};
use tts::tts_impl::TtsService;
use dotenvy::dotenv;
use std::env;

mod tts;
mod message_processor;
mod audio_player;
mod log_watcher;
mod app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let user_id = env::var("STEAM_USER_ID")
        .expect("STEAM_USER_ID must be set in .env file");

    let steam_folder = r"C:\Program Files (x86)\Steam";

    let config = SteamConfig::new(&user_id, steam_folder)?;
    let tts_service = TtsService::new();

    let app = App::new(config, tts_service);
    app.run().await?;

    Ok(())
}
