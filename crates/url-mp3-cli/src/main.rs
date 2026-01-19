use anyhow::{Context, Result, bail};
use colored::Colorize;
use std::io::{self, Write};
use url_mp3_core::{download_mp3, ensure_binaries, get_default_output_dir, get_libs_dir};

#[tokio::main]
async fn main() -> Result<()> {
    let output_dir = get_default_output_dir();
    let libs_dir = get_libs_dir();

    // Ensure output directory exists
    std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    // Ensure binaries are available
    ensure_binaries(&libs_dir)
        .await
        .context("Failed to ensure binaries")?;

    // Prompt for URL
    print!("Enter YouTube URL: ");
    io::stdout().flush()?;

    let mut url = String::new();
    io::stdin().read_line(&mut url)?;
    let url = url.trim();

    if url.is_empty() {
        bail!("No URL provided");
    }

    println!("Downloading...");

    match download_mp3(&libs_dir, &output_dir, url) {
        Ok(result) => {
            println!("{}: {}", "Saved".green(), result.path.display());
        }
        Err(e) => {
            eprintln!("{}: {}", "Error".red(), e);
        }
    }

    Ok(())
}
