pub fn parse_color(value: &str) -> Option<vello::peniko::Color> {
    match value.trim() {
        "red" => Some(vello::peniko::Color::from_rgb8(255, 0, 0)),
        "green" => Some(vello::peniko::Color::from_rgb8(0, 255, 0)),
        "blue" => Some(vello::peniko::Color::from_rgb8(0, 0, 255)),
        "black" => Some(vello::peniko::Color::from_rgb8(0, 0, 0)),
        "white" => Some(vello::peniko::Color::from_rgb8(255, 255, 255)),
        "transparent" => Some(vello::peniko::Color::from_rgba8(0, 0, 0, 0)),
        _ => None,
    }
}
