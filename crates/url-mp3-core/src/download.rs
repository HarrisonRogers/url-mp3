use crate::binaries::{get_ffmpeg_path, get_yt_dlp_path};
use crate::error::DownloadError;
use std::path::PathBuf;
use std::process::Command;

/// Check if a URL is a valid YouTube URL
pub fn is_youtube_url(url: &str) -> bool {
    url.contains("youtube.com") || url.contains("youtu.be")
}

/// Result of a successful download
#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub path: PathBuf,
    pub title: String,
}

/// Download a YouTube video as MP3
pub fn download_mp3(
    libs_dir: &PathBuf,
    output_dir: &PathBuf,
    url: &str,
) -> Result<DownloadResult, DownloadError> {
    if !is_youtube_url(url) {
        return Err(DownloadError::InvalidUrl(format!(
            "Not a valid YouTube URL: {}",
            url
        )));
    }

    let yt_dlp_path = get_yt_dlp_path(libs_dir);
    let ffmpeg_path = get_ffmpeg_path(libs_dir);

    if !yt_dlp_path.exists() {
        return Err(DownloadError::BinaryNotFound("yt-dlp".to_string()));
    }
    if !ffmpeg_path.exists() {
        return Err(DownloadError::BinaryNotFound("ffmpeg".to_string()));
    }

    let output_template = output_dir.join("%(title)s.%(ext)s");

    tracing::info!("Starting download for: {}", url);

    // Run yt-dlp with audio extraction
    let output = Command::new(&yt_dlp_path)
        .args([
            "-x",                                        // Extract audio
            "--audio-format",
            "mp3",                                       // Convert to MP3
            "--audio-quality",
            "320K",                                      // Best quality
            "--ffmpeg-location",
            ffmpeg_path.to_str().unwrap(),
            "-o",
            output_template.to_str().unwrap(),
            "--no-playlist",                             // Don't download playlists
            "--no-progress",                             // Clean output
            url,
        ])
        .output()
        .map_err(|e| DownloadError::DownloadFailed(format!("Failed to execute yt-dlp: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DownloadError::DownloadFailed(format!(
            "yt-dlp failed: {}",
            stderr
        )));
    }

    // Parse the output to find the actual filename
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Look for the destination line in yt-dlp output
    let (mp3_path, title) = if let Some(line) = stdout
        .lines()
        .find(|l| l.contains("Destination:") && l.contains(".mp3"))
    {
        let path_str = line.split("Destination:").nth(1).unwrap_or("").trim();
        let path = PathBuf::from(path_str);
        let title = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        (path, title)
    } else {
        // Fallback: try to find any mp3 file that was just created
        let title = stdout
            .lines()
            .find(|l| l.contains("[download]") && l.contains("Destination:"))
            .and_then(|l| l.split("Destination:").nth(1))
            .map(|s| s.trim())
            .unwrap_or("output");

        // Replace extension with mp3
        let base = PathBuf::from(title);
        let stem = base
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("output");
        (
            output_dir.join(format!("{}.mp3", stem)),
            stem.to_string(),
        )
    };

    tracing::info!("Download complete: {:?}", mp3_path);

    Ok(DownloadResult {
        path: mp3_path,
        title,
    })
}

/// Get the default output directory (~/Desktop/soundboard)
pub fn get_default_output_dir() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join("Desktop").join("soundboard"))
        .unwrap_or_else(|| PathBuf::from("."))
}
