use crate::bot::BotData;
use crate::commands::{CommandFuture, SlashCommand};
use serenity::all::*;
use std::sync::Arc;
use std::time::Instant;

pub struct PingCommand;

impl SlashCommand for PingCommand {
    fn name(&self) -> &'static str {
        "ping"
    }

    fn run(&self, ctx: Context, cmd: CommandInteraction, _data: Arc<BotData>) -> CommandFuture {
        Box::pin(async move {
            let start_time = Instant::now();

            cmd.defer(&ctx.http).await?;

            let latency = start_time.elapsed().as_millis();

            let color = if latency > 200 {
                0xf44336
            } else if latency > 100 {
                0xff9800
            } else {
                0x4caf50
            };

            let embed = CreateEmbed::new()
                .title("Pong 🏓")
                .description(format!("Bot: `{latency}ms` • API: `Heartbeat active`"))
                .color(color);

            cmd.edit_response(&ctx.http, EditInteractionResponse::new().add_embed(embed))
                .await?;

            Ok(())
        })
    }
}
