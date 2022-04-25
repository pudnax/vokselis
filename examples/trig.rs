use std::path::Path;
use vokselis::{
    run, shader_compiler::ShaderCompiler, CameraBinding, Context, Demo, PipelineHandle,
    ReloadablePipeline, Uniform,
};

use color_eyre::eyre::Result;
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

pub struct BasicPipeline {
    pub pipeline: wgpu::RenderPipeline,
    surface_format: wgpu::TextureFormat,
}

impl BasicPipeline {
    pub fn from_path(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        path: &Path,
        compiler: &mut ShaderCompiler,
    ) -> Self {
        let shader = unsafe {
            device.create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                label: path.to_str(),
                source: compiler.create_shader_module(path).unwrap().into(),
            })
        };
        Self::new_with_module(device, format, &shader)
    }

    pub fn new_with_module(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        shader: &wgpu::ShaderModule,
    ) -> Self {
        let global_bind_group_layout = device.create_bind_group_layout(&Uniform::DESC);
        let camera_bind_group_layout = device.create_bind_group_layout(&CameraBinding::DESC);
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Screen Pass Layout"),
            bind_group_layouts: &[&global_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render with Camera Pipeline"),
            layout: Some(&layout),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                targets: &[surface_format.into()],
            }),
            vertex: wgpu::VertexState {
                module: shader,
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

impl ReloadablePipeline for BasicPipeline {
    fn reload(&mut self, device: &wgpu::Device, module: &wgpu::ShaderModule) {
        *self = Self::new_with_module(device, self.surface_format, module);
    }
}

struct BasicTrig {
    pipeline: PipelineHandle<BasicPipeline>,
}

impl Demo for BasicTrig {
    fn init(ctx: &mut Context) -> Self {
        let path = Path::new("shaders/shader_with_camera.wgsl");
        let pipeline = BasicPipeline::from_path(
            &ctx.device,
            ctx.render_backbuffer.format(),
            path,
            &mut ctx.shader_compiler,
        );
        let pipeline = ctx.watcher.register(&path, pipeline).unwrap();
        Self { pipeline }
    }

    fn render(&mut self, ctx: &Context) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Trig Encoder"),
            });

        let pipeline = self.pipeline.borrow();

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Trig Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &ctx.render_backbuffer.texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&pipeline.pipeline);
            rpass.set_bind_group(0, &ctx.global_uniform_binding.binding, &[]);
            rpass.set_bind_group(1, &ctx.camera_binding.bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }

        ctx.queue.submit(Some(encoder.finish()));
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::with_user_event();
    let window = WindowBuilder::new()
        .with_title("Vokselis")
        .with_inner_size(LogicalSize::new(1280, 720))
        .build(&event_loop)?;

    run::<BasicTrig>(event_loop, window, None)
}
