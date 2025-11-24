// Cursor detection and color parsing utilities for macOS
use crate::types::CursorResult;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// Very basic XML key-value string parser for Apple's plist.
/// Only supports extracting flat keys with dict nesting one level deep. Not robust!
pub fn get_plist_dict_value<'a>(plist: &'a str, key: &str) -> Option<&'a str> {
    let key_tag = format!("<key>{}</key>", key);
    let idx = plist.find(&key_tag)?;
    let after = &plist[idx + key_tag.len()..];
    let val_start = after.find('<')?;
    let val_end = after[val_start..].find('>')?;
    let tag = &after[val_start + 1..val_start + val_end];
    let close_tag = format!("</{}>", tag);
    let content_start = val_start + val_end + 1;
    let content_end = after[content_start..].find(&close_tag)?;
    let value = &after[content_start..content_start + content_end];
    Some(value.trim())
}

/// Parse a color dictionary (as a string) and extract RGBA as f64.
/// Only works if the color dict is stored as XML inline.
pub fn parse_color_dict(dict_str: &str) -> Option<(u8, u8, u8, u8)> {
    let get_comp = |name| {
        dict_str
            .find(&format!("<key>{}</key>", name))
            .and_then(|idx| {
                let after = &dict_str[idx + format!("<key>{}</key>", name).len()..];
                let real_start = after.find("<real>")?;
                let real_end = after[real_start + 6..].find("</real>")?;
                let val = &after[real_start + 6..real_start + 6 + real_end];
                val.trim().parse::<f64>().ok()
            })
    };
    let r = get_comp("red")?;
    let g = get_comp("green")?;
    let b = get_comp("blue")?;
    let a = get_comp("alpha")?;
    Some((
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
        (a * 255.0).round() as u8,
    ))
}

/// Format color like the C code: White/Black or #RRGGBBAA
pub fn format_color(r: u8, g: u8, b: u8, a: u8) -> String {
    match (r, g, b, a) {
        (255, 255, 255, 255) => "White".to_string(),
        (0, 0, 0, 255) => "Black".to_string(),
        _ => format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a),
    }
}

/// Main function: detect cursor info for macOS, no external libraries.
pub fn detect_cursor_apple(home_dir: &str) -> CursorResult {
    let mut result = CursorResult::default();
    let mut plist_path = PathBuf::from(home_dir);
    plist_path.push("Library/Preferences/com.apple.universalaccess.plist");

    let mut file = match File::open(&plist_path) {
        Ok(f) => f,
        Err(e) => {
            result.error = Some(format!("Failed to open {}: {}", plist_path.display(), e));
            return result;
        }
    };

    let mut contents = String::new();
    if let Err(e) = file.read_to_string(&mut contents) {
        result.error = Some(format!("Failed to read {}: {}", plist_path.display(), e));
        return result;
    }

    result.theme.push_str("Fill - ");
    if let Some(color_str) = get_plist_dict_value(&contents, "cursorFill") {
        if let Some((r, g, b, a)) = parse_color_dict(color_str) {
            result.theme.push_str(&format_color(r, g, b, a));
        } else {
            result.theme.push_str("Black");
        }
    } else {
        result.theme.push_str("Black");
    }

    result.theme.push_str(", Outline - ");
    if let Some(color_str) = get_plist_dict_value(&contents, "cursorOutline") {
        if let Some((r, g, b, a)) = parse_color_dict(color_str) {
            result.theme.push_str(&format_color(r, g, b, a));
        } else {
            result.theme.push_str("White");
        }
    } else {
        result.theme.push_str("White");
    }

    // Cursor size (default: 32)
    if let Some(size_str) = get_plist_dict_value(&contents, "mouseDriverCursorSize") {
        if let Ok(size_f) = size_str.parse::<f64>() {
            let size = (size_f * 32.0).round() as u32;
            result.size = size.to_string();
        } else {
            result.size = "32".to_string();
        }
    } else {
        result.size = "32".to_string();
    }

    result
}
