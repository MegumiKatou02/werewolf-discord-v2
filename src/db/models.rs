use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    #[serde(rename = "guildId")]
    pub guild_id: String,

    #[serde(rename = "wolfVoteTime")]
    pub wolf_vote_time: u64,

    #[serde(rename = "nightTime")]
    pub night_time: u64,

    #[serde(rename = "discussTime")]
    pub discuss_time: u64,

    #[serde(rename = "voteTime")]
    pub vote_time: u64,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            id: None,
            guild_id: String::new(),
            wolf_vote_time: 40,
            night_time: 70,
            discuss_time: 90,
            vote_time: 30,
        }
    }
}
