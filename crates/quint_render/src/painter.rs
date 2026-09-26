use crate::color::parse_color;
use ab_glyph::{FontRef, Font, PxScale, ScaleFont, point, OutlineCurve};
use quint_layout::LayoutBox;
use vello::kurbo::{Affine, Rect, BezPath, PathEl};
use vello::peniko::{Color, Fill};
use vello::Scene;
use std::sync::OnceLock;

static FONT: OnceLock<FontRef> = OnceLock::new();

fn get_font() -> &'static FontRef<'static> {
    FONT.get_or_init(|| {
        let font_data = include_bytes!("Roboto-Regular.ttf");
        FontRef::try_from_slice(font_data).unwrap()
    })
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

    if let Some(color) = background_prop.and_then(|c| parse_color(c)) {
        let rect = layout_box.dimensions.border_box();
        if rect.width > 0.0 && rect.height > 0.0 {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                color,
                None,
                &Rect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64),
            );
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

        let font_size = layout_box
            .styled_node
            .properties
            .get("font-size")
            .and_then(|s| quint_layout::parse_length(s, 16.0))
            .unwrap_or(16.0);

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
    let scale = PxScale { x: font_size, y: font_size };
    let v_metrics = font.as_scaled(scale).ascent() - font.as_scaled(scale).descent() + font.as_scaled(scale).line_gap();
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
            
            let _glyph = glyph_id.with_scale_and_position(scale, point(cursor_x, cursor_y));
            
            if let Some(outline) = font.outline(glyph_id) {
                let mut path = BezPath::new();
                let mut current_point: Option<vello::kurbo::Point> = None;
                for curve in outline.curves {
                    match curve {
                        OutlineCurve::Line(p0, p1) => {
                            let start = vello::kurbo::Point::new(p0.x as f64, p0.y as f64);
                            let end = vello::kurbo::Point::new(p1.x as f64, p1.y as f64);
                            if current_point.map_or(true, |cp| cp != start) {
                                path.push(PathEl::MoveTo(start));
                            }
                            path.push(PathEl::LineTo(end));
                            current_point = Some(end);
                        }
                        OutlineCurve::Quad(p0, p1, p2) => {
                            let start = vello::kurbo::Point::new(p0.x as f64, p0.y as f64);
                            let ctrl = vello::kurbo::Point::new(p1.x as f64, p1.y as f64);
                            let end = vello::kurbo::Point::new(p2.x as f64, p2.y as f64);
                            if current_point.map_or(true, |cp| cp != start) {
                                path.push(PathEl::MoveTo(start));
                            }
                            path.push(PathEl::QuadTo(ctrl, end));
                            current_point = Some(end);
                        }
                        OutlineCurve::Cubic(p0, p1, p2, p3) => {
                            let start = vello::kurbo::Point::new(p0.x as f64, p0.y as f64);
                            let ctrl1 = vello::kurbo::Point::new(p1.x as f64, p1.y as f64);
                            let ctrl2 = vello::kurbo::Point::new(p2.x as f64, p2.y as f64);
                            let end = vello::kurbo::Point::new(p3.x as f64, p3.y as f64);
                            if current_point.map_or(true, |cp| cp != start) {
                                path.push(PathEl::MoveTo(start));
                            }
                            path.push(PathEl::CurveTo(ctrl1, ctrl2, end));
                            current_point = Some(end);
                        }
                    }
                }
                
                // The curves are in unscaled font units. We need to scale and translate them!
                let scale_factor = font.as_scaled(scale).scale_factor();
                
                let transform = Affine::translate((cursor_x as f64, cursor_y as f64))
                    * Affine::scale_non_uniform(scale_factor.horizontal as f64, -scale_factor.vertical as f64);
                
                scene.fill(
                    Fill::NonZero,
                    transform,
                    color,
                    None,
                    &path,
                );
            }
            cursor_x += font.as_scaled(scale).h_advance(glyph_id);
            last_glyph = Some(glyph_id);
        }

        if i + 1 < words.len() {
            cursor_x += space_width;
        }
    }
}
