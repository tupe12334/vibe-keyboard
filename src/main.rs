mod actions;
mod config;
mod daemon;
mod device;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "vibe-keyboard", about = "Ajazz Stream Dock controller", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the daemon — connect to device, load config, watch for changes
    Run {
        /// Path to the TOML config file
        #[arg(short, long, default_value = "config.toml")]
        config: PathBuf,
    },
    /// List connected Ajazz devices
    Devices,
    /// Set display brightness (0–100) without a full config
    SetBrightness {
        /// Brightness level 0–100
        value: u8,
        /// Device serial number (uses first device if omitted)
        #[arg(short, long)]
        serial: Option<String>,
    },
    /// Set an image on a single button without a full config
    SetImage {
        /// 0-based button index
        key: u8,
        /// Path to an image file (JPEG / PNG / BMP …)
        image: PathBuf,
        /// Device serial number (uses first device if omitted)
        #[arg(short, long)]
        serial: Option<String>,
    },
    /// Write a custom boot logo to the device
    SetLogo {
        /// Path to an image file
        image: PathBuf,
        /// Device serial number (uses first device if omitted)
        #[arg(short, long)]
        serial: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("vibe_keyboard=info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { config } => daemon::run(config).await?,
        Commands::Devices => device::list()?,
        Commands::SetBrightness { value, serial } => device::quick_brightness(value, serial)?,
        Commands::SetImage { key, image, serial } => device::quick_image(key, image, serial)?,
        Commands::SetLogo { image, serial } => device::quick_logo(image, serial)?,
    }

    Ok(())
}
