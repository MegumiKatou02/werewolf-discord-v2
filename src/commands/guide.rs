use crate::bot::BotData;
use crate::commands::{CommandFuture, SlashCommand};
use rand::seq::SliceRandom;
use serenity::all::*;
use std::sync::Arc;

pub struct HuongDanCommand;

impl SlashCommand for HuongDanCommand {
    fn name(&self) -> &'static str {
        "huongdan"
    }

    fn run(&self, ctx: Context, cmd: CommandInteraction, _data: Arc<BotData>) -> CommandFuture {
        Box::pin(async move {
            let owner_id = cmd.user.id.to_string();

            let (embed, row) = get_guide_content("guide_tips", &owner_id);

            cmd.create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .add_embed(embed)
                        .components(vec![row]),
                ),
            )
            .await?;

            Ok(())
        })
    }
}

fn get_random_tip() -> String {
    let tips = vec![
        "Dùng !role khi đang chơi ở chat bot để xem vai trò của bản thân",
        "Dùng !{role} để xem thông tin vai trò, ví dụ !masoi, !stalker, !xathu,...",
    ];
    tips.choose(&mut rand::thread_rng())
        .unwrap_or(&"Chúc bạn chơi game vui vẻ!")
        .to_string()
}

pub fn get_guide_content(value: &str, owner_id: &str) -> (CreateEmbed, CreateActionRow) {
    let footer_text = format!("💡 Mẹo: {}", get_random_tip());

    let embed = match value {
        "guide_rules" => CreateEmbed::new()
            .title("📜 LUẬT CHƠI CƠ BẢN")
            .color(0x9c27b0)
            .field(
                "⏰ Chu Kỳ Ngày Đêm",
                "• **Ban Đêm**: Các vai trò đặc biệt thực hiện khả năng của mình\n\
                    • **Ban Ngày**:Mọi người thảo luận và vote treo cổ người khả nghi",
                false
            )
            .field(
                "🎯 Cách Vote",
                "• **Ban Đêm**: Sói vote để cắn người\n\
                    • **Ban Ngày**: Mọi người vote để treo cổ",
                false
            )
            .field(
                "🏆 Điều Kiện Thắng",
                "• Người có số vote cao nhất và ít nhất 2 vote sẽ bị treo",
                false
            )
            .footer(CreateEmbedFooter::new(footer_text)),

        "guide_roles" => CreateEmbed::new()
            .title("🎭 CÁC VAI TRÒ CHÍNH")
            .color(0x2196f3)
            .field(
                "🐺 Sói (Phe Sói)",
                "• Mỗi đêm chọn 1 người để cắn\n• Biết được đồng đội là ai\n• Có thể chat riêng với nhau vào ban đêm",
                true
            )
            .field(
                "🛡️ Bảo Vệ (Phe Dân)",
                "• Mỗi đêm bảo vệ 1 người khỏi bị Sói cắn\n• Có 2 máu khi bảo vệ người bị cắn\n• Có thể tự bảo vệ mình",
                true
            )
            .field(
                "🔮 Tiên Tri (Phe Dân)",
                "• Mỗi đêm soi vai trò của 1 người\n• Biết được người đó thuộc phe nào",
                true
            )
            .field(
                "🕵️ Thám Tử (Phe Dân)",
                "• Mỗi đêm điều tra 2 người\n• Biết 2 người đó có cùng phe không",
                true
            )
            .field(
                "🧙‍♀️ Phù Thủy (Phe Dân)",
                "• Có 1 bình cứu và 1 bình độc\n• Biết ai bị Sói cắn để cứu\n• Có thể dùng bình độc giết 1 người",
                true
            )
            .field(
                "👻 Thầy Đồng (Phe Dân)",
                "• Có thể hồi sinh 1 người dân đã chết\n• Chỉ được dùng 1 lần trong game",
                true
            )
            .field(
                "🎪 Thằng Ngố (Phe Solo)",
                "• Thắng nếu bị dân làng treo cổ\n• Thua nếu chết vì lý do khác",
                true
            )
            .field(
                "🌙 Bán Sói (Phe Dân)",
                "• Ban đầu là dân thường\n• Biến thành Sói nếu bị Sói cắn",
                true
            )
            .field(
                "👒 Hầu Gái (Phe Dân)",
                "• Ban đầu là Hầu Gái và được chọn chủ trong đêm đầu tiên\n• Biến thành vai trò của chủ nếu chủ chết",
                true
            )
            .field(
                "🤷 Lycan (Phe Dân)",
                "• Không có gì cả ngoài việc bị cho là phe sói khi bị soi\n• LYCAN LÀ DÂN",
                true
            )
            .field(
                "🐺 Sói Trùm (Phe Sói)",
                "• Che các sói khỏi tiên tri\n• Được phép che liên tục một người",
                true
            )
            .field(
                "🐺 Sói Tiên Tri (Phe Sói)",
                "• Soi xem ai là tiên tri\n• Được quản trò báo cho cả làng soi ai và báo cho sói có phải tiên tri hay không",
                true
            )
            .field(
                "ℹ️ Xem thêm",
                "Xem thêm nhiều role khác bằng cách dùng lệnh `/role`",
                false
            )
            .footer(CreateEmbedFooter::new(footer_text)),

        _ => CreateEmbed::new() // Mặc định là guide_tips
            .title("💡 CÁCH CHƠI VỚI BOT")
            .color(0x4caf50)
            .field(
                "⚠️ Lưu Ý Quan Trọng",
                "• **Bạn cần BẬT \"Cho phép tin nhắn trực tiếp từ thành viên máy chủ\" trong Discord để có thể chơi!**\n\
                • Cách bật: Chuột phải vào server > Cài đặt bảo mật > Bật \"Cho phép tin nhắn trực tiếp từ thành viên máy chủ\"\n\
                • Nếu không bật, bạn sẽ không nhận được thông báo vai trò và không thể tương tác trong game!",
                false,
            )
            .field(
                "📋 Cách Chơi Chính",
                "• Khi bắt đầu game bot sẽ nhắn cho bạn\n\
                • Bạn và người khác sẽ giao tiếp thông qua bot bằng cách nhắn trực tiếp vào thanh chat\n\
                • Bạn cũng có thể tương tác với vai trò của mình thông qua bot\n",
                false
            )
            .field(
                "🎮 Các Lệnh Trong Game",
                concat!(
                    "`/masoi-create` - Tạo phòng mới\n",
                    "`/masoi-join` - Tham gia phòng\n",
                    "`/masoi-leave` - Rời phòng\n",
                    "`/masoi-start` - Bắt đầu game (chỉ host)\n",
                    "`/role` - Xem thông tin chi tiết của các vai trò trong game Ma Sói\n",
                    "`/status` - Xem trạng thái phòng trong server\n",
                    "`/huongdan` - Xem hướng dẫn này\n",
                    "...",
                ),
                false)
            .footer(CreateEmbedFooter::new(footer_text)),
    };

    let custom_id = format!("guide_select:{}", owner_id);

    let menu = CreateSelectMenu::new(
        custom_id,
        CreateSelectMenuKind::String {
            options: vec![
                CreateSelectMenuOption::new("Cách chơi", "guide_tips").emoji('💡'),
                CreateSelectMenuOption::new("Luật chơi", "guide_rules").emoji('📜'),
                CreateSelectMenuOption::new("Vai trò", "guide_roles").emoji('🎭'),
            ],
        },
    )
    .placeholder("Chọn hướng dẫn bạn muốn xem...");

    (embed, CreateActionRow::SelectMenu(menu))
}
