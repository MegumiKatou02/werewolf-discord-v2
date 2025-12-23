pub mod models;

use anyhow::Result;
use mongodb::{Client, Database};

pub async fn connect_mongodb(uri: &str) -> Result<Database> {
    let client = Client::with_uri_str(uri).await?;
    let db = client.database("werewolf_discord");

    tracing::info!("MongoDB connected successfully");

    Ok(db)
}

pub use models::*;
