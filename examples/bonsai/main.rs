use std::path::Path;

use vokselis::context::raycast::RaycastPipeline;
use vokselis::{run, Camera, Demo, PipelineHandle, VolumeTexture};

use color_eyre::eyre::Result;
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

struct Bonsai {
    volume_texture: VolumeTexture,
    pipeline: PipelineHandle<RaycastPipeline>,
}

impl Demo for Bonsai {
    fn init(ctx: &mut vokselis::Context) -> Self {
        let volume_texture = VolumeTexture::new(&ctx.device, &ctx.queue);
        let path = Path::new("shaders/raycast.wgsl");
        let pipeline = RaycastPipeline::from_path(&ctx.device, path, &mut ctx.shader_compiler);
        let pipeline = ctx.watcher.register(&path, pipeline).unwrap();
        Self {
            volume_texture,
            pipeline,
        }
    }

    fn render(&mut self, ctx: &vokselis::Context) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Volume Encoder"),
            });

        let pipeline = self.pipeline.borrow();
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Volume Pass"),
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

            pipeline.record(
                &mut rpass,
                &ctx.global_uniform_binding,
                &ctx.camera_binding,
                &self.volume_texture,
            );
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
    let window_size = window.inner_size();

    let camera = Camera::new(
        1.,
        0.5,
        1.,
        (0.5, 0.5, 0.5).into(),
        window_size.width as f32 / window_size.height as f32,
    );
    run::<Bonsai>(event_loop, window, Some(camera))
}
