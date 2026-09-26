pub mod color;
pub mod painter;

use quint_layout::LayoutBox;
use std::sync::Arc;
use vello::{Renderer, RendererOptions, Scene, util::RenderContext};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub fn render_window(layout_tree: Vec<LayoutBox>, initial_width: f32) {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Quint Browser (GPU Accelerated)")
            .with_inner_size(winit::dpi::LogicalSize::new(initial_width, 600.0))
            .build(&event_loop)
            .unwrap(),
    );

    let mut render_cx = RenderContext::new();
    let surface = pollster::block_on(render_cx.create_surface(
        window.clone(),
        window.inner_size().width,
        window.inner_size().height,
        vello::wgpu::PresentMode::AutoVsync,
    )).unwrap();

    let device_handle = &render_cx.devices[surface.dev_id];
    let blitter = vello::wgpu::util::TextureBlitter::new(&device_handle.device, surface.config.format);
    let mut renderer = Renderer::new(
        &device_handle.device,
        RendererOptions {
            use_cpu: false,
            antialiasing_support: vello::AaSupport::all(),
            num_init_threads: None,
            pipeline_cache: None,
        },
    )
    .unwrap();

    let mut surface = Some(surface);
    let mut intermediate: Option<(vello::wgpu::Texture, vello::wgpu::TextureView)> = None;

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    if size.width > 0 && size.height > 0 {
                        if let Some(surf) = &mut surface {
                            render_cx.resize_surface(surf, size.width, size.height);
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    let size = window.inner_size();
                    if size.width == 0 || size.height == 0 {
                        return;
                    }

                    if let Some(surf) = &surface {
                        let device_handle = &render_cx.devices[surf.dev_id];
                        
                        let surface_texture = match surf.surface.get_current_texture() {
                            vello::wgpu::CurrentSurfaceTexture::Success(tex) => tex,
                            vello::wgpu::CurrentSurfaceTexture::Suboptimal(tex) => tex,
                            _ => return,
                        };

                        let (target_texture, target_view) = intermediate
                            .take()
                            .filter(|(tex, _)| tex.width() == size.width && tex.height() == size.height)
                            .unwrap_or_else(|| {
                                let tex = device_handle.device.create_texture(&vello::wgpu::TextureDescriptor {
                                    label: Some("vello_intermediate_texture"),
                                    size: vello::wgpu::Extent3d {
                                        width: size.width,
                                        height: size.height,
                                        depth_or_array_layers: 1,
                                    },
                                    mip_level_count: 1,
                                    sample_count: 1,
                                    dimension: vello::wgpu::TextureDimension::D2,
                                    format: vello::wgpu::TextureFormat::Rgba8Unorm,
                                    usage: vello::wgpu::TextureUsages::STORAGE_BINDING
                                        | vello::wgpu::TextureUsages::TEXTURE_BINDING,
                                    view_formats: &[],
                                });
                                let view = tex.create_view(&vello::wgpu::TextureViewDescriptor::default());
                                (tex, view)
                            });

                        let mut scene = Scene::new();
                        painter::paint_tree(&layout_tree, &mut scene);

                        renderer
                            .render_to_texture(
                                &device_handle.device,
                                &device_handle.queue,
                                &scene,
                                &target_view,
                                &vello::RenderParams {
                                    base_color: vello::peniko::Color::WHITE,
                                    width: size.width,
                                    height: size.height,
                                    antialiasing_method: vello::AaConfig::Msaa16,
                                },
                            )
                            .expect("failed to render to intermediate texture");

                        let surface_view = surface_texture
                            .texture
                            .create_view(&vello::wgpu::TextureViewDescriptor::default());

                        let mut encoder = device_handle
                            .device
                            .create_command_encoder(&vello::wgpu::CommandEncoderDescriptor {
                                label: Some("blitter_encoder"),
                            });

                        blitter.copy(
                            &device_handle.device,
                            &mut encoder,
                            &target_view,
                            &surface_view,
                        );

                        device_handle.queue.submit([encoder.finish()]);
                        surface_texture.present();

                        intermediate = Some((target_texture, target_view));
                    }
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
