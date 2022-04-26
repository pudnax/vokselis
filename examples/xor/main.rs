use std::path::Path;

use color_eyre::eyre::Result;
use vokselis::{context::raycast::RaycastPipeline, run, Camera, Demo, PipelineHandle};
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

mod xor_compute;

struct Xor {
    xor_texture: PipelineHandle<xor_compute::XorCompute>,
    raycast: PipelineHandle<RaycastPipeline>,
}

impl Demo for Xor {
    fn init(ctx: &mut vokselis::Context) -> Self {
        let path = Path::new("shaders/raycast.wgsl");
        let raycast = ctx.watcher.register(
            &path,
            RaycastPipeline::from_path(&ctx.device, &path, &mut ctx.shader_compiler),
        );
        let path = Path::new("shaders/xor.wgsl");
        let xor_texture = ctx.watcher.register(
            &path,
            xor_compute::XorCompute::from_path(&ctx.device, &path, &mut ctx.shader_compiler),
        );
        Self {
            xor_texture,
            raycast,
        }
    }

    fn update(&mut self, ctx: &mut vokselis::Context) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("XOR Update encoder"),
            });

        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("XOR Update Pass"),
        });
        self.xor_texture
            .record(&mut cpass, &ctx.global_uniform_binding);
        drop(cpass);
        ctx.queue.submit(Some(encoder.finish()));
    }

    fn render(&mut self, ctx: &vokselis::Context) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Volume Encoder"),
            });

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

            self.raycast.record(
                &mut rpass,
                &ctx.global_uniform_binding,
                &ctx.camera_binding,
                &self.xor_texture.render_bind_group,
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
    run::<Xor>(event_loop, window, Some(camera))
}
