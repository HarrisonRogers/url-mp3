use yt_dlp::Youtube;
use std::path::PathBuf;
use yt_dlp::client::deps::Libraries;
use std::io;
use dialoguer::Select;

fn get_default_output_dir(download_type: &str) -> PathBuf {
    if download_type == "audio" {
        return dirs::home_dir()
            .map(|h| h.join("Desktop").join("soundboard").join("sound-effects"))
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        return dirs::home_dir()
            .map(|h| h.join("Desktop").join("soundboard"))
            .unwrap_or_else(|| PathBuf::from("."))
    }    
}

fn get_libs_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("url-mp3")
        .join("libs")
}

fn is_youtube_url(url: &str) -> bool {
    url.contains("youtube.com") || url.contains("youtu.be")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Terminal prompt for download type
    let items = vec!["audio", "video"];
    let selection = Select::new()
        .with_prompt("What do you want to download?")
        .default(0)
        .items(&items)
        .interact()
        .expect("No input provided");
    let download_type = items[selection];

    // Terminal prompt for YouTube URL
    println!("Enter YouTube URL: ");
    let mut url = String::new();
    io::stdin().read_line(&mut url).expect("No input provided");
    let url = url.trim().to_string();
    println!("Fetching video details...");
    if !is_youtube_url(&url) {
        println!("Not a valid YouTube URL");
        return Ok(());
    }

    // Initialize libraries and directories
    let libraries_dir = PathBuf::from(get_libs_dir());
    let output_dir = PathBuf::from(get_default_output_dir(&download_type));
    
    let youtube = libraries_dir.join("yt-dlp");
    let ffmpeg = libraries_dir.join("ffmpeg");
    
    let libraries = Libraries::new(youtube, ffmpeg);
    let mut fetcher = Youtube::new(libraries, output_dir).await?;
    fetcher.cache = None;
    fetcher.download_cache = None;
    fetcher.playlist_cache = None;


    let video = fetcher.fetch_video_infos(url.clone()).await?;

    // Download audio or video logic
    if download_type == "audio" {
        println!("Downloading audio...");
        let file_name = format!("{}.mp3", video.title);
        let audio_path = fetcher.download_audio_stream(&video, file_name).await?;
        println!("Downloaded audio to {}", audio_path.display());
    } else {
        println!("Downloading video...");
        let file_name = format!("{}.mp4", video.title);
        let video_path = fetcher.download_video_from_url(url, file_name).await?;
        println!("Downloaded video to {}", video_path.display());
    }
    Ok(())
}