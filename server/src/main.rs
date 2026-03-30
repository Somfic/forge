use std::path::PathBuf;

use clap::Parser;
use cinema::CinemaModule;
use forge::{Config, Platform, Result};

#[derive(Parser)]
#[command(name = "forge", about = "Forge media server")]
struct Cli {
    /// Host address to bind to
    #[arg(long, env = "FORGE_HOST")]
    host: Option<String>,

    /// Port to listen on
    #[arg(short, long, env = "FORGE_PORT")]
    port: Option<u16>,

    /// Path to data directory
    #[arg(long, env = "FORGE_DATA_DIR")]
    data_dir: Option<PathBuf>,

    /// Database URL (e.g. sqlite:./data.db, postgres://user:pass@host/db)
    #[arg(long, env = "FORGE_DATABASE_URL")]
    database_url: Option<String>,

    /// Path to config file
    #[arg(short, long, default_value = "forge.toml", env = "FORGE_CONFIG")]
    config: PathBuf,

    /// Run in development mode
    #[arg(long)]
    dev: bool,
}

fn modules() -> Vec<Box<dyn forge::Module>> {
    vec![Box::new(CinemaModule)]
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    match main_wrapper().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

async fn main_wrapper() -> Result<()> {
    // Raise the file descriptor limit for torrent peer connections + streaming
    #[cfg(unix)]
    {
        use std::io::Error;
        let mut rlim = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
        unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlim); }
        rlim.rlim_cur = rlim.rlim_max.min(10240);
        if unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) } != 0 {
            eprintln!("Warning: could not raise file descriptor limit: {}", Error::last_os_error());
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .event_format(forge::ForgeFormatter)
        .init();

    let cli = Cli::parse();

    let mut config = Config::from_file(&cli.config)?;

    if let Some(host) = cli.host {
        config.host = host;
    }
    if let Some(port) = cli.port {
        config.port = port;
    }
    if let Some(data_dir) = cli.data_dir {
        config.data_dir = data_dir;
    }
    if cli.database_url.is_some() {
        config.database_url = cli.database_url;
    }

    Platform::new(config, modules()).dev(cli.dev).run().await
}
