pub mod handler;

use mongodb::Database;
use serenity::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

use crate::bot::handler::Handler;
use crate::game::room::PlayerRegistry;
use crate::game::RoomRegistry;
use crate::types::data::RolesData;

pub struct BotData {
    pub room_registry: RoomRegistry,
    pub db: Database,
    pub roles_json: RolesData,
    pub player_registry: PlayerRegistry,
}

pub async fn start_bot(token: &str, db: Database) -> anyhow::Result<()> {
    let rooms = Arc::new(RwLock::new(std::collections::HashMap::new()));
    let players = Arc::new(RwLock::new(std::collections::HashMap::new()));

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("data.json");
    let file = File::open(path).expect("Không tìm thấy file data/data.json");
    let reader = BufReader::new(file);

    let roles_json: RolesData = serde_json::from_reader(reader).expect("Lỗi định dạng file JSON");

    let data = Arc::new(BotData {
        room_registry: rooms.clone(),
        player_registry: players.clone(),
        db,
        roles_json,
    });

    let handler = Handler { data };

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .await?;

    client.start().await?;

    Ok(())
}
