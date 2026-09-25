use crate::types::{BoxDimensions, LayoutBox};
use quint_style::StyledNode;

pub fn to_px(value: &str) -> f32 {
    let trimmed = value.trim();
    if trimmed.ends_with("px") {
        trimmed[..trimmed.len() - 2].parse().unwrap_or(0.0)
    } else {
        trimmed.parse().unwrap_or(0.0)
    }
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

    let margin_left = lookup_px(styled, "margin-left");
    let margin_right = lookup_px(styled, "margin-right");
    let border_left = lookup_px(styled, "border-left-width");
    let border_right = lookup_px(styled, "border-right-width");
    let padding_left = lookup_px(styled, "padding-left");
    let padding_right = lookup_px(styled, "padding-right");

    let total =
        margin_left + border_left + padding_left + border_right + padding_right + margin_right;

    let (content_width, final_margin_left, final_margin_right) =
        if width.is_none() || width == Some("auto") {
            let auto_width = (container_width - total).max(0.0);
            (auto_width, margin_left, margin_right)
        } else {
            let w = to_px(width.unwrap());
            let remaining = container_width - total - w;

            let ml_auto = is_auto(styled, "margin-left");
            let mr_auto = is_auto(styled, "margin-right");

            match (ml_auto, mr_auto) {
                (true, true) => {
                    let m = (remaining / 2.0).max(0.0);
                    (w, m, m)
                }
                (true, false) => (w, remaining.max(0.0), margin_right),
                (false, true) => (w, margin_left, remaining.max(0.0)),
                (false, false) => (w, margin_left, margin_right + remaining),
            }
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

    d.margin.top = lookup_px(styled, "margin-top");
    d.margin.bottom = lookup_px(styled, "margin-bottom");
    d.border.top = lookup_px(styled, "border-top-width");
    d.border.bottom = lookup_px(styled, "border-bottom-width");
    d.padding.top = lookup_px(styled, "padding-top");
    d.padding.bottom = lookup_px(styled, "padding-bottom");

    d.content.x = containing_block.content.x + d.margin.left + d.border.left + d.padding.left;
    d.content.y = containing_block.content.y
        + containing_block.content.height
        + d.margin.top
        + d.border.top
        + d.padding.top;
}

fn layout_children(layout_box: &mut LayoutBox) {
    for child_styled in &layout_box.styled_node.children {
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
            }
        }
    }
}
