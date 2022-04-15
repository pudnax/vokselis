use std::{
    cell::RefCell,
    fmt::Display,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::Instant,
};

use color_eyre::eyre::{eyre, Result};
use wgpu::Instance;
use winit::{dpi::PhysicalSize, window::Window};

mod basic;
mod global_ubo;
mod hdr_backbuffer;
mod present;

use basic::BasicPipeline;
use hdr_backbuffer::HdrBackBuffer;
use present::PresentPipeline;

use crate::{
    utils::frame_counter::FrameCounter,
    utils::input::Input,
    utils::RcWrap,
    watcher::{ReloadablePipeline, Watcher},
};

use global_ubo::GlobalUniformBinding;
pub use global_ubo::Uniform;

pub struct State {
    watcher: Watcher,
    adapter: wgpu::Adapter,
    pub device: Arc<wgpu::Device>,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface_format: wgpu::TextureFormat,

    render_backbuffer: HdrBackBuffer,
    multisampled_framebuffer: wgpu::TextureView,

    rgb_texture: wgpu::Texture,

    pub width: u32,
    pub height: u32,

    timeline: Instant,

    pipeline: Rc<RefCell<BasicPipeline>>,
    pipeline_sec: Rc<RefCell<BasicPipeline>>,
    present_pipeline: Rc<RefCell<PresentPipeline>>,

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
        let surface_format = wgpu::TextureFormat::Bgra8Unorm;

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
        let pipeline = BasicPipeline::from_path(&device, HdrBackBuffer::FORMAT, sh1).wrap();
        watcher.register(&sh1, pipeline.clone())?;

        let sh2 = Path::new("shaders/shader_sec.wgsl");
        let pipeline_sec = BasicPipeline::from_path(&device, HdrBackBuffer::FORMAT, sh2).wrap();
        watcher.register(&sh2, pipeline_sec.clone())?;

        let present_shader = Path::new("shaders/present.wgsl");
        let present_pipeline =
            PresentPipeline::from_path(&device, surface_format, present_shader).wrap();
        watcher.register(&present_shader, present_pipeline.clone())?;

        let multisampled_framebuffer = create_multisampled_framebuffer(
            &device,
            HdrBackBuffer::FORMAT,
            (surface_config.width, surface_config.height),
        );

        let global_uniform = Uniform::default();
        let global_uniform_binding = GlobalUniformBinding::new(&device);

        let render_backbuffer = HdrBackBuffer::new(&device, (width, height));
        let rgb_texture = create_rgb_framebuffer(&device, &surface_config);

        Ok(Self {
            adapter,
            device,
            queue,
            surface,
            surface_config,
            surface_format,

            multisampled_framebuffer,

            rgb_texture,

            render_backbuffer,

            width,
            height,

            timeline: Instant::now(),

            pipeline,
            pipeline_sec,
            watcher,

            present_pipeline,

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

        self.multisampled_framebuffer = create_multisampled_framebuffer(
            &self.device,
            HdrBackBuffer::FORMAT,
            (self.surface_config.width, self.surface_config.height),
        );
        self.rgb_texture = create_rgb_framebuffer(&self.device, &self.surface_config);
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let frame_view = frame.texture.create_view(&Default::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Present Encoder"),
            });

        let pipeline = self.pipeline.borrow();
        let pipeline_sec = self.pipeline_sec.borrow();

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Drawing Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &self.multisampled_framebuffer,
                resolve_target: Some(&self.render_backbuffer.texture_view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        pipeline.record(&mut rpass, &self.global_uniform_binding);
        pipeline_sec.record(&mut rpass, &self.global_uniform_binding);
        drop(rpass);

        let present_pipeline = self.present_pipeline.borrow();
        let rgb = self.rgb_texture.create_view(&Default::default());
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Present Pass"),
            color_attachments: &[
                wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                },
                wgpu::RenderPassColorAttachment {
                    view: &rgb,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                },
            ],
            depth_stencil_attachment: None,
        });

        present_pipeline.record(
            &mut rpass,
            &self.global_uniform_binding,
            &self.render_backbuffer.render_bind_group,
        );
        drop(rpass);

        self.queue.submit(Some(encoder.finish()));

        frame.present();

        Ok(())
    }

    pub fn register_shader_change(&mut self, path: PathBuf, shader: wgpu::ShaderModule) {
        if let Some(pipelines) = self.watcher.hash_dump.get_mut(&path) {
            for pipeline in pipelines.iter_mut() {
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

    pub fn get_info(&self) -> RendererInfo {
        let info = self.adapter.get_info();
        RendererInfo {
            device_name: info.name,
            device_type: self.get_device_type().to_string(),
            vendor_name: self.get_vendor_name().to_string(),
            backend: self.get_backend().to_string(),
            screen_format: self.surface_config.format,
        }
    }
    fn get_vendor_name(&self) -> &str {
        match self.adapter.get_info().vendor {
            0x1002 => "AMD",
            0x1010 => "ImgTec",
            0x10DE => "NVIDIA Corporation",
            0x13B5 => "ARM",
            0x5143 => "Qualcomm",
            0x8086 => "INTEL Corporation",
            _ => "Unknown vendor",
        }
    }
    fn get_backend(&self) -> &str {
        match self.adapter.get_info().backend {
            wgpu::Backend::Empty => "Empty",
            wgpu::Backend::Vulkan => "Vulkan",
            wgpu::Backend::Metal => "Metal",
            wgpu::Backend::Dx12 => "Dx12",
            wgpu::Backend::Dx11 => "Dx11",
            wgpu::Backend::Gl => "GL",
            wgpu::Backend::BrowserWebGpu => "Browser WGPU",
        }
    }
    fn get_device_type(&self) -> &str {
        match self.adapter.get_info().device_type {
            wgpu::DeviceType::Other => "Other",
            wgpu::DeviceType::IntegratedGpu => "Integrated GPU",
            wgpu::DeviceType::DiscreteGpu => "Discrete GPU",
            wgpu::DeviceType::VirtualGpu => "Virtual GPU",
            wgpu::DeviceType::Cpu => "CPU",
        }
    }
}

#[derive(Debug)]
pub struct RendererInfo {
    pub device_name: String,
    pub device_type: String,
    pub vendor_name: String,
    pub backend: String,
    pub screen_format: wgpu::TextureFormat,
}

impl Display for RendererInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Vendor name: {}", self.vendor_name)?;
        writeln!(f, "Device name: {}", self.device_name)?;
        writeln!(f, "Device type: {}", self.device_type)?;
        writeln!(f, "Backend: {}", self.backend)?;
        write!(f, "Screen format: {:?}", self.screen_format)?;
        Ok(())
    }
}

fn create_multisampled_framebuffer(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    (width, height): (u32, u32),
) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("Mult 16"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        })
        .create_view(&Default::default())
}

fn create_rgb_framebuffer(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };
    let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
        label: Some("RGB Texture"),
        format: wgpu::TextureFormat::Rgba8Unorm,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    };

    device.create_texture(multisampled_frame_descriptor)
}
