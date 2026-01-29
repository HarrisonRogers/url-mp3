use anyhow::{Context, Result, bail};
use colored::Colorize;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use yt_dlp::Youtube;
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

async fn ensure_binaries() -> Result<PathBuf> {
    let libs_dir = get_libs_dir();
    std::fs::create_dir_all(&libs_dir)
        .context("Failed to create libs directory")?;

    let yt_dlp_path = libs_dir.join("yt-dlp");
    let ffmpeg_path = libs_dir.join("ffmpeg");

    // Check if binaries already exist
    if yt_dlp_path.exists() && ffmpeg_path.exists() {
        return Ok(libs_dir);
    }

    println!("Downloading yt-dlp and ffmpeg (first run only)...");

    // Use a temp directory for the Youtube client since we just need to trigger binary download
    let temp_dir = std::env::temp_dir().join("url-mp3-init");
    std::fs::create_dir_all(&temp_dir)?;

    // This will download the binaries
    Youtube::with_new_binaries(&libs_dir, &temp_dir)
        .await
        .context("Failed to download yt-dlp/ffmpeg binaries")?;

    Ok(libs_dir)
}

fn is_youtube_url(url: &str) -> bool {
    url.contains("youtube.com") || url.contains("youtu.be")
}

fn download_mp3(
    libs_dir: &PathBuf,
    output_dir: &PathBuf,
    url: &str,
) -> Result<PathBuf> {
    if !is_youtube_url(url) {
        bail!("Not a valid YouTube URL: {}", url);
    }

    let yt_dlp_path = libs_dir.join("yt-dlp");
    let ffmpeg_path = libs_dir.join("ffmpeg");

    let output_template = output_dir.join("%(title)s.%(ext)s");

    println!("Downloading...");

    // Run yt-dlp with audio extraction
    let output = Command::new(&yt_dlp_path)
        .args([
            "-x",                          // Extract audio
            "--audio-format", "mp3",       // Convert to MP3
            "--audio-quality", "320K",     // Best quality
            "--ffmpeg-location", ffmpeg_path.to_str().unwrap(),
            "-o", output_template.to_str().unwrap(),
            "--no-playlist",               // Don't download playlists
            "--no-progress",               // Clean output
            url,
        ])
        .output()
        .context("Failed to execute yt-dlp")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("yt-dlp failed: {}", stderr);
    }

    // Parse the output to find the actual filename
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Look for the destination line in yt-dlp output
    let mp3_path = if let Some(line) = stdout.lines().find(|l| l.contains("Destination:") && l.contains(".mp3")) {
        let path_str = line.split("Destination:").nth(1).unwrap_or("").trim();
        PathBuf::from(path_str)
    } else {
        // Fallback: try to find any mp3 file that was just created
        let title = stdout.lines()
            .find(|l| l.contains("[download]") && l.contains("Destination:"))
            .and_then(|l| l.split("Destination:").nth(1))
            .map(|s| s.trim())
            .unwrap_or("output");

        // Replace extension with mp3
        let base = PathBuf::from(title);
        let stem = base.file_stem().unwrap_or_default().to_str().unwrap_or("output");
        output_dir.join(format!("{}.mp3", stem))
    };

    Ok(mp3_path)
}

fn download_mp4(
    libs_dir: &PathBuf,
    output_dir: &PathBuf,
    url: &str,
) -> Result<PathBuf> {
    if !is_youtube_url(url) {
        bail!("Not a valid YouTube URL: {}", url);
    }

    let yt_dlp_path = libs_dir.join("yt-dlp");
    let ffmpeg_path = libs_dir.join("ffmpeg");

    let output_template = output_dir.join("%(title)s.%(ext)s");

    println!("Downloading...");

    // Run yt-dlp for video download
    let output = Command::new(&yt_dlp_path)
        .args([
            "-f", "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best",
            "--merge-output-format", "mp4",
            "--ffmpeg-location", ffmpeg_path.to_str().unwrap(),
            "-o", output_template.to_str().unwrap(),
            "--no-playlist",
            "--no-progress",
            url,
        ])
        .output()
        .context("Failed to execute yt-dlp")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("yt-dlp failed: {}", stderr);
    }

    // Parse the output to find the actual filename
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Look for the destination line in yt-dlp output
    let mp4_path = if let Some(line) = stdout.lines().find(|l| l.contains("Destination:") && l.contains(".mp4")) {
        let path_str = line.split("Destination:").nth(1).unwrap_or("").trim();
        PathBuf::from(path_str)
    } else if let Some(line) = stdout.lines().find(|l| l.contains("[Merger]") && l.contains(".mp4")) {
        // Sometimes yt-dlp shows the merged output path differently
        let path_str = line.split("Merging formats into").nth(1)
            .or_else(|| line.split("\"").nth(1))
            .unwrap_or("")
            .trim()
            .trim_matches('"');
        PathBuf::from(path_str)
    } else {
        // Fallback: construct path from title
        let title = stdout.lines()
            .find(|l| l.contains("[download]") && l.contains("Destination:"))
            .and_then(|l| l.split("Destination:").nth(1))
            .map(|s| s.trim())
            .unwrap_or("output");

        let base = PathBuf::from(title);
        let stem = base.file_stem().unwrap_or_default().to_str().unwrap_or("output");
        output_dir.join(format!("{}.mp4", stem))
    };

    Ok(mp4_path)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Terminal prompt for download type
    let items = vec!["audio", "video"];
    let selection = Select::new()
        .with_prompt("What do you want to download?")
        .default(0)
        .items(&items)
        .interact()
        .expect("No input provided");
    let download_type = items[selection];

    let output_dir = get_default_output_dir(download_type);


    // Ensure output directory exists
    std::fs::create_dir_all(&output_dir)
        .context("Failed to create output directory")?;

    // Ensure binaries are available
    let libs_dir = ensure_binaries().await?;

    // Prompt for URL
    print!("Enter YouTube URL: ");
    io::stdout().flush()?;

    let mut url = String::new();
    io::stdin().read_line(&mut url)?;
    let url = url.trim();

    if url.is_empty() {
        bail!("No URL provided");
    }

    let result = if download_type == "audio" {
        download_mp3(&libs_dir, &output_dir, url)
    } else {
        download_mp4(&libs_dir, &output_dir, url)
    };

    match result {
        Ok(path) => {
            println!("{}: {}", "Saved".green(), path.display());
        }
        Err(e) => {
            eprintln!("{}: {}", "Error".red(), e);
        }
    }

    Ok(())
}
