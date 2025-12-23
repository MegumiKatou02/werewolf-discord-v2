use crate::bot::BotData;
use crate::utils::embed::create_werewolf_embed;
use serenity::all::{
    ButtonStyle, Context, CreateActionRow, CreateAllowedMentions, CreateButton, CreateMessage,
    Message, User,
};

pub async fn role_response(
    ctx: &Context,
    message: &Message,
    command_names: Vec<&str>,
    file_name: &str,
    index_role: u32,
    faction_role: &str,
    data: &BotData,
) -> serenity::Result<()> {
    let msg_content = message.content.to_lowercase();

    if command_names
        .iter()
        .any(|&cmd| cmd.to_lowercase() == msg_content)
    {
        let role_key = index_role.to_string();

        if let Some(role) = data.roles_json.get(&role_key) {
            let title = format!("{} ({})", role.title, role.e_name);
            let description = format!("{}\n\nPhe: {}", role.description, faction_role);

            let data_embed = create_werewolf_embed(file_name, &title, &description).await?;

            message
                .channel_id
                .send_message(
                    &ctx.http,
                    CreateMessage::new()
                        .add_embed(data_embed.embed)
                        .add_file(data_embed.attachment)
                        .reference_message(message)
                        .allowed_mentions(CreateAllowedMentions::new().replied_user(true)),
                )
                .await?;
        }
    }
    Ok(())
}

pub async fn role_response_dms(
    ctx: &Context,
    user: &User,
    file_name: &str,
    index_role: u32,
    faction_role: &str,
    data: &BotData,
) -> serenity::Result<()> {
    let role_key = index_role.to_string();

    if let Some(role) = data.roles_json.get(&role_key) {
        let title = format!("{} ({})", role.title, role.e_name);
        let description = format!("{}\n\nPhe: {}", role.description, faction_role);

        let data_embed = create_werewolf_embed(file_name, &title, &description).await?;

        user.direct_message(
            &ctx.http,
            CreateMessage::new()
                .add_embed(data_embed.embed)
                .add_file(data_embed.attachment),
        )
        .await?;
    }
    Ok(())
}

pub fn row_single(
    custom_id: impl Into<String>,
    label: &str,
    style: ButtonStyle,
    disabled: bool,
) -> Vec<CreateActionRow> {
    vec![CreateActionRow::Buttons(vec![CreateButton::new(
        custom_id.into(),
    )
    .label(label)
    .style(style)
    .disabled(disabled)])]
}
