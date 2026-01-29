use yt_dlp::Youtube;
use std::path::PathBuf;
use std::io;
use dialoguer::Select;
use yt_dlp::client::deps::Libraries;

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
    let libraries_dir = PathBuf::from("libs");
    let output_dir = PathBuf::from(get_default_output_dir(&download_type));

    let youtube = libraries_dir.join("yt-dlp");
    let ffmpeg = libraries_dir.join("ffmpeg");
    
    let libraries = Libraries::new(youtube, ffmpeg);
    let fetcher = Youtube::new(libraries, output_dir).await?;

    let video = fetcher.fetch_video_infos(url.clone()).await?;
    fetcher.update_downloader().await?;

    // Download audio or video logic
    if download_type == "audio" {
        println!("Downloading audio...");
        let video_format = video.best_video_format().unwrap();
        let format_path = fetcher.download_format(&video_format, format!("{}.mp3", video.title)).await?;
        println!("Downloaded audio to {}", format_path.display());
    } else {
        println!("Downloading video...");
        let video_format = video.best_video_format().unwrap();
        let video_path = fetcher.download_format(&video_format, format!("{}.mp4", video.title)).await?;
        println!("Downloaded video to {}", video_path.display());
    }
    Ok(())
}