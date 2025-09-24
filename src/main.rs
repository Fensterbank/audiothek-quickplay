use anyhow::{anyhow, Context, Result};
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use dialoguer::{theme::ColorfulTheme, Select};
use rodio::{Decoder, OutputStream, Sink, Source};
use serde::Deserialize;
use std::fs::File;
use std::io::{stdout, BufReader, Write};
use std::path::{Path};
use std::time::{Duration, Instant};
use std::env;

// --- Structs for API Deserialization ---
#[derive(Debug, Deserialize)]
struct ApiResponse { data: Option<Data> }
#[derive(Debug, Deserialize)]
struct Data { result: Option<ProgramSetResult> }
#[derive(Debug, Deserialize)]
struct ProgramSetResult { items: Items }
#[derive(Debug, Deserialize)]
struct Items { nodes: Vec<Node> }
#[derive(Debug, Deserialize)]
struct Node { title: String, audios: Vec<Audio> }
#[derive(Debug, Deserialize)]
struct Audio { url: String }

// --- Struct for podcasts.json Deserialization ---
#[derive(Debug, Deserialize, Clone)]
struct Podcast { key: String, id: String }

/// Fetches the latest episode's audio URL for a given show ID.
async fn get_latest_episode_url(show_id: &str) -> Result<(String, String)> {
    const QUERY: &str = r#"
    query ProgramSetEpisodesQuery($id: ID!, $offset: Int!, $count: Int!) {
      result: programSet(id: $id) {
        items(offset: $offset, first: $count, filter: { isPublished: { equalTo: true } }) {
          nodes { title, audios { url } }
        }
      }
    }
  "#;
    let variables = serde_json::json!({ "id": show_id, "offset": 0, "count": 1 });
    let query_encoded = urlencoding::encode(QUERY.trim());
    let variables_string = variables.to_string();
    let variables_encoded = urlencoding::encode(&variables_string);
    let url = format!(
        "https://api.ardaudiothek.de/graphql?query={}&variables={}",
        query_encoded, variables_encoded
    );

    let response = reqwest::get(&url).await.context("API request failed")?;
    if !response.status().is_success() {
        return Err(anyhow!("API request failed with status: {}", response.status()));
    }
    let api_response: ApiResponse = response.json().await.context("Failed to parse JSON")?;

    let episode = api_response
        .data.and_then(|d| d.result).and_then(|r| r.items.nodes.into_iter().next())
        .context("No episode found in API response")?;
    let audio = episode.audios.into_iter().next().context("Episode has no audio sources")?;
    Ok((episode.title, audio.url))
}

/// Loads podcast configuration from podcasts.json
fn load_podcasts(path: &Path) -> Result<Vec<Podcast>> {
    let file = File::open(path).with_context(|| format!("Failed to open file at {:?}", path))?;
    let reader = BufReader::new(file);
    let podcasts = serde_json::from_reader(reader).with_context(|| format!("Failed to parse JSON from {:?}", path))?;
    Ok(podcasts)
}

/// Plays audio from a URL with interactive controls.
async fn play_audio_interactive(url: &str) -> Result<()> {
    let response = reqwest::get(url).await?.bytes().await?;
    let cursor = std::io::Cursor::new(response);
    let source = Decoder::new(cursor)?.convert_samples::<f32>();
    let total_duration = source.total_duration().unwrap_or_default();

    let (_stream, stream_handle) = OutputStream::try_default().context("No audio output device")?;
    let sink = Sink::try_new(&stream_handle).context("Failed to create audio sink")?;
    sink.append(source);

    enable_raw_mode()?;
    let mut stdout = stdout();
    let mut last_known_pos = Duration::from_secs(0);
    let mut last_update_time = Instant::now();

    loop {
        let elapsed_since_update = last_update_time.elapsed();
        let current_pos = if !sink.is_paused() {
            (last_known_pos + elapsed_since_update).min(total_duration)
        } else {
            last_known_pos
        };

        let status = if sink.is_paused() { "Paused" } else { "Playing" };

        write!(
            stdout,
            "\rStatus: {} | Position: {:02}:{:02} / {:02}:{:02} | Controls: [Space] Pause/Play, [<-] Rewind 10s, [->] Forward 10s, [q] Quit  ",
            status,
            current_pos.as_secs() / 60,
            current_pos.as_secs() % 60,
            total_duration.as_secs() / 60,
            total_duration.as_secs() % 60
        )?;
        stdout.flush()?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                let mut seek_to: Option<Duration> = None;
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char(' ') => {
                        if sink.is_paused() {
                            sink.play();
                            last_update_time = Instant::now(); // Reset timer on play
                        } else {
                            sink.pause();
                            last_known_pos = current_pos; // Save position on pause
                        }
                    }
                    KeyCode::Left => {
                        seek_to = Some(current_pos.saturating_sub(Duration::from_secs(10)));
                    }
                    KeyCode::Right => {
                        seek_to = Some((current_pos + Duration::from_secs(10)).min(total_duration));
                    }
                    _ => {}
                }

                if let Some(seek_pos) = seek_to {
                    match sink.try_seek(seek_pos) {
                        Ok(_) => {
                            last_known_pos = seek_pos;
                            last_update_time = Instant::now();
                        }
                        Err(e) => {
                            // Can't use `?` here due to Sync trait bounds, handle explicitly
                            eprintln!("\nSeek failed: {:?}", e);
                        }
                    }
                }
            }
        }

        if sink.empty() {
            break;
        }
    }

    disable_raw_mode()?;
    println!("\nPlayback finished.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Audiothek Quickplay v{}", env!("CARGO_PKG_VERSION"));
    println!("----------------------------------");

    let mut exe_path = env::current_exe().context("Failed to get executable path")?;
    exe_path.pop(); // Remove the executable name to get the directory
    let podcast_path = exe_path.join("podcasts.json");

    let podcasts = load_podcasts(&podcast_path)
        .with_context(|| format!("Failed to load podcasts from {:?}", podcast_path))?;

    let podcast_keys: Vec<&str> = podcasts.iter().map(|p| p.key.as_str()).collect();

    let args: Vec<String> = env::args().collect();
    let podcast_key = match args.get(1) {
        Some(key) => key.clone(),
        None => {
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose a podcast to play")
                .items(&podcast_keys)
                .default(0)
                .interact_opt()
                .context("Menu interaction failed")?;

            match selection {
                Some(index) => podcasts[index].key.clone(),
                None => {
                    println!("No podcast selected.");
                    return Ok(());
                }
            }
        }
    };

    let selected_podcast = podcasts
        .iter()
        .find(|p| p.key == podcast_key)
        .context(format!("Unknown podcast key: '{}'", podcast_key))?;

    println!("Fetching latest episode for '{}'...", selected_podcast.key);
    let (title, audio_url) = get_latest_episode_url(&selected_podcast.id).await?;

    println!("Found episode: {}", title);
    println!("Loading. Please wait...");
    play_audio_interactive(&audio_url).await?;

    Ok(())
}
