use futures::future::join_all;
use image::{
    imageops, DynamicImage, GenericImageView, ImageBuffer, ImageOutputFormat, Rgba, RgbaImage,
};
use imageproc::drawing::{draw_line_segment_mut, draw_text_mut};
use rusttype::{Font, Point, Scale};
use std::io::Cursor;

use crate::types::player::PlayerInfo;
use anyhow::{Context, Result};

const AVATAR_SIZE: u32 = 64;
const PADDING: u32 = 15;
const TEXT_HEIGHT: u32 = 20;
const AVATARS_PER_ROW: u32 = 5;
const FONT_SIZE: f32 = 12.0;

const BG_COLOR: Rgba<u8> = Rgba([47, 49, 54, 255]);
const TEXT_COLOR: Rgba<u8> = Rgba([255, 255, 255, 255]);
const DEAD_COLOR: Rgba<u8> = Rgba([255, 0, 0, 255]);

pub async fn create_avatar_collage(players: &[PlayerInfo]) -> Result<Vec<u8>> {
    if players.is_empty() {
        return Ok(Vec::new());
    }

    let rows = (players.len() as u32 + AVATARS_PER_ROW - 1) / AVATARS_PER_ROW;
    let width = PADDING + AVATARS_PER_ROW * (AVATAR_SIZE + PADDING);
    let height = PADDING + rows * (AVATAR_SIZE + TEXT_HEIGHT + PADDING);

    let mut canvas: RgbaImage = ImageBuffer::from_pixel(width, height, BG_COLOR);

    let font_data = std::fs::read("assets/fonts/ARIAL.TTF")
        .context("Không tìm thấy file assets/fonts/ARIAL.TTF")?;

    let font =
        Font::try_from_vec(font_data).context("Lỗi parse font (file bị lỗi hoặc không hỗ trợ)")?;
    let scale = Scale::uniform(FONT_SIZE);

    let client = reqwest::Client::new();
    let fetch_tasks = players
        .iter()
        .map(|p| async { download_and_resize_avatar(&client, &p.avatar_url).await });

    let images = join_all(fetch_tasks).await;

    for (i, player) in players.iter().enumerate() {
        let col = (i as u32) % AVATARS_PER_ROW;
        let row = (i as u32) / AVATARS_PER_ROW;

        let x = PADDING + col * (AVATAR_SIZE + PADDING);
        let y = PADDING + row * (AVATAR_SIZE + TEXT_HEIGHT + PADDING);

        if let Some(img) = &images[i] {
            imageops::overlay(&mut canvas, img, x as i64, y as i64);
        }

        if !player.alive {
            let thickness = 5.0;
            let start_x = x as f32;
            let start_y = y as f32;
            let end_x = (x + AVATAR_SIZE) as f32;
            let end_y = (y + AVATAR_SIZE) as f32;

            draw_thick_line(
                &mut canvas,
                (start_x, start_y),
                (end_x, end_y),
                thickness,
                DEAD_COLOR,
            );
            draw_thick_line(
                &mut canvas,
                (end_x, start_y),
                (start_x, end_y),
                thickness,
                DEAD_COLOR,
            );
        }

        let raw_name = player.global_name.as_ref().unwrap_or(&player.username);
        let display_name = format!("({}) {}", i + 1, raw_name);

        let truncated_name = truncate_text(&font, scale, &display_name, AVATAR_SIZE as i32);

        let text_width = get_text_width(&font, scale, &truncated_name);
        let text_x = x as i32 + (AVATAR_SIZE as i32 / 2) - (text_width / 2);
        let text_y = y as i32 + AVATAR_SIZE as i32 + 2;

        draw_text_mut(
            &mut canvas,
            TEXT_COLOR,
            text_x,
            text_y,
            scale,
            &font,
            &truncated_name,
        );
    }

    let mut cursor = Cursor::new(Vec::new());
    canvas
        .write_to(&mut cursor, ImageOutputFormat::Png)
        .context("Lỗi khi encode ảnh sang PNG")?;

    Ok(cursor.into_inner())
}

async fn download_and_resize_avatar(client: &reqwest::Client, url: &str) -> Option<DynamicImage> {
    let url = if url.contains("discord") && !url.contains("?size=") {
        format!("{}?size=64", url)
    } else {
        url.to_string()
    };

    let resp = client.get(&url).send().await.ok()?;
    let bytes = resp.bytes().await.ok()?;
    let img = image::load_from_memory(&bytes).ok()?;

    Some(img.resize_exact(AVATAR_SIZE, AVATAR_SIZE, imageops::FilterType::Lanczos3))
}

fn get_text_width(font: &Font, scale: Scale, text: &str) -> i32 {
    font.layout(text, scale, Point { x: 0.0, y: 0.0 })
        .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .last()
        .unwrap_or(0.0)
        .ceil() as i32
}

fn truncate_text(font: &Font, scale: Scale, text: &str, max_width: i32) -> String {
    if get_text_width(font, scale, text) <= max_width {
        return text.to_string();
    }

    let mut truncated = text.to_string();
    let ellipsis = "...";

    loop {
        if truncated.is_empty() {
            break;
        }

        let test_str = format!("{}{}", truncated, ellipsis);

        if get_text_width(font, scale, &test_str) <= max_width {
            return test_str;
        }

        truncated.pop();
    }

    "...".to_string()
}

fn draw_thick_line(
    canvas: &mut RgbaImage,
    start: (f32, f32),
    end: (f32, f32),
    thickness: f32,
    color: Rgba<u8>,
) {
    let half = (thickness / 2.0).ceil() as i32;
    for offset in -half..=half {
        draw_line_segment_mut(
            canvas,
            (start.0 + offset as f32, start.1),
            (end.0 + offset as f32, end.1),
            color,
        );
        draw_line_segment_mut(
            canvas,
            (start.0, start.1 + offset as f32),
            (end.0, end.1 + offset as f32),
            color,
        );
    }
}
