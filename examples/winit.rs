use sdf::{
    rectangle::{Rectangle, RectangleRenderer},
    Device,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowBuilderExtWebSys;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::WindowBuilder,
};

fn main() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch="wasm32")] {
            console_error_panic_hook::set_once();
            tracing_wasm::set_as_global_default();
        } else {
            tracing_subscriber::fmt::init();
        }
    };

    let event_loop = EventLoop::new().unwrap();

    let (width, height) = (640, 480);
    let window_builder = WindowBuilder::new().with_inner_size(PhysicalSize::new(width, height));
    #[cfg(target_arch = "wasm32")]
    let window_builder = window_builder.with_canvas(
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| {
                let canvas = document.create_element("canvas").ok()?;

                document.body()?.append_child(&canvas).ok()?;

                canvas.dyn_into().ok()
            }),
    );
    let window = window_builder.build(&event_loop).unwrap();

    let rectangle = Rectangle {
        position: (320.0, 240.0).into(),
        half_dimensions: (160.0, 120.0).into(),
        corner_radii: (0.0, 0.0, 0.0, 0.0).into(),
        inner_color: (0.2, 0.4, 0.8, 1.0).into(),
        outer_color: (0.8, 0.4, 0.2, 1.0).into(),
        phase: 0.0,
        _padding: [0; 3],
    };

    let mut renderer = RectangleRenderer::new(
        futures::executor::block_on(Device::new(width, height, &window)).unwrap(),
        rectangle,
    );

    event_loop
        .run(
            |event, event_loop: &EventLoopWindowTarget<()>| match event {
                Event::WindowEvent {
                    ref event,
                    window_id: _,
                } => {
                    match event {
                        WindowEvent::CloseRequested => event_loop.exit(),
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(physical_size.width, physical_size.height);
                            window.request_redraw();
                        }
                        WindowEvent::RedrawRequested => match renderer.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                renderer.configure_surface()
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                            Err(wgpu::SurfaceError::Timeout) => tracing::warn!("Surface timeout"),
                        },
                        _ => {}
                    };
                }
                _ => {}
            },
        )
        .unwrap();
}
