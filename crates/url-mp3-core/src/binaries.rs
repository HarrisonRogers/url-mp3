use crate::error::DownloadError;
use std::path::PathBuf;
use yt_dlp::Youtube;

/// Get the default directory for storing yt-dlp and ffmpeg binaries
pub fn get_libs_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("url-mp3")
        .join("libs")
}

/// Ensure yt-dlp and ffmpeg binaries are available
pub async fn ensure_binaries(libs_dir: &PathBuf) -> Result<(), DownloadError> {
    std::fs::create_dir_all(libs_dir).map_err(|e| {
        DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create libs directory: {}", e),
        ))
    })?;

    let yt_dlp_path = libs_dir.join("yt-dlp");
    let ffmpeg_path = libs_dir.join("ffmpeg");

    // Check if binaries already exist
    if yt_dlp_path.exists() && ffmpeg_path.exists() {
        return Ok(());
    }

    tracing::info!("Downloading yt-dlp and ffmpeg (first run only)...");

    // Use a temp directory for the Youtube client since we just need to trigger binary download
    let temp_dir = std::env::temp_dir().join("url-mp3-init");
    std::fs::create_dir_all(&temp_dir)?;

    // This will download the binaries
    Youtube::with_new_binaries(libs_dir, &temp_dir)
        .await
        .map_err(|e| DownloadError::BinaryInit(e.to_string()))?;

    Ok(())
}

/// Get the path to the yt-dlp binary
pub fn get_yt_dlp_path(libs_dir: &PathBuf) -> PathBuf {
    libs_dir.join("yt-dlp")
}

/// Get the path to the ffmpeg binary
pub fn get_ffmpeg_path(libs_dir: &PathBuf) -> PathBuf {
    libs_dir.join("ffmpeg")
}
