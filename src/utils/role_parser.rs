use serde_json::Value;
use std::collections::HashMap;

pub fn get_role_id_by_name(name: &str) -> Option<u8> {
    let key = name.trim().to_lowercase();
    match key.as_str() {
        // 0
        "masoi" | "werewolf" | "ma sói" | "soi" | "sói" => Some(0),
        // 1
        "danlang" | "villager" | "dân làng" | "dan" | "dân" => Some(1),
        // 2
        "baove" | "bodyguard" | "bảo vệ" => Some(2),
        // 3
        "bansoi" | "cursed" | "bán sói" => Some(3),
        // 4
        "tientri" | "seer" | "tiên tri" => Some(4),
        // 5
        "thamtu" | "detective" | "thám tử" => Some(5),
        // 6
        "phuthuy" | "witch" | "phù thủy" => Some(6),
        // 7
        "thangngo" | "fool" | "thằng ngố" => Some(7),
        // 8
        "thaydong" | "medium" | "thầy đồng" => Some(8),
        // 10
        "haugai" | "maid" | "hầu gái" => Some(10),
        // 11
        "lycan" => Some(11),
        // 12
        "soitientri" | "wolfseer" | "sói tiên tri" => Some(12),
        // 13
        "soitrum" | "alphawerewolf" | "sói trùm" => Some(13),
        // 14
        "cao" | "foxspirit" | "cáo" => Some(14),
        // 15
        "gialang" | "elder" | "già làng" => Some(15),
        // 16
        "stalker" | "hori" | "stalkẻ" => Some(16),
        // 17
        "xathu" | "gunner" | "xạ thủ" => Some(17),
        // 18
        "soimeocon" | "kittenwolf" | "kitten" | "sói mèo con" => Some(18),
        // 19
        "puppeteer" | "nguoimuaroi" | "người múa rối" => Some(19),
        // 20
        "voodoo" | "soitathuat" | "sói tà thuật" => Some(20),
        // 21
        "wolffluencer" | "wolffluence" | "soithaotung" | "awai" | "sói thao túng" => Some(21),
        // 22
        "loudmouth" | "caubemiengbu" | "cậu bé miệng bự" => Some(22),
        _ => None,
    }
}

pub fn parse_roles_from_string(input: &str) -> Result<HashMap<u8, u8>, String> {
    let mut roles = HashMap::new();
    let pairs: Vec<&str> = input.split(',').map(|s| s.trim()).collect();

    for pair in pairs {
        if pair.is_empty() {
            continue;
        }
        let parts: Vec<&str> = pair.split(':').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err(format!("Sai định dạng: '{}'. Dùng 'tên: số lượng'", pair));
        }

        let role_id = get_role_id_by_name(parts[0])
            .ok_or_else(|| format!("Không tìm thấy vai trò: {}", parts[0]))?;

        let count: u8 = parts[1]
            .parse()
            .map_err(|_| format!("Số lượng không hợp lệ cho '{}'", parts[0]))?;

        if count == 0 {
            return Err(format!(
                "Số lượng không hợp lệ cho '{}': phải > 0",
                parts[0]
            ));
        }

        *roles.entry(role_id).or_insert(0) += count;
    }
    Ok(roles)
}

pub fn parse_roles_from_json_string(input: &str) -> Result<HashMap<u8, u8>, String> {
    let v: Value = serde_json::from_str(input)
        .map_err(|_| "JSON không hợp lệ. Vui lòng kiểm tra lại cú pháp.".to_string())?;

    let obj = v
        .as_object()
        .ok_or_else(|| "JSON phải là object, ví dụ: {\"0\": 2, \"1\": 3}".to_string())?;

    if obj.is_empty() {
        return Err("JSON vai trò không được để trống.".to_string());
    }

    let mut roles: HashMap<u8, u8> = HashMap::new();

    for (key, val) in obj.iter() {
        let role_id_u64: u64 = key
            .parse()
            .map_err(|_| "Các key trong JSON phải là ID vai trò (số).".to_string())?;
        let role_id: u8 = role_id_u64
            .try_into()
            .map_err(|_| "Role ID quá lớn (phải nằm trong 0..255).".to_string())?;

        let count_u64: u64 = match val {
            Value::Number(n) => n
                .as_u64()
                .ok_or_else(|| "Số lượng vai trò phải là số dương.".to_string())?,
            Value::String(s) => s
                .parse::<u64>()
                .map_err(|_| "Số lượng vai trò phải là số dương.".to_string())?,
            _ => return Err("Số lượng vai trò phải là số dương.".to_string()),
        };

        if count_u64 == 0 {
            return Err("Số lượng vai trò phải là số dương.".to_string());
        }

        let count: u8 = count_u64
            .try_into()
            .map_err(|_| "Số lượng role quá lớn (phải nằm trong 1..255).".to_string())?;

        roles.insert(role_id, count);
    }

    Ok(roles)
}
