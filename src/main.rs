// use anyhow::{Context, Result, bail};
// use colored::Colorize;
// use std::io::{self, Write};
// use std::path::PathBuf;
// use std::process::Command;
// use yt_dlp::Youtube;

fn get_default_output_dir() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join("Desktop").join("soundboard"))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn get_libs_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("url-mp3")
        .join("libs")
}

// async fn ensure_binaries() -> Result<PathBuf> {
//     let libs_dir = get_libs_dir();
//     std::fs::create_dir_all(&libs_dir)
//         .context("Failed to create libs directory")?;

//     let yt_dlp_path = libs_dir.join("yt-dlp");
//     let ffmpeg_path = libs_dir.join("ffmpeg");

//     // Check if binaries already exist
//     if yt_dlp_path.exists() && ffmpeg_path.exists() {
//         return Ok(libs_dir);
//     }

//     println!("Downloading yt-dlp and ffmpeg (first run only)...");

//     // Use a temp directory for the Youtube client since we just need to trigger binary download
//     let temp_dir = std::env::temp_dir().join("url-mp3-init");
//     std::fs::create_dir_all(&temp_dir)?;

//     // This will download the binaries
//     Youtube::with_new_binaries(&libs_dir, &temp_dir)
//         .await
//         .context("Failed to download yt-dlp/ffmpeg binaries")?;

//     Ok(libs_dir)
// }

// fn is_youtube_url(url: &str) -> bool {
//     url.contains("youtube.com") || url.contains("youtu.be")
// }

// fn download_mp3(
//     libs_dir: &PathBuf,
//     output_dir: &PathBuf,
//     url: &str,
// ) -> Result<PathBuf> {
//     if !is_youtube_url(url) {
//         bail!("Not a valid YouTube URL: {}", url);
//     }

//     let yt_dlp_path = libs_dir.join("yt-dlp");
//     let ffmpeg_path = libs_dir.join("ffmpeg");

//     let output_template = output_dir.join("%(title)s.%(ext)s");

//     println!("Downloading...");

//     // Run yt-dlp with audio extraction
//     let output = Command::new(&yt_dlp_path)
//         .args([
//             "-x",                          // Extract audio
//             "--audio-format", "mp3",       // Convert to MP3
//             "--audio-quality", "320K",     // Best quality
//             "--ffmpeg-location", ffmpeg_path.to_str().unwrap(),
//             "-o", output_template.to_str().unwrap(),
//             "--no-playlist",               // Don't download playlists
//             "--no-progress",               // Clean output
//             url,
//         ])
//         .output()
//         .context("Failed to execute yt-dlp")?;

//     if !output.status.success() {
//         let stderr = String::from_utf8_lossy(&output.stderr);
//         bail!("yt-dlp failed: {}", stderr);
//     }

//     // Parse the output to find the actual filename
//     let stdout = String::from_utf8_lossy(&output.stdout);

//     // Look for the destination line in yt-dlp output
//     let mp3_path = if let Some(line) = stdout.lines().find(|l| l.contains("Destination:") && l.contains(".mp3")) {
//         let path_str = line.split("Destination:").nth(1).unwrap_or("").trim();
//         PathBuf::from(path_str)
//     } else {
//         // Fallback: try to find any mp3 file that was just created
//         let title = stdout.lines()
//             .find(|l| l.contains("[download]") && l.contains("Destination:"))
//             .and_then(|l| l.split("Destination:").nth(1))
//             .map(|s| s.trim())
//             .unwrap_or("output");

//         // Replace extension with mp3
//         let base = PathBuf::from(title);
//         let stem = base.file_stem().unwrap_or_default().to_str().unwrap_or("output");
//         output_dir.join(format!("{}.mp3", stem))
//     };

//     Ok(mp3_path)
// }

// #[tokio::main]
// async fn main() -> Result<()> {
//     let output_dir = get_default_output_dir();

//     // Ensure output directory exists
//     std::fs::create_dir_all(&output_dir)
//         .context("Failed to create output directory")?;

//     // Ensure binaries are available
//     let libs_dir = ensure_binaries().await?;

//     // Prompt for URL
//     print!("Enter YouTube URL: ");
//     io::stdout().flush()?;

//     let mut url = String::new();
//     io::stdin().read_line(&mut url)?;
//     let url = url.trim();

//     if url.is_empty() {
//         bail!("No URL provided");
//     }

//     match download_mp3(&libs_dir, &output_dir, url) {
//         Ok(path) => {
//             println!("{}: {}", "Saved".green(), path.display());
//         }
//         Err(e) => {
//             eprintln!("{}: {}", "Error".red(), e);
//         }
//     }

//     Ok(())
// }

//  --- Second iteration
// use yt_dlp::Youtube;
// use std::path::PathBuf;
// use yt_dlp::client::deps::Libraries;
// use std::io::{self};

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let libraries_dir = PathBuf::from("libs");
//     let output_dir = PathBuf::from("output");

//     let youtube = libraries_dir.join("yt-dlp");
//     let ffmpeg = libraries_dir.join("ffmpeg");

//     let libraries = Libraries::new(youtube, ffmpeg);
//     let fetcher = Youtube::new(libraries, output_dir).await?;

//     println!("Enter YouTube URL: ");
//     let mut url = String::new();
//     io::stdin().read_line(&mut url).expect("No input provided");
//     let url = url.trim().to_string();

//     println!("{}", url);

//     println!("Downloading audio and video...");

//     let video = fetcher.fetch_video_infos(url).await?;
//     let video_format = video.best_video_format().unwrap();

//     let audio_format = video.worst_audio_format().unwrap();
//     let audio_path = fetcher.download_format(&audio_format, &video.title).await?;

//     println!("Downloaded audio to {}", audio_path.display());
//     println!("Downloaded video to {:?}", video_format.download_info.url);

//     Ok(())
// }

// --- Third iteration
use yt_dlp::Youtube;
use std::path::PathBuf;
use yt_dlp::client::deps::Libraries;
use std::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let libraries_dir = PathBuf::from(get_libs_dir());
    let output_dir = PathBuf::from(get_default_output_dir());

    println!("outpuat {:?}", output_dir);
    
    let youtube = libraries_dir.join("yt-dlp");
    let ffmpeg = libraries_dir.join("ffmpeg");
    
    let libraries = Libraries::new(youtube, ffmpeg);
    let fetcher = Youtube::new(libraries, output_dir).await?;

    println!("Enter YouTube URL: ");
    let mut url = String::new();
    io::stdin().read_line(&mut url).expect("No input provided");
    let url = url.trim().to_string();
    println!("Fetching video details...");
    let video = fetcher.fetch_video_infos(url.clone()).await?;
    println!("{}", video.title);


    let video_path = fetcher.download_video_from_url(url, video.title).await?;
    println!("Downloaded video to {}", video_path.display());
    Ok(())
}