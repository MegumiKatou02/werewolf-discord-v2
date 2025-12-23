use serenity::{all::Message, prelude::*};

use crate::{bot::BotData, types::Faction, utils::response::role_response};

pub async fn command_handler(ctx: &Context, msg: &Message, data: &BotData) {
    let _ = role_response(
        ctx,
        msg,
        vec!["!soi", "!masoi", "!werewolf"],
        "werewolf.png",
        0,
        Faction::Werewolf.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!danlang", "!villager"],
        "villager.png",
        1,
        Faction::Village.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!baove", "!bodyguard"],
        "bodyguard.png",
        2,
        Faction::Village.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!bansoi", "!cursed"],
        "cursed.png",
        3,
        Faction::ViWolf.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!tientri", "!seer"],
        "seer.png",
        4,
        Faction::Village.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!thamtu", "!detective"],
        "detective.png",
        5,
        Faction::Village.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!phuthuy", "!witch"],
        "witch.png",
        6,
        Faction::Village.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!thangngo", "!fool"],
        "fool.png",
        7,
        Faction::Solo.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!thaydong", "!medium"],
        "medium.png",
        8,
        Faction::Village.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!haugai", "!maid"],
        "maid.png",
        10,
        Faction::Village.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!lycan", "!soicodoc", "!shiba"],
        "lycan.png",
        11,
        Faction::Village.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!stalker", "!hori", "!stalkẻ"],
        "stalker.png",
        16,
        Faction::Solo.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!wolfseer", "!soitientri", "!soitri"],
        "wolf_seer.png",
        12,
        Faction::Werewolf.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!alphawerewolf", "!soitrum", "!soicosplay"],
        "alpha_werewolf.png",
        13,
        Faction::Werewolf.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!cao", "!foxspirit", "!holy", "!fox"],
        "fox_spirit.png",
        14,
        Faction::Village.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!gialang", "!elder"],
        "elder.png",
        15,
        Faction::Village.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!xathu", "!gunner"],
        "gunner.png",
        17,
        Faction::Village.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!soimeocon", "!kittenwolf", "!kitten"],
        "kitten_wolf.png",
        18,
        Faction::Werewolf.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!puppeteer", "!nguoimuaroi"],
        "the_puppeteer.png",
        19,
        Faction::Village.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!voodoo", "!soitathuat"],
        "voodoo_werewolf.png",
        20,
        Faction::Werewolf.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!wolffluencer", "!soithaotung", "!awai"],
        "wolffluencer.png",
        21,
        Faction::Werewolf.name(),
        data,
    )
    .await;
    let _ = role_response(
        ctx,
        msg,
        vec!["!loudmouth", "!caubemiengbu"],
        "loudmouth.png",
        22,
        Faction::Village.name(),
        data,
    )
    .await;
}
