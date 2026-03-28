use outbox_rs::db::Db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");

    let db = Db::new(&database_url).await?;
    db.migrate().await?;

    Ok(())
}
