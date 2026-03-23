use forge_core::{Config, Platform, Result};

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
        .init();

    let config = Config::from_file("spine.toml")?;
    tracing::info!("loaded config from spine.toml");

    Platform::new(config).run().await
}
