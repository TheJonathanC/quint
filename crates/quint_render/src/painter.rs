use crate::color::parse_color;
use font8x8::UnicodeFonts;
use quint_layout::LayoutBox;
use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

pub fn paint_tree(layout_tree: &[LayoutBox], pixmap: &mut Pixmap) {
    pixmap.fill(Color::WHITE);
    for box_node in layout_tree {
        paint_box(box_node, pixmap);
    }
}

fn paint_box(layout_box: &LayoutBox, pixmap: &mut Pixmap) {
    let background_prop = layout_box
        .styled_node
        .properties
        .get("background-color")
        .or_else(|| layout_box.styled_node.properties.get("background"));

    if let Some(color) = background_prop.and_then(|c| parse_color(c)) {
        let mut paint = Paint::default();
        paint.set_color(color);

        let rect = layout_box.dimensions.border_box();
        if rect.width > 0.0
            && rect.height > 0.0
            && let Some(skia_rect) = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height)
        {
            pixmap.fill_rect(skia_rect, &paint, Transform::identity(), None);
        }
    }

    if let quint_html::Node::Text(text) = layout_box.styled_node.node {
        let color_str = layout_box
            .styled_node
            .properties
            .get("color")
            .map(|s| s.as_str())
            .unwrap_or("black");
        let color = parse_color(color_str).unwrap_or(Color::BLACK);

        draw_text(
            pixmap,
            text,
            layout_box.dimensions.content.x,
            layout_box.dimensions.content.y,
            layout_box.dimensions.content.width,
            color,
        );
    }

    for child in &layout_box.children {
        paint_box(child, pixmap);
    }
}

fn draw_text(
    pixmap: &mut Pixmap,
    text: &str,
    start_x: f32,
    start_y: f32,
    max_box_width: f32,
    color: Color,
) {
    let scale = 2.0;
    let char_width = 8.0 * scale;
    let line_height = 20.0;
    let max_x = if max_box_width > 0.0 {
        (start_x + max_box_width).min(pixmap.width() as f32)
    } else {
        pixmap.width() as f32
    };

    let mut cursor_x = start_x;
    let mut cursor_y = start_y;

    let mut paint = Paint::default();
    paint.set_color(color);

    let words: Vec<&str> = text.split_whitespace().collect();
    for (i, word) in words.iter().enumerate() {
        let word_width = word.chars().count() as f32 * char_width;

        if cursor_x > start_x && cursor_x + word_width > max_x {
            cursor_x = start_x;
            cursor_y += line_height;
        }

        for ch in word.chars() {
            if cursor_x + char_width > max_x && cursor_x > start_x {
                cursor_x = start_x;
                cursor_y += line_height;
            }

            if cursor_y + (8.0 * scale) > pixmap.height() as f32 {
                return;
            }

            if let Some(bitmap) = font8x8::BASIC_FONTS
                .get(ch)
                .or_else(|| font8x8::LATIN_FONTS.get(ch))
            {
                for (y, row) in bitmap.iter().enumerate() {
                    for x in 0..8 {
                        if (*row & (1 << x)) != 0 {
                            let px = cursor_x + (x as f32 * scale);
                            let py = cursor_y + (y as f32 * scale);

                            if let Some(rect) = Rect::from_xywh(px, py, scale, scale) {
                                pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                            }
                        }
                    }
                }
            }
            cursor_x += char_width;
        }

        if i + 1 < words.len() {
            cursor_x += char_width;
        }
    }
}
