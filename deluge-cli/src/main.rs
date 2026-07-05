use crate::run::run;
use std::process;

mod cli_config;
mod commands;
mod helpers;
mod record;
mod run;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    if let Err(e) = run().await {
        tracing::error!("error: {e:#}");
        process::exit(1);
    }
}
