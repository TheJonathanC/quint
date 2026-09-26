use crate::types::{BoxDimensions, EdgeSizes, LayoutBox, Rect};
use quint_html::Node;
use quint_style::StyledNode;
use taffy::prelude::*;
use taffy::style::{Dimension, LengthPercentage, LengthPercentageAuto, FlexDirection, Display};
use taffy::geometry::Size;
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
        stripped.parse::<f32>().ok().map(|v| v * container_dimension / 100.0)
    } else if let Some(stripped) = trimmed.strip_suffix("vh") {
        stripped.parse::<f32>().ok().map(|v| v * 600.0 / 100.0)
    } else if let Some(stripped) = trimmed.strip_suffix('%') {
        stripped.parse::<f32>().ok().map(|v| v * container_dimension / 100.0)
    } else if let Some(stripped) = trimmed.strip_suffix("em").or_else(|| trimmed.strip_suffix("rem")) {
        stripped.parse::<f32>().ok().map(|v| v * 16.0)
    } else if let Some(stripped) = trimmed.strip_suffix("pt") {
        stripped.parse::<f32>().ok().map(|v| v * 96.0 / 72.0)
    } else {
        trimmed.parse().ok()
    }
}

pub fn is_hidden(styled: &StyledNode) -> bool {
    if let Some(display) = styled.properties.get("display") {
        if display == "none" {
            return true;
        }
    }
    false
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EdgeSide {
    Top,
    Right,
    Bottom,
    Left,
}

pub fn get_shorthand_edge<'a>(
    properties: &'a std::collections::HashMap<String, String>,
    shorthand_name: &str,
    side: EdgeSide,
) -> Option<&'a str> {
    let specific_name = match side {
        EdgeSide::Top => format!("{}-top", shorthand_name),
        EdgeSide::Right => format!("{}-right", shorthand_name),
        EdgeSide::Bottom => format!("{}-bottom", shorthand_name),
        EdgeSide::Left => format!("{}-left", shorthand_name),
    };
    if let Some(val) = properties.get(&specific_name) {
        return Some(val.as_str());
    }

    if let Some(val) = properties.get(shorthand_name) {
        let parts: Vec<&str> = val.split_whitespace().collect();
        match parts.len() {
            1 => Some(parts[0]),
            2 => match side {
                EdgeSide::Top | EdgeSide::Bottom => Some(parts[0]),
                EdgeSide::Right | EdgeSide::Left => Some(parts[1]),
            },
            3 => match side {
                EdgeSide::Top => Some(parts[0]),
                EdgeSide::Right | EdgeSide::Left => Some(parts[1]),
                EdgeSide::Bottom => Some(parts[2]),
            },
            4 => match side {
                EdgeSide::Top => Some(parts[0]),
                EdgeSide::Right => Some(parts[1]),
                EdgeSide::Bottom => Some(parts[2]),
                EdgeSide::Left => Some(parts[3]),
            },
            _ => None,
        }
    } else {
        None
    }
}

fn to_length(v: Option<f32>) -> LengthPercentage {
    match v {
        Some(val) => LengthPercentage::Points(val),
        None => LengthPercentage::Points(0.0),
    }
}

fn to_length_auto(v: Option<&str>, container_dim: f32) -> LengthPercentageAuto {
    match v {
        Some("auto") => LengthPercentageAuto::Auto,
        Some(val) => {
            if let Some(px) = parse_length(val, container_dim) {
                LengthPercentageAuto::Points(px)
            } else {
                LengthPercentageAuto::Auto
            }
        }
        None => LengthPercentageAuto::Points(0.0),
    }
}

pub fn layout_tree<'a>(
    styled_nodes: &'a [StyledNode<'a>],
    width: f32,
) -> Vec<LayoutBox<'a>> {
    let height = 600.0;
    let mut taffy = Taffy::new();

    let mut root_style = Style::default();
    root_style.size.width = Dimension::Points(width);
    root_style.display = Display::Flex;
    root_style.flex_direction = FlexDirection::Column;

    let root_node = taffy.new_leaf(root_style).unwrap();
    let mut child_mapping = Vec::new();

    for styled in styled_nodes {
        if is_hidden(styled) { continue; }
        let node = build_taffy_tree(&mut taffy, styled, width, height);
        taffy.add_child(root_node, node).unwrap();
        child_mapping.push((styled, node));
    }

    taffy.compute_layout(root_node, Size::MAX_CONTENT).unwrap();

    let mut boxes = Vec::new();
    let root_layout = taffy.layout(root_node).unwrap().clone();
    for (styled, node) in child_mapping {
        let b = build_layout_box(
            &taffy,
            node,
            styled,
            root_layout.location.x,
            root_layout.location.y,
            width,
            height,
        );
        boxes.push(b);
    }

    boxes
}

fn build_taffy_tree(
    taffy: &mut Taffy,
    styled: &StyledNode,
    viewport_width: f32,
    viewport_height: f32,
) -> taffy::prelude::Node {
    let mut style = Style::default();
    style.display = Display::Flex;
    style.flex_direction = FlexDirection::Column;

    if let Some(disp) = styled.properties.get("display") {
        if disp == "flex" {
            style.display = Display::Flex;
            style.flex_direction = FlexDirection::Row;
        } else if disp == "grid" {
            style.display = Display::Grid;
        }
    }

    if let Some(w) = styled.properties.get("width") {
        if let Some(px) = parse_length(w, viewport_width) {
            style.size.width = Dimension::Points(px);
        }
    }
    if let Some(h) = styled.properties.get("height") {
        if let Some(px) = parse_length(h, viewport_height) {
            style.size.height = Dimension::Points(px);
        }
    }

    let margin_left = get_shorthand_edge(&styled.properties, "margin", EdgeSide::Left);
    let margin_right = get_shorthand_edge(&styled.properties, "margin", EdgeSide::Right);
    let margin_top = get_shorthand_edge(&styled.properties, "margin", EdgeSide::Top);
    let margin_bottom = get_shorthand_edge(&styled.properties, "margin", EdgeSide::Bottom);

    style.margin = taffy::geometry::Rect {
        left: to_length_auto(margin_left, viewport_width),
        right: to_length_auto(margin_right, viewport_width),
        top: to_length_auto(margin_top, viewport_height),
        bottom: to_length_auto(margin_bottom, viewport_height),
    };

    let pad_left = get_shorthand_edge(&styled.properties, "padding", EdgeSide::Left)
        .and_then(|v| parse_length(v, viewport_width));
    let pad_right = get_shorthand_edge(&styled.properties, "padding", EdgeSide::Right)
        .and_then(|v| parse_length(v, viewport_width));
    let pad_top = get_shorthand_edge(&styled.properties, "padding", EdgeSide::Top)
        .and_then(|v| parse_length(v, viewport_height));
    let pad_bottom = get_shorthand_edge(&styled.properties, "padding", EdgeSide::Bottom)
        .and_then(|v| parse_length(v, viewport_height));

    style.padding = taffy::geometry::Rect {
        left: to_length(pad_left),
        right: to_length(pad_right),
        top: to_length(pad_top),
        bottom: to_length(pad_bottom),
    };
    
    let border_left = get_shorthand_edge(&styled.properties, "border-width", EdgeSide::Left)
        .or_else(|| styled.properties.get("border-left-width").map(|s| s.as_str()))
        .and_then(|v| parse_length(v, viewport_width));
    let border_right = get_shorthand_edge(&styled.properties, "border-width", EdgeSide::Right)
        .or_else(|| styled.properties.get("border-right-width").map(|s| s.as_str()))
        .and_then(|v| parse_length(v, viewport_width));
    let border_top = get_shorthand_edge(&styled.properties, "border-width", EdgeSide::Top)
        .or_else(|| styled.properties.get("border-top-width").map(|s| s.as_str()))
        .and_then(|v| parse_length(v, viewport_height));
    let border_bottom = get_shorthand_edge(&styled.properties, "border-width", EdgeSide::Bottom)
        .or_else(|| styled.properties.get("border-bottom-width").map(|s| s.as_str()))
        .and_then(|v| parse_length(v, viewport_height));

    style.border = taffy::geometry::Rect {
        left: to_length(border_left),
        right: to_length(border_right),
        top: to_length(border_top),
        bottom: to_length(border_bottom),
    };

    let mut children = Vec::new();
    for child in &styled.children {
        if is_hidden(child) { continue; }
        children.push(build_taffy_tree(taffy, child, viewport_width, viewport_height));
    }

    if let Node::Text(text) = &styled.node {
        let text_clone = text.clone();
        let font_size = styled
            .properties
            .get("font-size")
            .and_then(|s| parse_length(s, 16.0))
            .unwrap_or(16.0);

        taffy.new_leaf_with_measure(style, taffy::node::MeasureFunc::Boxed(Box::new(move |_known, available| {
            let font = get_font();
            let scale = PxScale { x: font_size, y: font_size };
            let v_metrics = font.as_scaled(scale).ascent() - font.as_scaled(scale).descent() + font.as_scaled(scale).line_gap();
            let line_height = v_metrics * 1.2;
            let space_width = font.as_scaled(scale).h_advance(font.glyph_id(' '));
            
            let max_w = available.width.unwrap_or(viewport_width);
            
            let mut lines = 1;
            let mut cur_x = 0.0;
            let mut total_max_x = 0.0_f32;

            let words: Vec<&str> = text_clone.split_whitespace().collect();
            if words.is_empty() {
                return Size { width: 0.0, height: 0.0 };
            }

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

                if cur_x > 0.0 && cur_x + word_width > max_w {
                    lines += 1;
                    cur_x = 0.0;
                }
                cur_x += word_width;
                if cur_x > total_max_x {
                    total_max_x = cur_x;
                }
                if i + 1 < words.len() {
                    cur_x += space_width;
                }
            }
            
            Size {
                width: total_max_x,
                height: lines as f32 * line_height,
            }
        }))).unwrap()
    } else {
        taffy.new_with_children(style, &children).unwrap()
    }
}

fn build_layout_box<'a>(
    taffy: &Taffy,
    node: taffy::prelude::Node,
    styled: &'a StyledNode<'a>,
    parent_x: f32,
    parent_y: f32,
    viewport_width: f32,
    viewport_height: f32,
) -> LayoutBox<'a> {
    let layout = taffy.layout(node).unwrap();
    
    let pad_left = get_shorthand_edge(&styled.properties, "padding", EdgeSide::Left)
        .and_then(|v| parse_length(v, viewport_width))
        .unwrap_or(0.0);
    let pad_right = get_shorthand_edge(&styled.properties, "padding", EdgeSide::Right)
        .and_then(|v| parse_length(v, viewport_width))
        .unwrap_or(0.0);
    let pad_top = get_shorthand_edge(&styled.properties, "padding", EdgeSide::Top)
        .and_then(|v| parse_length(v, viewport_height))
        .unwrap_or(0.0);
    let pad_bottom = get_shorthand_edge(&styled.properties, "padding", EdgeSide::Bottom)
        .and_then(|v| parse_length(v, viewport_height))
        .unwrap_or(0.0);
    
    let border_left = get_shorthand_edge(&styled.properties, "border-width", EdgeSide::Left)
        .or_else(|| styled.properties.get("border-left-width").map(|s| s.as_str()))
        .and_then(|v| parse_length(v, viewport_width))
        .unwrap_or(0.0);
    let border_right = get_shorthand_edge(&styled.properties, "border-width", EdgeSide::Right)
        .or_else(|| styled.properties.get("border-right-width").map(|s| s.as_str()))
        .and_then(|v| parse_length(v, viewport_width))
        .unwrap_or(0.0);
    let border_top = get_shorthand_edge(&styled.properties, "border-width", EdgeSide::Top)
        .or_else(|| styled.properties.get("border-top-width").map(|s| s.as_str()))
        .and_then(|v| parse_length(v, viewport_height))
        .unwrap_or(0.0);
    let border_bottom = get_shorthand_edge(&styled.properties, "border-width", EdgeSide::Bottom)
        .or_else(|| styled.properties.get("border-bottom-width").map(|s| s.as_str()))
        .and_then(|v| parse_length(v, viewport_height))
        .unwrap_or(0.0);

    let margin_left = get_shorthand_edge(&styled.properties, "margin", EdgeSide::Left)
        .and_then(|v| parse_length(v, viewport_width))
        .unwrap_or(0.0);
    let margin_right = get_shorthand_edge(&styled.properties, "margin", EdgeSide::Right)
        .and_then(|v| parse_length(v, viewport_width))
        .unwrap_or(0.0);
    let margin_top = get_shorthand_edge(&styled.properties, "margin", EdgeSide::Top)
        .and_then(|v| parse_length(v, viewport_height))
        .unwrap_or(0.0);
    let margin_bottom = get_shorthand_edge(&styled.properties, "margin", EdgeSide::Bottom)
        .and_then(|v| parse_length(v, viewport_height))
        .unwrap_or(0.0);
    
    let border_box_x = parent_x + layout.location.x;
    let border_box_y = parent_y + layout.location.y;

    let content_x = border_box_x + border_left + pad_left;
    let content_y = border_box_y + border_top + pad_top;

    let content_width = (layout.size.width - pad_left - pad_right - border_left - border_right).max(0.0);
    let content_height = (layout.size.height - pad_top - pad_bottom - border_top - border_bottom).max(0.0);

    let mut dimensions = BoxDimensions::default();
    dimensions.content = Rect {
        x: content_x,
        y: content_y,
        width: content_width,
        height: content_height,
    };
    dimensions.padding = EdgeSizes {
        left: pad_left,
        right: pad_right,
        top: pad_top,
        bottom: pad_bottom,
    };
    dimensions.border = EdgeSizes {
        left: border_left,
        right: border_right,
        top: border_top,
        bottom: border_bottom,
    };
    dimensions.margin = EdgeSizes {
        left: margin_left,
        right: margin_right,
        top: margin_top,
        bottom: margin_bottom,
    };
    
    let mut children = Vec::new();
    if let quint_html::Node::Element { .. } = styled.node {
        let taffy_children = taffy.children(node).unwrap();
        for (i, child) in styled.children.iter().filter(|c| !is_hidden(c)).enumerate() {
            if i < taffy_children.len() {
                let built_child = build_layout_box(
                    taffy,
                    taffy_children[i],
                    child,
                    content_x,
                    content_y,
                    viewport_width,
                    viewport_height,
                );
                children.push(built_child);
            }
        }
    }
    
    LayoutBox {
        dimensions,
        styled_node: styled,
        children,
    }
}
