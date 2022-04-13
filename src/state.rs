use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::Instant,
};

use color_eyre::eyre::{eyre, Result};
use wgpu::Instance;
use winit::{dpi::PhysicalSize, window::Window};

mod screen_space;
use screen_space::ScreenSpacePipeline;

mod global_ubo;

use crate::{frame_counter::FrameCounter, input::Input, utils::RcWrap, watcher::Watcher};

use global_ubo::GlobalUniformBinding;
pub use global_ubo::Uniform;

pub struct State {
    watcher: Watcher,
    pub device: Arc<wgpu::Device>,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface_format: wgpu::TextureFormat,

    pub width: u32,
    pub height: u32,

    timeline: Instant,

    pipeline: Rc<RefCell<ScreenSpacePipeline>>,
    pipeline_sec: Rc<RefCell<ScreenSpacePipeline>>,

    pub global_uniform: Uniform,
    global_uniform_binding: GlobalUniformBinding,
}

impl State {
    pub async fn new(
        window: &Window,
        event_loop: &winit::event_loop::EventLoop<(PathBuf, wgpu::ShaderModule)>,
    ) -> Result<Self> {
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
        let device = Arc::new(device);

        let PhysicalSize { width, height } = window.inner_size();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &surface_config);

        let mut watcher = Watcher::new(device.clone(), event_loop)?;

        let sh1 = Path::new("shaders/shader.wgsl");
        let pipeline = ScreenSpacePipeline::from_path(&device, surface_format, sh1).wrap();
        watcher.register(&sh1, pipeline.clone())?;

        let sh2 = Path::new("shaders/shader_sec.wgsl");
        let pipeline_sec = ScreenSpacePipeline::from_path(&device, surface_format, sh2).wrap();
        watcher.register(&sh2, pipeline_sec.clone())?;

        let global_uniform = Uniform::default();
        let global_uniform_binding = GlobalUniformBinding::new(&device);

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            surface_format,

            width,
            height,

            timeline: Instant::now(),

            pipeline,
            pipeline_sec,
            watcher,

            global_uniform,
            global_uniform_binding,
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

        let pipeline = self.pipeline.borrow();
        let pipeline_sec = self.pipeline_sec.borrow();

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Present Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            pipeline.record(&mut rpass, &self.global_uniform_binding);
            pipeline_sec.record(&mut rpass, &self.global_uniform_binding);
        }

        self.queue.submit(Some(encoder.finish()));

        frame.present();

        Ok(())
    }

    pub fn register_shader_change(&mut self, path: PathBuf, shader: wgpu::ShaderModule) {
        if let Some(pipelines) = self.watcher.hash_dump.get_mut(&path) {
            for pipeline in pipelines.iter_mut() {
                let mut pipeline = pipeline.borrow_mut();
                pipeline.reload(&self.device, &shader);
            }
        }
    }

    pub fn update(&mut self, frame_counter: &FrameCounter, input: &Input) {
        self.global_uniform.time = self.timeline.elapsed().as_secs_f32();
        self.global_uniform.time_delta = frame_counter.time_delta();
        self.global_uniform.frame = frame_counter.frame_count;
        self.global_uniform.resolution = [self.width as _, self.height as _];
        input.process_position(&mut self.global_uniform);

        self.global_uniform_binding
            .update(&self.queue, &self.global_uniform);
    }
}
