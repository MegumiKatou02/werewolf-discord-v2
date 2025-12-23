mod create;
pub mod guide;
mod join;
mod leave;
mod ping;
pub mod role;
mod start;
mod status;

use crate::commands::{
    guide::HuongDanCommand, join::JoinCommand, leave::LeaveCommand, ping::PingCommand,
    role::RoleCommand, start::StartCommand, status::StatusCommand,
};
use create::CreateCommand;

use crate::bot::BotData;
use serenity::all::{CommandInteraction, Context};
use std::{future::Future, pin::Pin, sync::Arc};

pub type CommandResult = anyhow::Result<()>;
pub type CommandFuture = Pin<Box<dyn Future<Output = CommandResult> + Send>>;

pub trait SlashCommand: Send + Sync {
    fn name(&self) -> &'static str;
    fn run(&self, ctx: Context, command: CommandInteraction, data: Arc<BotData>) -> CommandFuture;
}

pub fn all_commands() -> Vec<Box<dyn SlashCommand>> {
    vec![
        Box::new(CreateCommand),
        Box::new(HuongDanCommand),
        Box::new(PingCommand),
        Box::new(RoleCommand),
        Box::new(JoinCommand),
        Box::new(LeaveCommand),
        Box::new(StatusCommand),
        Box::new(StartCommand),
    ]
}
