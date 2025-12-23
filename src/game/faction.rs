use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactionRole {
    Village,
    Werewolf,
    ViWolf,
    Solo,
}

impl fmt::Display for FactionRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            FactionRole::Village => "Dân Làng",
            FactionRole::Werewolf => "Ma Sói",
            FactionRole::ViWolf => "Dân Làng hoặc Ma Sói",
            FactionRole::Solo => "Solo",
        };
        write!(f, "{}", text)
    }
}
