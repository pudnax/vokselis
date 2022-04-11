use color_eyre::eyre::{eyre, Result};
use wgpu::Instance;
use winit::{dpi::PhysicalSize, window::Window};

pub struct State {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface_format: wgpu::TextureFormat,

    pub width: u32,
    pub height: u32,

    pipeline: wgpu::RenderPipeline,
    pipeline_sec: wgpu::RenderPipeline,
}

impl State {
    pub async fn new(window: &Window) -> Result<Self> {
        let instance = Instance::new(wgpu::Backends::PRIMARY);

        let surface = unsafe { instance.create_surface(&window) };

        let adapter: wgpu::Adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .ok_or(eyre!("Failed to create device adapter."))?;

        let features = adapter.features();
        let limits = adapter.limits();
        let surface_format = surface
            .get_preferred_format(&adapter)
            .unwrap_or(wgpu::TextureFormat::Bgra8Unorm);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device Descriptor"),
                    features,
                    limits,
                },
                None,
            )
            .await?;

        let PhysicalSize { width, height } = window.inner_size();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &surface_config);

        let shader = device.create_shader_module(&wgpu::include_wgsl!("./shader.wgsl"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: None,
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[surface_format.into()],
            }),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let shader_sec = device.create_shader_module(&wgpu::include_wgsl!("./shader_sec.wgsl"));

        let pipeline_sec = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: None,
            fragment: Some(wgpu::FragmentState {
                module: &shader_sec,
                entry_point: "fs_main",
                targets: &[surface_format.into()],
            }),
            vertex: wgpu::VertexState {
                module: &shader_sec,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            surface_format,

            width,
            height,

            pipeline,
            pipeline_sec,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.surface_config.height = height;
        self.surface_config.width = width;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.3,
                        g: 0.2,
                        b: 0.2,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        rpass.set_pipeline(&self.pipeline);
        rpass.draw(0..3, 0..1);
        rpass.set_pipeline(&self.pipeline_sec);
        rpass.draw(0..3, 0..1);
        drop(rpass);

        self.queue.submit(Some(encoder.finish()));

        frame.present();

        Ok(())
    }
}
