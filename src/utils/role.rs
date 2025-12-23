use crate::types::Faction;
use serde::{Deserialize, Serialize};

pub fn convert_faction_role(role_id: i8) -> &'static str {
    match role_id {
        0 => Faction::Werewolf.name(),
        1 => Faction::Village.name(),
        2 => Faction::Solo.name(),
        3 => Faction::ViWolf.name(),
        _ => "Người Chết",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum RoleId {
    Werewolf = 0,
    Villager = 1,
    Bodyguard = 2,
    Cursed = 3,
    Seer = 4,
    Detective = 5,
    Witch = 6,
    Fool = 7,
    Medium = 8,
    Dead = 9,
    Maid = 10,
    Lycan = 11,
    WolfSeer = 12,
    AlphaWerewolf = 13,
    FoxSpirit = 14,
    Elder = 15,
    Stalker = 16,
    Gunner = 17,
    KittenWolf = 18,
    Puppeteer = 19,
    Voodoo = 20,
    Wolffluence = 21,
    Loudmouth = 22,
}

impl RoleId {
    pub fn name(&self) -> &'static str {
        match self {
            RoleId::Werewolf => "Ma Sói",
            RoleId::Villager => "Dân Làng",
            RoleId::Bodyguard => "Bảo Vệ",
            RoleId::Cursed => "Bán Sói",
            RoleId::Seer => "Tiên Tri",
            RoleId::Detective => "Thám Tử",
            RoleId::Witch => "Phù Thuỷ",
            RoleId::Fool => "Thằng Ngố",
            RoleId::Medium => "Thầy Đồng",
            RoleId::Dead => "Người Chết",
            RoleId::Maid => "Hầu Gái",
            RoleId::Lycan => "Lycan",
            RoleId::WolfSeer => "Sói Tiên Tri",
            RoleId::AlphaWerewolf => "Sói Trùm",
            RoleId::FoxSpirit => "Cáo",
            RoleId::Elder => "Già Làng",
            RoleId::Stalker => "Stalker",
            RoleId::Gunner => "Xạ Thủ",
            RoleId::KittenWolf => "Sói Mèo Con",
            RoleId::Puppeteer => "Người Múa Rối",
            RoleId::Voodoo => "Sói Tà Thuật",
            RoleId::Wolffluence => "Sói Thao Túng",
            RoleId::Loudmouth => "Cậu Bé Miệng Bự",
        }
    }

    pub fn from_u8(id: u8) -> Option<Self> {
        match id {
            0 => Some(RoleId::Werewolf),
            1 => Some(RoleId::Villager),
            2 => Some(RoleId::Bodyguard),
            3 => Some(RoleId::Cursed),
            4 => Some(RoleId::Seer),
            5 => Some(RoleId::Detective),
            6 => Some(RoleId::Witch),
            7 => Some(RoleId::Fool),
            8 => Some(RoleId::Medium),
            9 => Some(RoleId::Dead),
            10 => Some(RoleId::Maid),
            11 => Some(RoleId::Lycan),
            12 => Some(RoleId::WolfSeer),
            13 => Some(RoleId::AlphaWerewolf),
            14 => Some(RoleId::FoxSpirit),
            15 => Some(RoleId::Elder),
            16 => Some(RoleId::Stalker),
            17 => Some(RoleId::Gunner),
            18 => Some(RoleId::KittenWolf),
            19 => Some(RoleId::Puppeteer),
            20 => Some(RoleId::Voodoo),
            21 => Some(RoleId::Wolffluence),
            22 => Some(RoleId::Loudmouth),
            _ => None,
        }
    }
}

pub fn get_role_table(players: u32) -> Option<&'static [(RoleId, u32)]> {
    match players {
        4 => Some(&[
            (RoleId::Werewolf, 1),
            (RoleId::Villager, 2),
            (RoleId::Bodyguard, 1),
        ]),
        5 => Some(&[
            (RoleId::Werewolf, 1),
            (RoleId::Villager, 2),
            (RoleId::Bodyguard, 1),
            (RoleId::Witch, 1),
        ]),
        6 => Some(&[
            (RoleId::Werewolf, 2),
            (RoleId::Villager, 1),
            (RoleId::Bodyguard, 1),
            (RoleId::Witch, 1),
            (RoleId::Medium, 1),
        ]),
        7 => Some(&[
            (RoleId::Werewolf, 2),
            (RoleId::Villager, 1),
            (RoleId::Bodyguard, 1),
            (RoleId::Witch, 1),
            (RoleId::Medium, 1),
            (RoleId::Detective, 1),
        ]),
        8 => Some(&[
            (RoleId::Werewolf, 2),
            (RoleId::Villager, 1),
            (RoleId::Bodyguard, 1),
            (RoleId::Witch, 1),
            (RoleId::Medium, 1),
            (RoleId::Detective, 1),
            (RoleId::Cursed, 1),
        ]),
        9 => Some(&[
            (RoleId::Werewolf, 2),
            (RoleId::Villager, 2),
            (RoleId::Bodyguard, 1),
            (RoleId::Witch, 1),
            (RoleId::Medium, 1),
            (RoleId::Detective, 1),
            (RoleId::Cursed, 1),
        ]),
        10 => Some(&[
            (RoleId::Werewolf, 3),
            (RoleId::Villager, 2),
            (RoleId::Bodyguard, 1),
            (RoleId::Witch, 1),
            (RoleId::Detective, 1),
            (RoleId::Cursed, 1),
            (RoleId::Seer, 1),
        ]),
        11 => Some(&[
            (RoleId::Werewolf, 3),
            (RoleId::Villager, 2),
            (RoleId::Bodyguard, 1),
            (RoleId::Witch, 1),
            (RoleId::Detective, 1),
            (RoleId::Cursed, 1),
            (RoleId::Seer, 1),
            (RoleId::Fool, 1),
        ]),
        12 => Some(&[
            (RoleId::Werewolf, 3),
            (RoleId::Villager, 3),
            (RoleId::Bodyguard, 1),
            (RoleId::Witch, 1),
            (RoleId::Detective, 1),
            (RoleId::Cursed, 1),
            (RoleId::Seer, 1),
            (RoleId::Fool, 1),
        ]),
        _ => None,
    }
}
