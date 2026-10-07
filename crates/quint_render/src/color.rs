use vello::peniko::Color;

pub fn parse_color(value: &str) -> Option<Color> {
    let s = value.trim();
    if s.is_empty() {
        return None;
    }

    // 1. Hex Colors: #rgb, #rgba, #rrggbb, #rrggbbaa
    if let Some(hex) = s.strip_prefix('#') {
        return parse_hex_color(hex);
    }

    // 2. Functional notation: rgb(...), rgba(...), hsl(...), hsla(...)
    let lower = s.to_ascii_lowercase();
    if lower.starts_with("rgb(") || lower.starts_with("rgba(") {
        return parse_rgb_functional(&lower);
    }
    if lower.starts_with("hsl(") || lower.starts_with("hsla(") {
        return parse_hsl_functional(&lower);
    }

    // 3. Named CSS Colors
    match lower.as_str() {
        "transparent" => Some(Color::from_rgba8(0, 0, 0, 0)),
        "black" => Some(Color::from_rgb8(0, 0, 0)),
        "white" => Some(Color::from_rgb8(255, 255, 255)),
        "red" => Some(Color::from_rgb8(255, 0, 0)),
        "green" => Some(Color::from_rgb8(0, 128, 0)),
        "blue" => Some(Color::from_rgb8(0, 0, 255)),
        "yellow" => Some(Color::from_rgb8(255, 255, 0)),
        "purple" => Some(Color::from_rgb8(128, 0, 128)),
        "orange" => Some(Color::from_rgb8(255, 165, 0)),
        "cyan" | "aqua" => Some(Color::from_rgb8(0, 255, 255)),
        "magenta" | "fuchsia" => Some(Color::from_rgb8(255, 0, 255)),
        "lime" => Some(Color::from_rgb8(0, 255, 0)),
        "gray" | "grey" => Some(Color::from_rgb8(128, 128, 128)),
        "lightgray" | "lightgrey" => Some(Color::from_rgb8(211, 211, 211)),
        "darkgray" | "darkgrey" => Some(Color::from_rgb8(169, 169, 169)),
        "silver" => Some(Color::from_rgb8(192, 192, 192)),
        "maroon" => Some(Color::from_rgb8(128, 0, 0)),
        "olive" => Some(Color::from_rgb8(128, 128, 0)),
        "navy" => Some(Color::from_rgb8(0, 0, 128)),
        "teal" => Some(Color::from_rgb8(0, 128, 128)),
        "brown" => Some(Color::from_rgb8(165, 42, 42)),
        "pink" => Some(Color::from_rgb8(255, 192, 203)),
        "coral" => Some(Color::from_rgb8(255, 127, 80)),
        "gold" => Some(Color::from_rgb8(255, 215, 0)),
        "indigo" => Some(Color::from_rgb8(75, 0, 130)),
        "violet" => Some(Color::from_rgb8(238, 130, 238)),
        "khaki" => Some(Color::from_rgb8(240, 230, 140)),
        "crimson" => Some(Color::from_rgb8(220, 20, 60)),
        "tomato" => Some(Color::from_rgb8(255, 99, 71)),
        "turquoise" => Some(Color::from_rgb8(64, 224, 208)),
        "salmon" => Some(Color::from_rgb8(250, 128, 114)),
        "snow" => Some(Color::from_rgb8(255, 250, 250)),
        "beige" => Some(Color::from_rgb8(245, 245, 220)),
        "wheat" => Some(Color::from_rgb8(245, 222, 179)),
        _ => None,
    }
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(Color::from_rgb8(r, g, b))
        }
        4 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            let a = u8::from_str_radix(&hex[3..4].repeat(2), 16).ok()?;
            Some(Color::from_rgba8(r, g, b, a))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::from_rgb8(r, g, b))
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

fn parse_channel(val: &str, max: f32) -> Option<f32> {
    let trimmed = val.trim();
    if let Some(pct) = trimmed.strip_suffix('%') {
        pct.parse::<f32>().ok().map(|p| (p / 100.0) * max)
    } else {
        trimmed.parse::<f32>().ok()
    }
}

fn parse_rgb_functional(s: &str) -> Option<Color> {
    let start = s.find('(')? + 1;
    let end = s.rfind(')')?;
    let inner = &s[start..end];

    // Split either by comma or whitespace / slash
    let cleaned = inner.replace('/', ",");
    let parts: Vec<&str> = if cleaned.contains(',') {
        cleaned.split(',').collect()
    } else {
        cleaned.split_whitespace().collect()
    };

    if parts.len() < 3 {
        return None;
    }

    let r = parse_channel(parts[0], 255.0)?.clamp(0.0, 255.0) as u8;
    let g = parse_channel(parts[1], 255.0)?.clamp(0.0, 255.0) as u8;
    let b = parse_channel(parts[2], 255.0)?.clamp(0.0, 255.0) as u8;

    let a = if parts.len() >= 4 {
        let alpha = parse_channel(parts[3], 1.0)?;
        (alpha.clamp(0.0, 1.0) * 255.0) as u8
    } else {
        255
    };

    Some(Color::from_rgba8(r, g, b, a))
}

fn parse_hsl_functional(s: &str) -> Option<Color> {
    let start = s.find('(')? + 1;
    let end = s.rfind(')')?;
    let inner = &s[start..end];

    let cleaned = inner.replace('/', ",");
    let parts: Vec<&str> = if cleaned.contains(',') {
        cleaned.split(',').collect()
    } else {
        cleaned.split_whitespace().collect()
    };

    if parts.len() < 3 {
        return None;
    }

    let h = parts[0].trim().trim_end_matches("deg").parse::<f32>().ok()? % 360.0;
    let h = if h < 0.0 { h + 360.0 } else { h };
    let s = parts[1].trim().trim_end_matches('%').parse::<f32>().ok()? / 100.0;
    let l = parts[2].trim().trim_end_matches('%').parse::<f32>().ok()? / 100.0;

    let a = if parts.len() >= 4 {
        let alpha_raw = parse_channel(parts[3], 1.0)?;
        (alpha_raw.clamp(0.0, 1.0) * 255.0) as u8
    } else {
        255
    };

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r1, g1, b1) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let r = ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;

    Some(Color::from_rgba8(r, g, b, a))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_colors() {
        assert_eq!(parse_color("#ff0000"), Some(Color::from_rgb8(255, 0, 0)));
        assert_eq!(parse_color("#0f0"), Some(Color::from_rgb8(0, 255, 0)));
        assert_eq!(parse_color("#eee"), Some(Color::from_rgb8(238, 238, 238)));
        assert_eq!(parse_color("#348"), Some(Color::from_rgb8(0x33, 0x44, 0x88)));
        assert_eq!(parse_color("#ffffff"), Some(Color::from_rgb8(255, 255, 255)));
        assert_eq!(parse_color("#f0f0f2"), Some(Color::from_rgb8(0xf0, 0xf0, 0xf2)));
        assert_eq!(parse_color("#0000ff80"), Some(Color::from_rgba8(0, 0, 255, 128)));
    }

    #[test]
    fn test_rgb_functional() {
        assert_eq!(parse_color("rgb(255, 0, 0)"), Some(Color::from_rgb8(255, 0, 0)));
        assert_eq!(parse_color("rgb(10, 20, 30)"), Some(Color::from_rgb8(10, 20, 30)));
        assert_eq!(parse_color("rgba(0, 255, 0, 0.5)"), Some(Color::from_rgba8(0, 255, 0, 127)));
        assert_eq!(parse_color("rgba(10, 20, 30, 0.5)"), Some(Color::from_rgba8(10, 20, 30, 127)));
        assert_eq!(parse_color("rgb(100%, 100%, 100%)"), Some(Color::from_rgb8(255, 255, 255)));
    }

    #[test]
    fn test_hsl_functional() {
        assert_eq!(parse_color("hsl(0, 100%, 50%)"), Some(Color::from_rgb8(255, 0, 0)));
        assert_eq!(parse_color("hsl(120, 100%, 50%)"), Some(Color::from_rgb8(0, 255, 0)));
    }

    #[test]
    fn test_named_colors() {
        assert_eq!(parse_color("yellow"), Some(Color::from_rgb8(255, 255, 0)));
        assert_eq!(parse_color("orange"), Some(Color::from_rgb8(255, 165, 0)));
        assert_eq!(parse_color("transparent"), Some(Color::from_rgba8(0, 0, 0, 0)));
    }
}
