use crate::types::{BoxDimensions, LayoutBox};
use quint_style::StyledNode;
use ab_glyph::{FontRef, Font, PxScale, ScaleFont};
use std::sync::OnceLock;

static FONT: OnceLock<FontRef> = OnceLock::new();

fn get_font() -> &'static FontRef<'static> {
    FONT.get_or_init(|| {
        let font_data = include_bytes!("Roboto-Regular.ttf");
        FontRef::try_from_slice(font_data).unwrap()
    })
}

pub fn parse_length(value: &str, container_dimension: f32) -> Option<f32> {
    let trimmed = value.trim();
    if let Some(stripped) = trimmed.strip_suffix("px") {
        stripped.parse().ok()
    } else if let Some(stripped) = trimmed.strip_suffix("vw") {
        stripped
            .parse::<f32>()
            .ok()
            .map(|v| v * container_dimension / 100.0)
    } else if let Some(stripped) = trimmed.strip_suffix("vh") {
        stripped.parse::<f32>().ok().map(|v| v * 600.0 / 100.0)
    } else if let Some(stripped) = trimmed.strip_suffix('%') {
        stripped
            .parse::<f32>()
            .ok()
            .map(|v| v * container_dimension / 100.0)
    } else if let Some(stripped) = trimmed
        .strip_suffix("em")
        .or_else(|| trimmed.strip_suffix("rem"))
    {
        stripped.parse::<f32>().ok().map(|v| v * 16.0)
    } else if let Some(stripped) = trimmed.strip_suffix("pt") {
        stripped.parse::<f32>().ok().map(|v| v * 96.0 / 72.0)
    } else {
        trimmed.parse().ok()
    }
}

pub fn to_px(value: &str) -> f32 {
    parse_length(value, 800.0).unwrap_or(0.0)
}

pub fn lookup_px(styled: &StyledNode, property: &str) -> f32 {
    styled
        .properties
        .get(property)
        .map(|v| to_px(v))
        .unwrap_or(0.0)
}

pub fn is_auto(styled: &StyledNode, property: &str) -> bool {
    styled
        .properties
        .get(property)
        .map(|v| v.trim() == "auto")
        .unwrap_or(false)
}

pub fn is_hidden(styled: &StyledNode) -> bool {
    if styled.properties.get("display").map(|v| v.trim()) == Some("none") {
        return true;
    }
    if let quint_html::Node::Element { tag, .. } = styled.node {
        matches!(
            tag.to_ascii_lowercase().as_str(),
            "head" | "title" | "style" | "script" | "link" | "meta" | "noscript"
        )
    } else {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarginValue {
    Auto,
    Px(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

pub fn get_margin_val(styled: &StyledNode, side: Side, container_width: f32) -> MarginValue {
    let specific_prop = match side {
        Side::Top => "margin-top",
        Side::Right => "margin-right",
        Side::Bottom => "margin-bottom",
        Side::Left => "margin-left",
    };

    if let Some(val) = styled.properties.get(specific_prop) {
        let t = val.trim();
        if t == "auto" {
            return MarginValue::Auto;
        }
        return MarginValue::Px(parse_length(t, container_width).unwrap_or(0.0));
    }

    if let Some(val) = styled.properties.get("margin") {
        let tokens: Vec<&str> = val.split_whitespace().collect();
        let token = match tokens.len() {
            1 => tokens.first().copied(),
            2 => match side {
                Side::Top | Side::Bottom => tokens.first().copied(),
                Side::Right | Side::Left => tokens.get(1).copied(),
            },
            3 => match side {
                Side::Top => tokens.first().copied(),
                Side::Right | Side::Left => tokens.get(1).copied(),
                Side::Bottom => tokens.get(2).copied(),
            },
            4 => match side {
                Side::Top => tokens.first().copied(),
                Side::Right => tokens.get(1).copied(),
                Side::Bottom => tokens.get(2).copied(),
                Side::Left => tokens.get(3).copied(),
            },
            _ => None,
        };

        if let Some(t) = token {
            let t = t.trim();
            if t == "auto" {
                return MarginValue::Auto;
            }
            return MarginValue::Px(parse_length(t, container_width).unwrap_or(0.0));
        }
    }

    MarginValue::Px(0.0)
}

pub fn get_padding_val(styled: &StyledNode, side: Side, container_width: f32) -> f32 {
    let specific_prop = match side {
        Side::Top => "padding-top",
        Side::Right => "padding-right",
        Side::Bottom => "padding-bottom",
        Side::Left => "padding-left",
    };

    if let Some(val) = styled.properties.get(specific_prop) {
        return parse_length(val, container_width).unwrap_or(0.0);
    }

    if let Some(val) = styled.properties.get("padding") {
        let tokens: Vec<&str> = val.split_whitespace().collect();
        let token = match tokens.len() {
            1 => tokens.first().copied(),
            2 => match side {
                Side::Top | Side::Bottom => tokens.first().copied(),
                Side::Right | Side::Left => tokens.get(1).copied(),
            },
            3 => match side {
                Side::Top => tokens.first().copied(),
                Side::Right | Side::Left => tokens.get(1).copied(),
                Side::Bottom => tokens.get(2).copied(),
            },
            4 => match side {
                Side::Top => tokens.first().copied(),
                Side::Right => tokens.get(1).copied(),
                Side::Bottom => tokens.get(2).copied(),
                Side::Left => tokens.get(3).copied(),
            },
            _ => None,
        };

        if let Some(t) = token {
            return parse_length(t, container_width).unwrap_or(0.0);
        }
    }

    0.0
}

pub fn layout_block<'a>(
    styled_node: &'a StyledNode<'a>,
    containing_block: &mut BoxDimensions,
) -> LayoutBox<'a> {
    let mut layout_box = LayoutBox {
        dimensions: BoxDimensions::default(),
        styled_node,
        children: vec![],
    };

    calculate_block_width(&mut layout_box, containing_block);
    calculate_block_position(&mut layout_box, containing_block);
    layout_children(&mut layout_box);
    calculate_block_height(&mut layout_box);

    layout_box
}

fn calculate_block_width(layout_box: &mut LayoutBox, containing_block: &BoxDimensions) {
    let styled = layout_box.styled_node;
    let container_width = containing_block.content.width;

    let width = styled.properties.get("width").map(|v| v.as_str());

    let ml_val = get_margin_val(styled, Side::Left, container_width);
    let mr_val = get_margin_val(styled, Side::Right, container_width);
    let border_left = lookup_px(styled, "border-left-width");
    let border_right = lookup_px(styled, "border-right-width");
    let padding_left = get_padding_val(styled, Side::Left, container_width);
    let padding_right = get_padding_val(styled, Side::Right, container_width);

    let margin_left_px = match ml_val {
        MarginValue::Px(v) => v,
        MarginValue::Auto => 0.0,
    };
    let margin_right_px = match mr_val {
        MarginValue::Px(v) => v,
        MarginValue::Auto => 0.0,
    };

    let total = margin_left_px
        + border_left
        + padding_left
        + border_right
        + padding_right
        + margin_right_px;

    let parsed_width = width
        .filter(|&w| w.trim() != "auto")
        .and_then(|w| parse_length(w, container_width));

    let (content_width, final_margin_left, final_margin_right) = if let Some(w) = parsed_width {
        let remaining = container_width - total - w;

        let ml_auto = ml_val == MarginValue::Auto;
        let mr_auto = mr_val == MarginValue::Auto;

        match (ml_auto, mr_auto) {
            (true, true) => {
                let m = (remaining / 2.0).max(0.0);
                (w, m, m)
            }
            (true, false) => (w, remaining.max(0.0), margin_right_px),
            (false, true) => (w, margin_left_px, remaining.max(0.0)),
            (false, false) => (w, margin_left_px, margin_right_px + remaining),
        }
    } else {
        let auto_width = (container_width - total).max(0.0);
        (auto_width, margin_left_px, margin_right_px)
    };

    let d = &mut layout_box.dimensions;
    d.content.width = content_width;
    d.padding.left = padding_left;
    d.padding.right = padding_right;
    d.border.left = border_left;
    d.border.right = border_right;
    d.margin.left = final_margin_left;
    d.margin.right = final_margin_right;
}

fn calculate_block_position(layout_box: &mut LayoutBox, containing_block: &BoxDimensions) {
    let styled = layout_box.styled_node;
    let d = &mut layout_box.dimensions;
    let container_width = containing_block.content.width;

    d.margin.top = match get_margin_val(styled, Side::Top, container_width) {
        MarginValue::Px(v) => v,
        MarginValue::Auto => 0.0,
    };
    d.margin.bottom = match get_margin_val(styled, Side::Bottom, container_width) {
        MarginValue::Px(v) => v,
        MarginValue::Auto => 0.0,
    };
    d.border.top = lookup_px(styled, "border-top-width");
    d.border.bottom = lookup_px(styled, "border-bottom-width");
    d.padding.top = get_padding_val(styled, Side::Top, container_width);
    d.padding.bottom = get_padding_val(styled, Side::Bottom, container_width);

    d.content.x = containing_block.content.x + d.margin.left + d.border.left + d.padding.left;
    d.content.y = containing_block.content.y
        + containing_block.content.height
        + d.margin.top
        + d.border.top
        + d.padding.top;
}

fn layout_children(layout_box: &mut LayoutBox) {
    for child_styled in &layout_box.styled_node.children {
        if is_hidden(child_styled) {
            continue;
        }
        let child_layout = layout_block(child_styled, &mut layout_box.dimensions);
        layout_box.dimensions.content.height += child_layout.dimensions.margin_box().height;
        layout_box.children.push(child_layout);
    }
}

fn calculate_block_height(layout_box: &mut LayoutBox) {
    if let Some(h) = layout_box.styled_node.properties.get("height") {
        let h = h.trim();
        if h != "auto" {
            let px = to_px(h);
            if px > 0.0 {
                layout_box.dimensions.content.height = px;
                return;
            }
        }
    }

    if let quint_html::Node::Text(text) = &layout_box.styled_node.node {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            layout_box.dimensions.content.height = 0.0;
            return;
        }

        let font = get_font();
        let font_size = 16.0;
        let scale = PxScale { x: font_size, y: font_size };
        
        let v_metrics = font.as_scaled(scale).ascent() - font.as_scaled(scale).descent() + font.as_scaled(scale).line_gap();
        let line_height = v_metrics * 1.2;
        let space_width = font.as_scaled(scale).h_advance(font.glyph_id(' '));
        let max_width = layout_box.dimensions.content.width.max(space_width);

        let mut lines = 1;
        let mut cur_x = 0.0;

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

            if cur_x > 0.0 && cur_x + word_width > max_width {
                lines += 1;
                cur_x = 0.0;
            }
            cur_x += word_width;
            if i + 1 < words.len() {
                cur_x += space_width;
            }
        }

        layout_box.dimensions.content.height = lines as f32 * line_height;
    }
}
