use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::Duration,
};

use color_eyre::eyre::{eyre, Result};
use notify::{event::ModifyKind, Config, EventKind, Watcher};
use wgpu::Instance;
use winit::{dpi::PhysicalSize, window::Window};

use crate::shader_compiler::ShaderCompiler;

pub trait ReloadablePipeline {
    fn reload(&mut self, device: &wgpu::Device, module: wgpu::ShaderModule);
}

struct ScreenSpacePipeline {
    pipeline: wgpu::RenderPipeline,
    surface_format: wgpu::TextureFormat,
}

impl ScreenSpacePipeline {
    #![allow(dead_code)]
    fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(&wgpu::include_wgsl!("./shader.wgsl"));
        Self::new_with_module(device, surface_format, shader)
    }

    fn new_with_module(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        shader: wgpu::ShaderModule,
    ) -> Self {
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
        Self {
            pipeline,
            surface_format,
        }
    }
}

impl ReloadablePipeline for ScreenSpacePipeline {
    fn reload(&mut self, device: &wgpu::Device, module: wgpu::ShaderModule) {
        *self = Self::new_with_module(device, self.surface_format, module);
    }
}

pub struct State {
    _watcher: notify::PollWatcher,
    pub device: Arc<wgpu::Device>,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface_format: wgpu::TextureFormat,

    pub width: u32,
    pub height: u32,

    pipeline: Rc<RefCell<ScreenSpacePipeline>>,
    pipeline_sec: Rc<RefCell<ScreenSpacePipeline>>,
    pub hash_dump: HashMap<PathBuf, Rc<RefCell<dyn ReloadablePipeline>>>,
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

        let shader = device.create_shader_module(&wgpu::include_wgsl!("./shader.wgsl"));
        let pipeline = ScreenSpacePipeline::new_with_module(&device, surface_format, shader);
        let pipeline = Rc::new(RefCell::new(pipeline));

        let shader = device.create_shader_module(&wgpu::include_wgsl!("./shader_sec.wgsl"));
        let pipeline_sec = ScreenSpacePipeline::new_with_module(&device, surface_format, shader);
        let pipeline_sec = Rc::new(RefCell::new(pipeline_sec));

        let proxy = event_loop.create_proxy();
        let mut watcher = notify::PollWatcher::with_delay(
            {
                let device = Arc::downgrade(&device);
                let mut shader_compiler = ShaderCompiler::new();
                move |event| match event {
                    Ok(res) => {
                        if let notify::event::Event {
                            kind: EventKind::Modify(ModifyKind::Metadata(..) | ModifyKind::Data(..)),
                            paths,
                            ..
                        } = res
                        {
                            for path in paths {
                                let path = path.canonicalize().unwrap();
                                if let Ok(x) = shader_compiler.create_shader_module(&path) {
                                    let device_ref = device.upgrade().unwrap();
                                    let module = device_ref.create_shader_module(
                                        &wgpu::ShaderModuleDescriptor {
                                            label: None,
                                            source: wgpu::ShaderSource::SpirV(x.into()),
                                        },
                                    );
                                    proxy.send_event((path, module)).unwrap();
                                };
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("File watcher error: {err}");
                    }
                }
            },
            Duration::from_millis(3),
        )?;
        watcher.configure(Config::PreciseEvents(true))?;
        let sh1 = Path::new("src/shader.wgsl").canonicalize().unwrap();
        watcher.watch(&sh1, notify::RecursiveMode::NonRecursive)?;
        let sh2 = Path::new("src/shader_sec.wgsl").canonicalize().unwrap();
        watcher.watch(&sh2, notify::RecursiveMode::NonRecursive)?;

        let mut hash_dump: HashMap<PathBuf, Rc<RefCell<dyn ReloadablePipeline>>> = HashMap::new();
        hash_dump.insert(sh1, pipeline.clone());
        hash_dump.insert(sh2, pipeline_sec.clone());

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
            hash_dump,
            _watcher: watcher,
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

        let pipeline = &self.pipeline.borrow().pipeline;
        let pipeline_sec = &self.pipeline_sec.borrow().pipeline;

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

        rpass.set_pipeline(pipeline);
        rpass.draw(0..3, 0..1);
        rpass.set_pipeline(pipeline_sec);
        rpass.draw(0..3, 0..1);
        drop(rpass);

        self.queue.submit(Some(encoder.finish()));

        frame.present();

        Ok(())
    }
}
