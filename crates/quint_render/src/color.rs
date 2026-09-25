use tiny_skia::Color;

pub fn parse_color(value: &str) -> Option<Color> {
    let value = value.trim().to_lowercase();
    match value.as_str() {
        "red" => Some(Color::from_rgba8(255, 0, 0, 255)),
        "green" => Some(Color::from_rgba8(0, 128, 0, 255)), // CSS green is dark
        "lime" => Some(Color::from_rgba8(0, 255, 0, 255)),  // CSS lime is bright
        "blue" => Some(Color::from_rgba8(0, 0, 255, 255)),
        "black" => Some(Color::from_rgba8(0, 0, 0, 255)),
        "white" => Some(Color::from_rgba8(255, 255, 255, 255)),
        "yellow" => Some(Color::from_rgba8(255, 255, 0, 255)),
        "gray" | "grey" => Some(Color::from_rgba8(128, 128, 128, 255)),
        "transparent" => Some(Color::from_rgba8(0, 0, 0, 0)),
        _ if value.starts_with('#') => parse_hex(&value),
        _ => None,
    }
}

fn parse_hex(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            Some(Color::from_rgba8(
                (r << 4) | r,
                (g << 4) | g,
                (b << 4) | b,
                255,
            ))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::from_rgba8(r, g, b, 255))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Color::from_rgba8(r, g, b, a))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_keywords() {
        assert_eq!(
            parse_color("red").unwrap(),
            Color::from_rgba8(255, 0, 0, 255)
        );
        assert_eq!(
            parse_color("BLACK ").unwrap(),
            Color::from_rgba8(0, 0, 0, 255)
        );
    }

    #[test]
    fn parse_hex_colors() {
        assert_eq!(
            parse_color("#ff0000").unwrap(),
            Color::from_rgba8(255, 0, 0, 255)
        );
        assert_eq!(
            parse_color("#00FF00").unwrap(),
            Color::from_rgba8(0, 255, 0, 255)
        );
        assert_eq!(
            parse_color("#abc").unwrap(),
            Color::from_rgba8(0xaa, 0xbb, 0xcc, 255)
        );
        assert_eq!(
            parse_color("#12345678").unwrap(),
            Color::from_rgba8(0x12, 0x34, 0x56, 0x78)
        );
    }
}
