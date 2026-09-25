pub mod color;
pub mod painter;

use quint_layout::LayoutBox;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub fn render_window(layout_tree: Vec<LayoutBox>, initial_width: f32) {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Quint Browser")
            .with_inner_size(winit::dpi::LogicalSize::new(initial_width, 600.0))
            .build(&event_loop)
            .unwrap(),
    );

    let context = Context::new(window.clone()).unwrap();
    let mut surface = Surface::new(&context, window.clone()).unwrap();

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    let size = window.inner_size();
                    if size.width == 0 || size.height == 0 {
                        return;
                    }

                    surface
                        .resize(
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        )
                        .unwrap();

                    let mut buffer = surface.buffer_mut().unwrap();
                    let mut pixmap = tiny_skia::Pixmap::new(size.width, size.height).unwrap();

                    painter::paint_tree(&layout_tree, &mut pixmap);

                    for (index, pixel) in pixmap.pixels().iter().enumerate() {
                        let r = pixel.red() as u32;
                        let g = pixel.green() as u32;
                        let b = pixel.blue() as u32;
                        buffer[index] = (r << 16) | (g << 8) | b;
                    }

                    buffer.present().unwrap();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    elwt.exit();
                }
                _ => {}
            }
        })
        .unwrap();
}
