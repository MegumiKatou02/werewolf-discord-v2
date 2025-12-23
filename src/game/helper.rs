use crate::roles::*;
use crate::types::{Faction, Player};
use crate::utils::role::RoleId;
use serenity::model::id::UserId;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VictoryResult {
    pub winner: Winner,
    pub faction: Faction,
}

#[derive(Debug, Clone)]
pub enum Winner {
    Werewolf,
    Village,
    Solo,
}

pub fn check_victory(players: &[Player]) -> Option<VictoryResult> {
    let alive_players: Vec<&Player> = players.iter().filter(|p| p.alive).collect();
    let alive_wolves: Vec<&Player> = alive_players
        .iter()
        .filter(|p| p.role.faction() == Faction::Werewolf)
        .copied()
        .collect();
    let alive_solos: Vec<&Player> = alive_players
        .iter()
        .filter(|p| p.role.faction() == Faction::Solo)
        .copied()
        .collect();

    if alive_players.len() == alive_solos.len() && !alive_solos.is_empty() {
        return Some(VictoryResult {
            winner: Winner::Solo,
            faction: Faction::Solo,
        });
    }

    if alive_wolves.is_empty() {
        return Some(VictoryResult {
            winner: Winner::Village,
            faction: Faction::Village,
        });
    }

    let non_wolves = alive_players.len() - alive_wolves.len();
    if alive_wolves.len() >= non_wolves {
        return Some(VictoryResult {
            winner: Winner::Werewolf,
            faction: Faction::Werewolf,
        });
    }

    None
}

pub fn process_vote(players: &mut [Player]) -> Option<(UserId, usize)> {
    let mut fluence_player_id: Option<UserId> = None;

    for player in players.iter() {
        if let Some(wolffluence) = player.role.clone_box().downcast_ref::<Wolffluence>() {
            if player.alive && wolffluence.influence_player.is_some() {
                fluence_player_id = wolffluence.influence_player;
                break;
            }
        }
    }

    let mut total_votes: HashMap<String, usize> = HashMap::new();

    for player in players.iter() {
        if !player.alive {
            continue;
        }

        if let Some(voted) = player.role.vote_hanged() {
            if voted == "skip" {
                continue;
            }

            let is_wolffluence = matches!(player.role.id(), RoleId::Wolffluence);

            let is_influenced = fluence_player_id.map_or(false, |id| id == player.user_id);

            if is_wolffluence && fluence_player_id.is_some() {
                *total_votes.entry(voted.clone()).or_insert(0) += 2;
            } else if is_influenced {
                *total_votes.entry(voted).or_insert(0) += 0;
            } else {
                *total_votes.entry(voted).or_insert(0) += 1;
            }
        }
    }

    if total_votes.is_empty() {
        return None;
    }

    let mut max_votes = 0;
    let mut candidates: Vec<String> = Vec::new();

    for (user_id_str, count) in total_votes.iter() {
        if *count > max_votes {
            max_votes = *count;
            candidates = vec![user_id_str.clone()];
        } else if *count == max_votes {
            candidates.push(user_id_str.clone());
        }
    }

    if candidates.len() == 1 && max_votes >= 2 {
        if let Ok(user_id) = candidates[0].parse::<u64>() {
            return Some((UserId::new(user_id), max_votes));
        }
    }

    None
}

pub fn total_voted_wolves_solve(players: &[Player]) -> Option<UserId> {
    let mut total_votes: HashMap<UserId, usize> = HashMap::new();

    for player in players.iter() {
        if player.role.faction() != Faction::Werewolf {
            continue;
        }

        if let Some(werewolf) = player.role.clone_box().downcast_ref::<Werewolf>() {
            if let Some(target) = werewolf.vote_bite {
                *total_votes.entry(target).or_insert(0) += 1;
            }
        }
    }

    if total_votes.is_empty() {
        return None;
    }

    let mut max_votes = 0;
    let mut candidates: Vec<UserId> = Vec::new();

    for (user_id, count) in total_votes.iter() {
        if *count > max_votes {
            max_votes = *count;
            candidates = vec![*user_id];
        } else if *count == max_votes {
            candidates.push(*user_id);
        }
    }

    if candidates.len() == 1 {
        return Some(candidates[0]);
    }

    None
}

pub fn is_activity(players: &[Player], role_id: RoleId) -> bool {
    for player in players.iter() {
        if player.role.id() != role_id {
            continue;
        }

        match role_id {
            RoleId::Seer => {
                if let Some(seer) = player.role.clone_box().downcast_ref::<Seer>() {
                    return seer.view_count < 1;
                }
            }
            RoleId::Bodyguard => {
                if let Some(bodyguard) = player.role.clone_box().downcast_ref::<Bodyguard>() {
                    return bodyguard.protected_person.is_some();
                }
            }
            RoleId::AlphaWerewolf => {
                if let Some(alpha) = player.role.clone_box().downcast_ref::<AlphaWerewolf>() {
                    return alpha.mask_wolf.is_some();
                }
            }
            RoleId::FoxSpirit => {
                if let Some(fox) = player.role.clone_box().downcast_ref::<FoxSpirit>() {
                    return !fox.three_viewed.is_empty();
                }
            }
            RoleId::Puppeteer => {
                if let Some(puppet) = player.role.clone_box().downcast_ref::<Puppeteer>() {
                    return puppet.target_wolf.is_some();
                }
            }
            RoleId::Voodoo => {
                if let Some(voodoo) = player.role.clone_box().downcast_ref::<VoodooWerewolf>() {
                    return voodoo.silent_player.is_some() || voodoo.voodoo_player.is_some();
                }
            }
            RoleId::Wolffluence => {
                if let Some(wolf_flu) = player.role.clone_box().downcast_ref::<Wolffluence>() {
                    return wolf_flu.influence_player.is_some();
                }
            }
            _ => {}
        }
    }
    false
}

pub fn player_is_dead(player: &mut Player, night_count: i32) {
    let loudmouth_player =
        if let Some(loudmouth) = player.role.clone_box().downcast_ref::<Loudmouth>() {
            loudmouth.reveal_player
        } else {
            None
        };

    let faction = player.role.faction();
    let original_role_id = player.role.id();

    player.alive = false;
    player.role = Box::new(Dead::new(original_role_id, night_count, loudmouth_player));
}

trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: 'static> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub trait RoleExt {
    fn downcast_ref<T: 'static>(&self) -> Option<&T>;
}

impl RoleExt for Box<dyn crate::types::Role> {
    fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        (**self).as_any().downcast_ref::<T>()
    }
}
