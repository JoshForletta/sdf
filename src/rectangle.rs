use cgmath::{Matrix4, Vector2, Vector4};
use wgpu::util::DeviceExt;

use crate::Device;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    pub position: Vector2<f32>,
    pub half_dimensions: Vector2<f32>,
    pub corner_radii: Vector4<f32>,
    pub outer_color: Vector4<f32>,
    pub inner_color: Vector4<f32>,
    pub phase: f32,
    pub _padding: [u32; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct Globals {
    view_projection: Matrix4<f32>,
    rectangle: Rectangle,
}

unsafe impl bytemuck::Zeroable for Globals {}
unsafe impl bytemuck::Pod for Globals {}

pub struct RectangleRenderer<'window> {
    device: Device<'window>,
    globals: Globals,
    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
    dirty: bool,
    render_pipeline: wgpu::RenderPipeline,
}

impl<'window> RectangleRenderer<'window> {
    pub fn new(device: Device<'window>, rectangle: Rectangle) -> Self {
        let globals = Globals {
            view_projection: device.view_projection(),
            rectangle,
        };

        let globals_buffer = device
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("global_buffer"),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&[globals]),
            });

        let globals_bind_group_layout =
            device
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("buffer_bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let globals_bind_group = device.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("globals_buffer_bind_group"),
            layout: &globals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        let shader_module = device
            .device
            .create_shader_module(wgpu::include_wgsl!("shaders/rectangle.wgsl"));

        let render_pipeline_layout =
            device
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("rectangle_render_pipeline_layout"),
                    bind_group_layouts: &[&globals_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            device
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("RectangleRenderer"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: "vertex_main",
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: "fragment_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: device.surface_config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    multiview: None,
                });

        Self {
            device,
            globals,
            globals_buffer,
            globals_bind_group,
            dirty: false,
            render_pipeline,
        }
    }

    pub fn set_phase(&mut self, phase: f32) {
        self.globals.rectangle.phase = phase;

        self.dirty = true;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.device.surface_config.width = width;
        self.device.surface_config.height = height;

        self.globals.view_projection = self.device.view_projection();

        self.configure_surface();

        self.dirty = true;
    }

    pub fn configure_surface(&self) {
        self.device
            .surface
            .configure(&self.device.device, &self.device.surface_config);
    }

    fn write_globals(&mut self) {
        self.device.queue.write_buffer(
            &self.globals_buffer,
            0,
            bytemuck::cast_slice(&[self.globals]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.dirty.then(|| self.write_globals());

        let mut encoder = self
            .device
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let suface_texture = self.device.surface.get_current_texture()?;
        let surface_texture_view = suface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.globals_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        self.device.queue.submit(std::iter::once(encoder.finish()));
        suface_texture.present();

        Ok(())
    }
}
