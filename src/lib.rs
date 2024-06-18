use cgmath::Matrix4;

pub mod rectangle;

pub struct Device<'window> {
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl<'window> Device<'window> {
    pub async fn new(
        width: u32,
        height: u32,
        suface_target: impl Into<wgpu::SurfaceTarget<'window>>,
    ) -> Result<Device<'window>, DeviceError> {
        assert!(
            width != 0 && height != 0,
            "Dimensions cannot be zero, found: ({}, {})",
            width,
            height
        );

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(suface_target).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .ok_or(DeviceError::NoCompatibleAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("device"),
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    ..Default::default()
                },
                None,
            )
            .await?;

        let formats = surface.get_capabilities(&adapter).formats;
        let format = *formats
            .iter()
            .find(|format| format.is_srgb())
            .unwrap_or(formats.first().unwrap());

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 3,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: Vec::new(),
        };

        surface.configure(&device, &surface_config);

        Ok(Self {
            surface_config,
            surface,
            device,
            queue,
        })
    }

    pub fn view_projection(&self) -> Matrix4<f32> {
        let x_scalar = 1.0 / (self.surface_config.width as f32 / 2.0);
        let y_scalar = -1.0 / (self.surface_config.height as f32 / 2.0);
        let x_offset = -1.0;
        let y_offset = 1.0;

        Matrix4::from([
            [x_scalar, 0.0, 0.0, 0.0],
            [0.0, y_scalar, 0.0, 0.0],
            [x_offset, y_offset, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    #[error("no compatible adapter")]
    NoCompatibleAdapter,
    #[error("failed to request device: {0:?}")]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),
}
