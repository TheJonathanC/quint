use crate::color::parse_color;
use ab_glyph::{FontRef, Font, PxScale, ScaleFont, OutlineCurve, GlyphId};
use quint_layout::{LayoutBox, parse_length};
use vello::kurbo::{Affine, Rect, BezPath, PathEl, Point};
use vello::peniko::{Color, Fill};
use vello::Scene;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static FONT: OnceLock<FontRef> = OnceLock::new();
static GLYPH_CACHE: OnceLock<Mutex<HashMap<u16, BezPath>>> = OnceLock::new();

fn get_font() -> &'static FontRef<'static> {
    FONT.get_or_init(|| {
        let font_data = include_bytes!("Roboto-Regular.ttf");
        FontRef::try_from_slice(font_data).unwrap()
    })
}

fn get_glyph_path(font: &FontRef, glyph_id: GlyphId) -> Option<BezPath> {
    let cache = GLYPH_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = glyph_id.0;

    if let Ok(guard) = cache.lock()
        && let Some(path) = guard.get(&key)
    {
        return Some(path.clone());
    }

    let outline = font.outline(glyph_id)?;
    let mut path = BezPath::new();
    let mut last_pt: Option<Point> = None;

    for curve in outline.curves {
        match curve {
            OutlineCurve::Line(p0, p1) => {
                let start = Point::new(p0.x as f64, p0.y as f64);
                let end = Point::new(p1.x as f64, p1.y as f64);
                if last_pt.is_none() || last_pt != Some(start) {
                    path.push(PathEl::MoveTo(start));
                }
                path.push(PathEl::LineTo(end));
                last_pt = Some(end);
            }
            OutlineCurve::Quad(p0, p1, p2) => {
                let start = Point::new(p0.x as f64, p0.y as f64);
                let ctrl = Point::new(p1.x as f64, p1.y as f64);
                let end = Point::new(p2.x as f64, p2.y as f64);
                if last_pt.is_none() || last_pt != Some(start) {
                    path.push(PathEl::MoveTo(start));
                }
                path.push(PathEl::QuadTo(ctrl, end));
                last_pt = Some(end);
            }
            OutlineCurve::Cubic(p0, p1, p2, p3) => {
                let start = Point::new(p0.x as f64, p0.y as f64);
                let c1 = Point::new(p1.x as f64, p1.y as f64);
                let c2 = Point::new(p2.x as f64, p2.y as f64);
                let end = Point::new(p3.x as f64, p3.y as f64);
                if last_pt.is_none() || last_pt != Some(start) {
                    path.push(PathEl::MoveTo(start));
                }
                path.push(PathEl::CurveTo(c1, c2, end));
                last_pt = Some(end);
            }
        }
    }
    path.push(PathEl::ClosePath);

    if let Ok(mut guard) = cache.lock() {
        guard.insert(key, path.clone());
    }

    Some(path)
}

pub fn paint_tree(layout_tree: &[LayoutBox], scene: &mut Scene) {
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::WHITE,
        None,
        &Rect::new(0.0, 0.0, 4000.0, 4000.0),
    );
    for box_node in layout_tree {
        paint_box(box_node, scene);
    }
}

fn paint_box(layout_box: &LayoutBox, scene: &mut Scene) {
    let background_prop = layout_box
        .styled_node
        .properties
        .get("background-color")
        .or_else(|| layout_box.styled_node.properties.get("background"));

    // Paint background
    if let Some(color) = background_prop.and_then(|c| parse_color(c)) {
        let rect = layout_box.dimensions.border_box();
        if rect.width > 0.0 && rect.height > 0.0 {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                color,
                None,
                &Rect::new(
                    rect.x as f64,
                    rect.y as f64,
                    (rect.x + rect.width) as f64,
                    (rect.y + rect.height) as f64,
                ),
            );
        }
    }

    // Paint borders
    let border = &layout_box.dimensions.border;
    if border.top > 0.0 || border.right > 0.0 || border.bottom > 0.0 || border.left > 0.0 {
        let bbox = layout_box.dimensions.border_box();
        let fallback_color_str = layout_box
            .styled_node
            .properties
            .get("color")
            .map(|s| s.as_str())
            .unwrap_or("black");
        let fallback_color = parse_color(fallback_color_str).unwrap_or(Color::BLACK);

        let top_color = layout_box
            .styled_node
            .properties
            .get("border-top-color")
            .or_else(|| layout_box.styled_node.properties.get("border-color"))
            .and_then(|s| parse_color(s))
            .unwrap_or(fallback_color);
        let right_color = layout_box
            .styled_node
            .properties
            .get("border-right-color")
            .or_else(|| layout_box.styled_node.properties.get("border-color"))
            .and_then(|s| parse_color(s))
            .unwrap_or(fallback_color);
        let bottom_color = layout_box
            .styled_node
            .properties
            .get("border-bottom-color")
            .or_else(|| layout_box.styled_node.properties.get("border-color"))
            .and_then(|s| parse_color(s))
            .unwrap_or(fallback_color);
        let left_color = layout_box
            .styled_node
            .properties
            .get("border-left-color")
            .or_else(|| layout_box.styled_node.properties.get("border-color"))
            .and_then(|s| parse_color(s))
            .unwrap_or(fallback_color);

        if border.top > 0.0 {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                top_color,
                None,
                &Rect::new(
                    bbox.x as f64,
                    bbox.y as f64,
                    (bbox.x + bbox.width) as f64,
                    (bbox.y + border.top) as f64,
                ),
            );
        }
        if border.bottom > 0.0 {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                bottom_color,
                None,
                &Rect::new(
                    bbox.x as f64,
                    (bbox.y + bbox.height - border.bottom) as f64,
                    (bbox.x + bbox.width) as f64,
                    (bbox.y + bbox.height) as f64,
                ),
            );
        }
        if border.left > 0.0 {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                left_color,
                None,
                &Rect::new(
                    bbox.x as f64,
                    bbox.y as f64,
                    (bbox.x + border.left) as f64,
                    (bbox.y + bbox.height) as f64,
                ),
            );
        }
        if border.right > 0.0 {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                right_color,
                None,
                &Rect::new(
                    (bbox.x + bbox.width - border.right) as f64,
                    bbox.y as f64,
                    (bbox.x + bbox.width) as f64,
                    (bbox.y + bbox.height) as f64,
                ),
            );
        }
    }

    // Paint text
    if let quint_html::Node::Text(text) = &layout_box.styled_node.node {
        let color_str = layout_box
            .styled_node
            .properties
            .get("color")
            .map(|s| s.as_str())
            .unwrap_or("black");
        let color = parse_color(color_str).unwrap_or(Color::BLACK);

        let font_size = layout_box
            .styled_node
            .properties
            .get("font-size")
            .and_then(|v| parse_length(v, 800.0))
            .unwrap_or(16.0)
            .max(4.0);

        draw_text(
            scene,
            text,
            layout_box.dimensions.content.x,
            layout_box.dimensions.content.y,
            layout_box.dimensions.content.width,
            font_size,
            color,
        );
    }

    for child in &layout_box.children {
        paint_box(child, scene);
    }
}

fn draw_text(
    scene: &mut Scene,
    text: &str,
    start_x: f32,
    start_y: f32,
    max_box_width: f32,
    font_size: f32,
    color: Color,
) {
    let font = get_font();
    let scale = PxScale {
        x: font_size,
        y: font_size,
    };
    let v_metrics = font.as_scaled(scale).ascent() - font.as_scaled(scale).descent()
        + font.as_scaled(scale).line_gap();
    let line_height = v_metrics * 1.2;

    let max_x = if max_box_width > 0.0 {
        start_x + max_box_width
    } else {
        10000.0
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

            if let Some(path) = get_glyph_path(font, glyph_id) {
                let scale_factor = font.as_scaled(scale).scale_factor();
                let transform = Affine::translate((cursor_x as f64, cursor_y as f64))
                    * Affine::scale_non_uniform(
                        scale_factor.horizontal as f64,
                        -scale_factor.vertical as f64,
                    );

                scene.fill(Fill::NonZero, transform, color, None, &path);
            }
            cursor_x += font.as_scaled(scale).h_advance(glyph_id);
            last_glyph = Some(glyph_id);
        }

        if i + 1 < words.len() {
            cursor_x += space_width;
        }
    }
}
