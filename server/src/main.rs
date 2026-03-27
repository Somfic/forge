use cinema::CinemaModule;
use forge::{Config, Platform, Result};

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

    let config = Config::from_file("forge.toml")?;
    let dev = std::env::args().any(|a| a == "--dev");

    Platform::new(config, modules()).dev(dev).run().await
}
