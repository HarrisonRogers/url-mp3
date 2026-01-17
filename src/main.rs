use anyhow::{Context, Result, bail};
use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;
use yt_dlp::Youtube;

#[derive(Parser, Debug)]
#[command(name = "url-mp3")]
#[command(version, about = "Convert YouTube URLs to MP3 files")]
struct Args {
    /// YouTube URLs to download
    #[arg(required = true)]
    urls: Vec<String>,

    /// Output directory for MP3 files
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Audio quality: best, high, medium, low
    #[arg(short, long, default_value = "best")]
    quality: String,

    /// Custom filename (without extension)
    #[arg(short, long)]
    filename: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn get_default_output_dir() -> PathBuf {
    dirs::audio_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .map(|h| h.join("Music"))
            .unwrap_or_else(|| PathBuf::from("."))
    })
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

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if "/:*?\"<>|\\".contains(c) { '_' } else { c })
        .collect()
}

fn is_youtube_url(url: &str) -> bool {
    url.contains("youtube.com") || url.contains("youtu.be")
}

fn quality_to_bitrate(quality: &str) -> &str {
    match quality.to_lowercase().as_str() {
        "best" => "320",
        "high" => "256",
        "medium" => "192",
        "low" => "128",
        _ => "320",
    }
}

fn download_mp3(
    libs_dir: &PathBuf,
    output_dir: &PathBuf,
    url: &str,
    args: &Args,
) -> Result<PathBuf> {
    if !is_youtube_url(url) {
        bail!("Not a valid YouTube URL: {}", url);
    }

    let yt_dlp_path = libs_dir.join("yt-dlp");
    let ffmpeg_path = libs_dir.join("ffmpeg");

    // Build output template
    let output_template = if let Some(filename) = &args.filename {
        output_dir.join(format!("{}.%(ext)s", sanitize_filename(filename)))
    } else {
        output_dir.join("%(title)s.%(ext)s")
    };

    let bitrate = quality_to_bitrate(&args.quality);
    let audio_quality = format!("{}K", bitrate);

    if args.verbose {
        println!("  Downloading and converting to MP3...");
        println!("  Quality: {} kbps", bitrate);
    }

    // Run yt-dlp with audio extraction
    let output = Command::new(&yt_dlp_path)
        .args([
            "-x",                          // Extract audio
            "--audio-format", "mp3",       // Convert to MP3
            "--audio-quality", &audio_quality,
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

    if args.verbose {
        println!("  Output: {}", mp3_path.display());
    }

    Ok(mp3_path)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let output_dir = args.output.clone().unwrap_or_else(get_default_output_dir);

    println!("Output directory: {}", output_dir.display());

    // Ensure output directory exists
    std::fs::create_dir_all(&output_dir)
        .context("Failed to create output directory")?;

    // Ensure binaries are available
    let libs_dir = ensure_binaries().await?;

    let total = args.urls.len();
    let mut success_count = 0;

    for (i, url) in args.urls.iter().enumerate() {
        println!("\n[{}/{}] Processing: {}", i + 1, total, url);

        match download_mp3(&libs_dir, &output_dir, url, &args) {
            Ok(path) => {
                println!("{}: {}", "Saved".green(), path.display());
                success_count += 1;
            }
            Err(e) => {
                eprintln!("{}: {}", "Error".red(), e);
                continue;
            }
        }
    }

    println!(
        "\n{} Downloaded {}/{} file(s) to {}",
        "Done!".green(),
        success_count,
        total,
        output_dir.display()
    );

    Ok(())
}
