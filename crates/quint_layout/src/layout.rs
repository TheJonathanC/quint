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

fn to_length(v: Option<f32>) -> LengthPercentage {
    match v {
        Some(val) => LengthPercentage::Points(val),
        None => LengthPercentage::Points(0.0),
    }
}

fn to_length_auto(v: Option<&String>) -> LengthPercentageAuto {
    match v.map(|s| s.as_str()) {
        Some("auto") => LengthPercentageAuto::Auto,
        Some(val) => {
            if let Some(px) = parse_length(val, 800.0) {
                LengthPercentageAuto::Points(px)
            } else {
                LengthPercentageAuto::Auto
            }
        },
        None => LengthPercentageAuto::Points(0.0),
    }
}

pub fn layout_tree<'a>(
    styled_nodes: &'a [StyledNode<'a>],
    width: f32,
) -> Vec<LayoutBox<'a>> {
    let mut taffy = Taffy::new();

    let mut root_style = Style::default();
    root_style.size.width = Dimension::Points(width);
    root_style.display = Display::Flex;
    root_style.flex_direction = FlexDirection::Column;

    let root_node = taffy.new_leaf(root_style).unwrap();
    let mut child_mapping = Vec::new();

    for styled in styled_nodes {
        if is_hidden(styled) { continue; }
        let node = build_taffy_tree(&mut taffy, styled);
        taffy.add_child(root_node, node).unwrap();
        child_mapping.push((styled, node));
    }

    taffy.compute_layout(root_node, Size::MAX_CONTENT).unwrap();

    let mut boxes = Vec::new();
    let root_layout = taffy.layout(root_node).unwrap().clone();
    for (styled, node) in child_mapping {
        let mut b = build_layout_box(&taffy, node, styled);
        b.dimensions.content.x += root_layout.location.x;
        b.dimensions.content.y += root_layout.location.y;
        boxes.push(b);
    }

    boxes
}

fn build_taffy_tree(taffy: &mut Taffy, styled: &StyledNode) -> taffy::prelude::Node {
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
        if let Some(px) = parse_length(w, 800.0) {
            style.size.width = Dimension::Points(px);
        }
    }
    if let Some(h) = styled.properties.get("height") {
        if let Some(px) = parse_length(h, 600.0) {
            style.size.height = Dimension::Points(px);
        }
    }

    style.margin = taffy::geometry::Rect {
        left: to_length_auto(styled.properties.get("margin-left")),
        right: to_length_auto(styled.properties.get("margin-right")),
        top: to_length_auto(styled.properties.get("margin-top")),
        bottom: to_length_auto(styled.properties.get("margin-bottom")),
    };

    style.padding = taffy::geometry::Rect {
        left: to_length(styled.properties.get("padding-left").and_then(|v| parse_length(v, 800.0))),
        right: to_length(styled.properties.get("padding-right").and_then(|v| parse_length(v, 800.0))),
        top: to_length(styled.properties.get("padding-top").and_then(|v| parse_length(v, 600.0))),
        bottom: to_length(styled.properties.get("padding-bottom").and_then(|v| parse_length(v, 600.0))),
    };
    
    style.border = taffy::geometry::Rect {
        left: to_length(styled.properties.get("border-left-width").and_then(|v| parse_length(v, 800.0))),
        right: to_length(styled.properties.get("border-right-width").and_then(|v| parse_length(v, 800.0))),
        top: to_length(styled.properties.get("border-top-width").and_then(|v| parse_length(v, 600.0))),
        bottom: to_length(styled.properties.get("border-bottom-width").and_then(|v| parse_length(v, 600.0))),
    };

    let mut children = Vec::new();
    for child in &styled.children {
        if is_hidden(child) { continue; }
        children.push(build_taffy_tree(taffy, child));
    }

    if let Node::Text(text) = &styled.node {
        let text_clone = text.clone();
        taffy.new_leaf_with_measure(style, taffy::node::MeasureFunc::Boxed(Box::new(move |_known, available| {
            let font = get_font();
            let font_size = 16.0;
            let scale = PxScale { x: font_size, y: font_size };
            let v_metrics = font.as_scaled(scale).ascent() - font.as_scaled(scale).descent() + font.as_scaled(scale).line_gap();
            let line_height = v_metrics * 1.2;
            let space_width = font.as_scaled(scale).h_advance(font.glyph_id(' '));
            
            let max_w = available.width.unwrap_or(800.0);
            
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

fn build_layout_box<'a>(taffy: &Taffy, node: taffy::prelude::Node, styled: &'a StyledNode<'a>) -> LayoutBox<'a> {
    let layout = taffy.layout(node).unwrap();
    
    let padding_left = parse_length(styled.properties.get("padding-left").map(|s| s.as_str()).unwrap_or("0"), 800.0).unwrap_or(0.0);
    let padding_top = parse_length(styled.properties.get("padding-top").map(|s| s.as_str()).unwrap_or("0"), 600.0).unwrap_or(0.0);
    let padding_right = parse_length(styled.properties.get("padding-right").map(|s| s.as_str()).unwrap_or("0"), 800.0).unwrap_or(0.0);
    let padding_bottom = parse_length(styled.properties.get("padding-bottom").map(|s| s.as_str()).unwrap_or("0"), 600.0).unwrap_or(0.0);
    
    let border_left = parse_length(styled.properties.get("border-left-width").map(|s| s.as_str()).unwrap_or("0"), 800.0).unwrap_or(0.0);
    let border_top = parse_length(styled.properties.get("border-top-width").map(|s| s.as_str()).unwrap_or("0"), 600.0).unwrap_or(0.0);
    let border_right = parse_length(styled.properties.get("border-right-width").map(|s| s.as_str()).unwrap_or("0"), 800.0).unwrap_or(0.0);
    let border_bottom = parse_length(styled.properties.get("border-bottom-width").map(|s| s.as_str()).unwrap_or("0"), 600.0).unwrap_or(0.0);
    
    let mut dimensions = BoxDimensions::default();
    dimensions.content = Rect {
        x: layout.location.x + padding_left + border_left,
        y: layout.location.y + padding_top + border_top,
        width: (layout.size.width - padding_left - padding_right - border_left - border_right).max(0.0),
        height: (layout.size.height - padding_top - padding_bottom - border_top - border_bottom).max(0.0),
    };
    dimensions.padding = EdgeSizes {
        left: padding_left, right: padding_right, top: padding_top, bottom: padding_bottom
    };
    dimensions.border = EdgeSizes {
        left: border_left, right: border_right, top: border_top, bottom: border_bottom
    };
    
    let mut children = Vec::new();
    if let quint_html::Node::Element { .. } = styled.node {
        let taffy_children = taffy.children(node).unwrap();
        for (i, child) in styled.children.iter().filter(|c| !is_hidden(c)).enumerate() {
            if i < taffy_children.len() {
                let mut built_child = build_layout_box(taffy, taffy_children[i], child);
                built_child.dimensions.content.x += dimensions.content.x;
                built_child.dimensions.content.y += dimensions.content.y;
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
