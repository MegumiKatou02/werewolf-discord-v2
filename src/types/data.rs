use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone, Debug)]
pub struct RoleInfo {
    pub title: String,
    #[serde(rename = "eName")]
    pub e_name: String,
    pub description: String,
    pub faction: i8,
}

pub type RolesData = HashMap<String, RoleInfo>;
