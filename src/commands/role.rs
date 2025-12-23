use crate::bot::BotData;
use crate::{
    commands::{CommandFuture, SlashCommand},
    types::data::RolesData,
};
use serenity::all::*;
use std::sync::Arc;

pub struct RoleCommand;

impl SlashCommand for RoleCommand {
    fn name(&self) -> &'static str {
        "role"
    }

    fn run(&self, ctx: Context, cmd: CommandInteraction, data: Arc<BotData>) -> CommandFuture {
        Box::pin(async move {
            let owner_id = cmd.user.id.to_string();

            let initial_embed = CreateEmbed::new()
                .title("🎭 THÔNG TIN VAI TRÒ")
                .description(
                    "Chọn một vai trò từ menu bên dưới để xem thông tin chi tiết!\n\n\
                             🐺 **Phe Sói** - Cần tiêu diệt dân làng\n\
                             👤 **Phe Dân** - Cần tìm và tiêu diệt sói\n\
                             🎪 **Phe Solo** - Có mục tiêu riêng\n\
                             🌙 **??** - Có thể chuyển phe",
                )
                .color(0x00ae86)
                .footer(CreateEmbedFooter::new(
                    "Sử dụng menu bên dưới để chọn vai trò!",
                ));

            let row = get_role_menu_row(&owner_id, &data.roles_json);

            cmd.create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .add_embed(initial_embed)
                        .components(vec![row]),
                ),
            )
            .await?;

            Ok(())
        })
    }
}

pub fn get_role_menu_row(owner_id: &str, roles_data: &RolesData) -> CreateActionRow {
    let mut options = Vec::new();

    for (id, role) in roles_data {
        if id == "9" {
            continue;
        }

        let emoji = match role.faction {
            0 => '🐺',
            1 => '👤',
            2 => '🎪',
            3 => '🌙',
            _ => '❓',
        };

        let mut desc = role.description.clone();
        if desc.len() > 80 {
            desc = desc.chars().take(77).collect::<String>() + "...";
        }

        options.push(
            CreateSelectMenuOption::new(format!("{} ({})", role.title, role.e_name), id)
                .description(desc)
                .emoji(emoji),
        );
    }

    let menu = CreateSelectMenu::new(
        format!("role_select:{}", owner_id),
        CreateSelectMenuKind::String { options },
    )
    .placeholder("Chọn một vai trò để xem thông tin chi tiết...");

    CreateActionRow::SelectMenu(menu)
}
