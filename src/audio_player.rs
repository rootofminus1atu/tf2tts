use enigo::{Enigo, Key, Settings, Keyboard, Direction::{Press, Release}};
use tempfile::NamedTempFile;
use tokio::sync::mpsc::Receiver;
use rodio::{Decoder, OutputStream};
use std::fs::File;
use rodio::Source;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use tracing::info;
use std::io::Seek;

use crate::app::GenericError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Audio file with no extensions are not supported.")]
    MissingExtension,
    #[error("Got audio file with unsupported extension, only mp3 and wav are supported as of now.")]
    UnsupportedExtension,
    #[error("Io Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to decode audio file: {0}")]
    DecodeError(#[from] rodio::decoder::DecoderError),
    #[error("Failed to play audio: {0}")]
    PlaybackError(#[from] rodio::PlayError),
    #[error("This should never ever show up, but anyway: {0}")]
    TokioJoinError(#[from] tokio::task::JoinError),
    #[error("Failed to stream audio: {0}")]
    StreamError(#[from] rodio::StreamError),
    #[error("Failed to calculate the mp3 length.")]
    Mp3DurationNotFound
}

pub struct AudioPlayer {
    receiver: Receiver<NamedTempFile>,
    enigo: Enigo,
    is_key_held: bool,
}

impl AudioPlayer {
    pub fn new(receiver: Receiver<NamedTempFile>) -> Self {
        Self {
            receiver,
            enigo: Enigo::new(&Settings::default()).unwrap(),
            is_key_held: false
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        while let Some(speech_file) = self.receiver.recv().await {
            if !self.is_key_held {
                // TODO: pass down the vc key instaed
                self.enigo.key(Key::V, Press).unwrap();
                self.is_key_held = true;
            }

            if let Err(e) = self.play_speech(speech_file).await {
                tracing::error!("{}", e);  // also a way of doing non-critical errors
            };

            if self.receiver.is_empty() {
                self.enigo.key(Key::V, Release).unwrap();
                self.is_key_held = false;
            }
        }

        Ok(())
    }

    async fn play_speech(&self, speech_file: NamedTempFile) -> Result<(), Error> {
        let (duration, file) = tokio::task::spawn_blocking(move || {
            let path = speech_file.path();
            let extension = path
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .ok_or(Error::MissingExtension)?;

            let mut file = std::fs::File::open(path)?;

            let duration = match extension {
                "wav" => {
                    let source = Decoder::new(file.try_clone()?)?;
                    source.total_duration().unwrap()  // safe - should always work for WAVs I HOPE
                }
                "mp3" => get_mp3_duration(&file).map_err(|_| Error::Mp3DurationNotFound)?,  // TODO: manage this error
                _ => return Err(Error::UnsupportedExtension),
            };

            file.seek(std::io::SeekFrom::Start(0))?;

            Ok::<_, Error>((duration, file))
        })
        .await??;

        info!("Audio duration: {:?}", duration);

        let (_stream, stream_handle) = OutputStream::try_default()?;
        let source = Decoder::new(file)?;
        
        stream_handle.play_raw(source.convert_samples())?;
        tokio::time::sleep(duration).await;

        Ok(())
    }
}

// TODO: use a lib for this or improve the errors down below
fn get_mp3_duration(file: &File) -> Result<std::time::Duration, GenericError> {
    let mss = MediaSourceStream::new(Box::new(file.try_clone()?), Default::default());

    let mut hint = Hint::new();
    hint.with_extension("mp3");

    let metadata_opts: MetadataOptions = Default::default();
    let format_opts: FormatOptions = Default::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)?;

    let track = probed.format.default_track().ok_or("Mp3 time calculation error")?;
    let duration = track.codec_params.time_base.ok_or("Mp3 time calculation error")?;
    let n_frames = track.codec_params.n_frames.ok_or("Mp3 time calculation error")?;
    let duration = std::time::Duration::from_secs_f64(
        (n_frames as f64 * duration.numer as f64) / duration.denom as f64
    );

    Ok(duration)
}