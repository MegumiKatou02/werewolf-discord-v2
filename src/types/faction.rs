use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Faction {
    Werewolf = 0,
    Village = 1,
    Solo = 2,
    ViWolf = 3,
}

impl Faction {
    pub fn name(&self) -> &'static str {
        match self {
            Faction::Werewolf => "Ma Sói",
            Faction::Village => "Dân Làng",
            Faction::Solo => "Solo",
            Faction::ViWolf => "Dân Làng hoặc Ma Sói",
        }
    }
}
