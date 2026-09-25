use crate::color::parse_color;
use ab_glyph::{FontRef, Font, PxScale, point, ScaleFont};
use quint_layout::LayoutBox;
use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};
use std::sync::OnceLock;

static FONT: OnceLock<FontRef> = OnceLock::new();

fn get_font() -> &'static FontRef<'static> {
    FONT.get_or_init(|| {
        let font_data = include_bytes!("Roboto-Regular.ttf");
        FontRef::try_from_slice(font_data).unwrap()
    })
}

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
        if rect.width > 0.0 && rect.height > 0.0 {
            if let Some(skia_rect) = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height) {
                pixmap.fill_rect(skia_rect, &paint, Transform::identity(), None);
            }
        }
    }

    if let quint_html::Node::Text(text) = &layout_box.styled_node.node {
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
    let font = get_font();
    let font_size = 16.0;
    let scale = PxScale { x: font_size, y: font_size };
    let v_metrics = font.as_scaled(scale).ascent() - font.as_scaled(scale).descent() + font.as_scaled(scale).line_gap();
    let line_height = v_metrics * 1.2;
    
    let max_x = if max_box_width > 0.0 {
        (start_x + max_box_width).min(pixmap.width() as f32)
    } else {
        pixmap.width() as f32
    };

    let mut cursor_x = start_x;
    let mut cursor_y = start_y + font.as_scaled(scale).ascent();

    let words: Vec<&str> = text.split_whitespace().collect();
    let space_width = font.as_scaled(scale).h_advance(font.glyph_id(' '));

    for (i, word) in words.iter().enumerate() {
        let mut word_width = 0.0;
        let mut last_glyph = None;
        for ch in word.chars() {
            let glyph_id = font.glyph_id(ch);
            word_width += font.as_scaled(scale).h_advance(glyph_id);
            if let Some(last) = last_glyph {
                word_width += font.as_scaled(scale).kern(last, glyph_id);
            }
            last_glyph = Some(glyph_id);
        }

        if cursor_x > start_x && cursor_x + word_width > max_x {
            cursor_x = start_x;
            cursor_y += line_height;
        }

        last_glyph = None;
        for ch in word.chars() {
            let glyph_id = font.glyph_id(ch);
            if let Some(last) = last_glyph {
                cursor_x += font.as_scaled(scale).kern(last, glyph_id);
            }
            
            let glyph = glyph_id.with_scale_and_position(scale, point(cursor_x, cursor_y));
            if let Some(outline) = font.outline_glyph(glyph) {
                let bounds = outline.px_bounds();
                outline.draw(|x, y, v| {
                    let px = (bounds.min.x + x as f32) as u32;
                    let py = (bounds.min.y + y as f32) as u32;
                    let width = pixmap.width();
                    let height = pixmap.height();
                    if px < width && py < height {
                        let idx = (py * width + px) as usize;
                        let p = &mut pixmap.pixels_mut()[idx];
                        let r = ((color.red() as f32 * v) + (p.red() as f32 * (1.0 - v))) as u8;
                        let g = ((color.green() as f32 * v) + (p.green() as f32 * (1.0 - v))) as u8;
                        let b = ((color.blue() as f32 * v) + (p.blue() as f32 * (1.0 - v))) as u8;
                        let a = p.alpha();
                        *p = tiny_skia::PremultipliedColorU8::from_rgba(r, g, b, 255).unwrap_or(*p);
                    }
                });
            }
            cursor_x += font.as_scaled(scale).h_advance(glyph_id);
            last_glyph = Some(glyph_id);
        }

        if i + 1 < words.len() {
            cursor_x += space_width;
        }
    }
}
