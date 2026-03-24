use forge::{Config, Platform, Result};
use movies::MoviesModule;

fn modules() -> Vec<Box<dyn forge::Module>> {
    vec![Box::new(MoviesModule)]
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
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .event_format(forge::ForgeFormatter)
        .init();

    let config = Config::from_file("spine.toml")?;

    Platform::new(config, modules()).run().await
}
