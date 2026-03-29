use std::sync::Arc;

use outbox_rs::{db::Db, poller::PollerConfig, publisher::Publisher};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry().with(EnvFilter::from_default_env()).with(fmt::layer().json()).init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let amqp_url = std::env::var("AMQP_URL").expect("AMQP_URL must be set.");

    let publisher = Arc::new(Publisher::connect(&amqp_url, "events").await?);
    let db = Db::new(&database_url).await?;

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    //poller task
    let poller = Poller::new(db, PollerConfig::default());
    let poller_handle = tokio::spawn(async move {
        if let Err(e) = poller.run(publisher, shutdown_rx).await {
            tracing::error!(%e, "poller exited with error");
        }
    });

    // wait for SIGINT
    tokio::signal::ctrl_c().await?;
    tracing::info!("shutdown signal received");

    // broadcast shutdown 
    shutdown_tx.send(true)?;
    poller_handle.await?;

    Ok(())
}
