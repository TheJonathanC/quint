mod layout;
mod types;

pub use layout::{is_auto, is_hidden, layout_block, lookup_px, parse_length, to_px};
pub use types::{BoxDimensions, EdgeSizes, LayoutBox, Rect};

use quint_style::StyledNode;

pub fn layout_tree<'a>(
    styled_nodes: &'a [StyledNode<'a>],
    containing_block_width: f32,
) -> Vec<LayoutBox<'a>> {
    let mut containing_block = BoxDimensions::default();
    containing_block.content.width = containing_block_width;

    let mut boxes = Vec::new();
    for styled in styled_nodes {
        if is_hidden(styled) {
            continue;
        }
        let b = layout_block(styled, &mut containing_block);
        containing_block.content.height += b.dimensions.margin_box().height;
        boxes.push(b);
    }
    boxes
}

#[cfg(test)]
mod tests {
    use super::*;
    use quint_html::Node;

    #[test]
    fn rect_default_is_zero() {
        let r = Rect::default();
        assert_eq!(
            r,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0
            }
        );
    }

    #[test]
    fn padding_box_expands_content() {
        let mut d = BoxDimensions::default();
        d.content = Rect {
            x: 10.0,
            y: 10.0,
            width: 100.0,
            height: 50.0,
        };
        d.padding = EdgeSizes {
            top: 5.0,
            right: 5.0,
            bottom: 5.0,
            left: 5.0,
        };
        let pb = d.padding_box();
        assert_eq!(pb.width, 110.0);
        assert_eq!(pb.height, 60.0);
        assert_eq!(pb.x, 5.0);
        assert_eq!(pb.y, 5.0);
    }

    #[test]
    fn margin_box_includes_all_edges() {
        let mut d = BoxDimensions::default();
        d.content = Rect {
            x: 20.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        };
        d.padding = EdgeSizes {
            top: 5.0,
            right: 5.0,
            bottom: 5.0,
            left: 5.0,
        };
        d.border = EdgeSizes {
            top: 1.0,
            right: 1.0,
            bottom: 1.0,
            left: 1.0,
        };
        d.margin = EdgeSizes {
            top: 10.0,
            right: 10.0,
            bottom: 10.0,
            left: 10.0,
        };
        let mb = d.margin_box();
        assert_eq!(mb.width, 132.0);
        assert_eq!(mb.height, 82.0);
    }

    fn styled_elem<'a>(
        node: &'a Node,
        props: &[(&str, &str)],
        children: Vec<StyledNode<'a>>,
    ) -> StyledNode<'a> {
        StyledNode {
            node,
            properties: props
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            children,
        }
    }

    #[test]
    fn auto_width_fills_container() {
        let node = Node::Element {
            tag: "div".into(),
            attributes: vec![],
            children: vec![],
        };
        let styled = styled_elem(&node, &[], vec![]);
        let arr = [styled];
        let boxes = layout_tree(&arr, 800.0);
        assert_eq!(boxes[0].dimensions.content.width, 800.0);
    }

    #[test]
    fn explicit_width() {
        let node = Node::Element {
            tag: "div".into(),
            attributes: vec![],
            children: vec![],
        };
        let styled = styled_elem(&node, &[("width", "200px")], vec![]);
        let arr = [styled];
        let boxes = layout_tree(&arr, 800.0);
        assert_eq!(boxes[0].dimensions.content.width, 200.0);
    }

    #[test]
    fn margin_auto_centers() {
        let node = Node::Element {
            tag: "div".into(),
            attributes: vec![],
            children: vec![],
        };
        let styled = styled_elem(
            &node,
            &[
                ("width", "200px"),
                ("margin-left", "auto"),
                ("margin-right", "auto"),
            ],
            vec![],
        );
        let arr = [styled];
        let boxes = layout_tree(&arr, 800.0);
        assert_eq!(boxes[0].dimensions.content.width, 200.0);
        assert_eq!(boxes[0].dimensions.margin.left, 300.0);
        assert_eq!(boxes[0].dimensions.margin.right, 300.0);
    }

    #[test]
    fn padding_reduces_content_width() {
        let node = Node::Element {
            tag: "div".into(),
            attributes: vec![],
            children: vec![],
        };
        let styled = styled_elem(
            &node,
            &[("padding-left", "20px"), ("padding-right", "20px")],
            vec![],
        );
        let arr = [styled];
        let boxes = layout_tree(&arr, 800.0);
        assert_eq!(boxes[0].dimensions.content.width, 760.0);
    }

    #[test]
    fn children_stack_vertically() {
        let child1_node = Node::Element {
            tag: "p".into(),
            attributes: vec![],
            children: vec![],
        };
        let child2_node = Node::Element {
            tag: "p".into(),
            attributes: vec![],
            children: vec![],
        };
        let parent_node = Node::Element {
            tag: "div".into(),
            attributes: vec![],
            children: vec![],
        };

        let c1 = styled_elem(&child1_node, &[("height", "50px")], vec![]);
        let c2 = styled_elem(&child2_node, &[("height", "30px")], vec![]);
        let parent = styled_elem(&parent_node, &[], vec![c1, c2]);

        let arr = [parent];
        let boxes = layout_tree(&arr, 800.0);
        let p = &boxes[0];
        assert_eq!(p.children[0].dimensions.content.y, 0.0);
        assert_eq!(p.children[1].dimensions.content.y, 50.0);
        assert_eq!(p.dimensions.content.height, 80.0);
    }

    #[test]
    fn explicit_height_overrides_content() {
        let child_node = Node::Element {
            tag: "p".into(),
            attributes: vec![],
            children: vec![],
        };
        let parent_node = Node::Element {
            tag: "div".into(),
            attributes: vec![],
            children: vec![],
        };

        let c = styled_elem(&child_node, &[("height", "50px")], vec![]);
        let parent = styled_elem(&parent_node, &[("height", "200px")], vec![c]);

        let arr = [parent];
        let boxes = layout_tree(&arr, 800.0);
        assert_eq!(boxes[0].dimensions.content.height, 200.0);
    }

    #[test]
    fn nested_padding_and_margin() {
        let child_node = Node::Element {
            tag: "p".into(),
            attributes: vec![],
            children: vec![],
        };
        let parent_node = Node::Element {
            tag: "div".into(),
            attributes: vec![],
            children: vec![],
        };

        let c = styled_elem(&child_node, &[("height", "100px")], vec![]);
        let parent = styled_elem(
            &parent_node,
            &[("padding-top", "10px"), ("padding-bottom", "10px")],
            vec![c],
        );

        let arr = [parent];
        let boxes = layout_tree(&arr, 800.0);
        let pb = boxes[0].dimensions.padding_box();
        assert_eq!(pb.height, 120.0);
    }

    #[test]
    fn text_node_calculates_height() {
        let text_node = Node::Text("Hello World".into());
        let styled_text = styled_elem(&text_node, &[], vec![]);
        let parent_node = Node::Element {
            tag: "p".into(),
            attributes: vec![],
            children: vec![],
        };
        let parent = styled_elem(&parent_node, &[], vec![styled_text]);
        let arr = [parent];
        let boxes = layout_tree(&arr, 800.0);
        assert_eq!(boxes[0].dimensions.content.height, 20.0);
    }

    #[test]
    fn margin_shorthand_centers_auto() {
        let node = Node::Element {
            tag: "div".into(),
            attributes: vec![],
            children: vec![],
        };
        let styled = styled_elem(
            &node,
            &[("width", "400px"), ("margin", "10px auto")],
            vec![],
        );
        let arr = [styled];
        let boxes = layout_tree(&arr, 800.0);
        assert_eq!(boxes[0].dimensions.content.width, 400.0);
        assert_eq!(boxes[0].dimensions.margin.left, 200.0);
        assert_eq!(boxes[0].dimensions.margin.right, 200.0);
        assert_eq!(boxes[0].dimensions.margin.top, 10.0);
    }

    #[test]
    fn hidden_tags_skipped() {
        let head_node = Node::Element {
            tag: "head".into(),
            attributes: vec![],
            children: vec![],
        };
        let styled_head = styled_elem(&head_node, &[], vec![]);
        let body_node = Node::Element {
            tag: "body".into(),
            attributes: vec![],
            children: vec![],
        };
        let styled_body = styled_elem(&body_node, &[], vec![]);
        let arr = [styled_head, styled_body];
        let boxes = layout_tree(&arr, 800.0);
        assert_eq!(boxes.len(), 1);
    }
}
