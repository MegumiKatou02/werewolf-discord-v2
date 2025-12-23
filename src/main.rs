mod bot;
mod client;
mod constants;
mod db;
mod game;
mod roles;
mod types;
mod utils;

mod commands;

use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Werewolf Discord Bot (Rust)");

    dotenv::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");

    let mongo_uri = env::var("MONGODB_URI").expect("Expected MONGODB_URI in environment");

    let db = db::connect_mongodb(&mongo_uri).await?;

    tracing::info!("MongoDB connected successfully");

    tracing::info!("Starting Discord bot...");
    bot::start_bot(&token, db).await?;

    Ok(())
}
