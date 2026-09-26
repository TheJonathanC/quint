pub fn parse_color(value: &str) -> Option<vello::peniko::Color> {
    let s = value.trim();
    if let Some(hex) = s.strip_prefix('#') {
        parse_hex_color(hex)
    } else if s.starts_with("rgb(") && s.ends_with(')') {
        parse_rgb_func(&s[4..s.len() - 1])
    } else if s.starts_with("rgba(") && s.ends_with(')') {
        parse_rgba_func(&s[5..s.len() - 1])
    } else {
        match s.to_ascii_lowercase().as_str() {
            "red" => Some(vello::peniko::Color::from_rgb8(255, 0, 0)),
            "green" => Some(vello::peniko::Color::from_rgb8(0, 255, 0)),
            "blue" => Some(vello::peniko::Color::from_rgb8(0, 0, 255)),
            "black" => Some(vello::peniko::Color::from_rgb8(0, 0, 0)),
            "white" => Some(vello::peniko::Color::from_rgb8(255, 255, 255)),
            "gray" | "grey" => Some(vello::peniko::Color::from_rgb8(128, 128, 128)),
            "lightgray" | "lightgrey" => Some(vello::peniko::Color::from_rgb8(211, 211, 211)),
            "darkgray" | "darkgrey" => Some(vello::peniko::Color::from_rgb8(169, 169, 169)),
            "transparent" => Some(vello::peniko::Color::from_rgba8(0, 0, 0, 0)),
            "yellow" => Some(vello::peniko::Color::from_rgb8(255, 255, 0)),
            "cyan" => Some(vello::peniko::Color::from_rgb8(0, 255, 255)),
            "magenta" => Some(vello::peniko::Color::from_rgb8(255, 0, 255)),
            "orange" => Some(vello::peniko::Color::from_rgb8(255, 165, 0)),
            "purple" => Some(vello::peniko::Color::from_rgb8(128, 0, 128)),
            _ => None,
        }
    }
}

fn parse_hex_color(hex: &str) -> Option<vello::peniko::Color> {
    let hex = hex.trim();
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            Some(vello::peniko::Color::from_rgb8(r * 17, g * 17, b * 17))
        }
        4 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            let a = u8::from_str_radix(&hex[3..4], 16).ok()?;
            Some(vello::peniko::Color::from_rgba8(r * 17, g * 17, b * 17, a * 17))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(vello::peniko::Color::from_rgb8(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(vello::peniko::Color::from_rgba8(r, g, b, a))
        }
        _ => None,
    }
}

fn parse_rgb_func(inner: &str) -> Option<vello::peniko::Color> {
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() != 3 {
        return None;
    }
    let r = parts[0].trim().parse::<u8>().ok()?;
    let g = parts[1].trim().parse::<u8>().ok()?;
    let b = parts[2].trim().parse::<u8>().ok()?;
    Some(vello::peniko::Color::from_rgb8(r, g, b))
}

fn parse_rgba_func(inner: &str) -> Option<vello::peniko::Color> {
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() != 4 {
        return None;
    }
    let r = parts[0].trim().parse::<u8>().ok()?;
    let g = parts[1].trim().parse::<u8>().ok()?;
    let b = parts[2].trim().parse::<u8>().ok()?;
    let a_str = parts[3].trim();
    let a = if let Ok(val) = a_str.parse::<f32>() {
        (val.clamp(0.0, 1.0) * 255.0) as u8
    } else {
        a_str.parse::<u8>().ok()?
    };
    Some(vello::peniko::Color::from_rgba8(r, g, b, a))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_colors() {
        assert_eq!(parse_color("#eee"), Some(vello::peniko::Color::from_rgb8(238, 238, 238)));
        assert_eq!(parse_color("#348"), Some(vello::peniko::Color::from_rgb8(0x33, 0x44, 0x88)));
        assert_eq!(parse_color("#ffffff"), Some(vello::peniko::Color::from_rgb8(255, 255, 255)));
        assert_eq!(parse_color("#f0f0f2"), Some(vello::peniko::Color::from_rgb8(0xf0, 0xf0, 0xf2)));
    }

    #[test]
    fn test_rgb_functions() {
        assert_eq!(parse_color("rgb(10, 20, 30)"), Some(vello::peniko::Color::from_rgb8(10, 20, 30)));
        assert_eq!(parse_color("rgba(10, 20, 30, 0.5)"), Some(vello::peniko::Color::from_rgba8(10, 20, 30, 127)));
    }
}

