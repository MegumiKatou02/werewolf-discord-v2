use serenity::all::{ComponentInteraction, GuildId, ModalInteraction, UserId};
pub enum InteractionWrapper {
    Component(ComponentInteraction),
    Modal(ModalInteraction),
}

impl InteractionWrapper {
    pub fn user_id(&self) -> UserId {
        match self {
            Self::Component(i) => i.user.id,
            Self::Modal(i) => i.user.id,
        }
    }

    pub fn guild_id(&self) -> Option<GuildId> {
        match self {
            Self::Component(i) => i.guild_id,
            Self::Modal(i) => i.guild_id,
        }
    }
}
