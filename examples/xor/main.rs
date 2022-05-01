use std::path::Path;

use bytemuck::{Pod, Zeroable};
use color_eyre::eyre::Result;
use vokselis::{dispatch_optimal, run, Camera, Demo, HdrBackBuffer, PipelineHandle};
use wgpu::util::DeviceExt;
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

mod raycast;
mod xor_compute;

const TILE_SIZE: u32 = 256;

#[derive(Debug)]
enum Mode {
    SinglePass,
    Tile,
}

#[repr(C)]
#[derive(Debug, Pod, Zeroable, Clone, Copy)]
pub struct Offset {
    x: f32,
    y: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct TimestampData {
    start: u64,
    end: u64,
}

struct Xor {
    xor_texture: PipelineHandle<xor_compute::XorCompute>,
    raycast_single: PipelineHandle<raycast::RaycastPipeline>,
    raycast_tile: PipelineHandle<raycast::RaycastPipeline>,
    mode: Mode,

    offset_buffer_bind_group: wgpu::BindGroup,
    buffer_len: usize,
    aligned_offset: u32,

    timestamp: wgpu::QuerySet,
    timestamp_period: f32,
    timestamp_buffer: wgpu::Buffer,
}

impl Demo for Xor {
    fn init(ctx: &mut vokselis::Context) -> Self {
        let path = Path::new("shaders/raycast_compute.wgsl");
        let raycast_single = ctx.watcher.register(
            &path,
            raycast::RaycastPipeline::from_path(
                &ctx.device,
                &path,
                &mut ctx.shader_compiler,
                "single",
            ),
        );
        let path = Path::new("shaders/raycast_compute.wgsl");
        let raycast_tile = ctx.watcher.register(
            &path,
            raycast::RaycastPipeline::from_path(
                &ctx.device,
                &path,
                &mut ctx.shader_compiler,
                "tile",
            ),
        );
        let path = Path::new("shaders/xor.wgsl");
        let xor_texture = ctx.watcher.register(
            &path,
            xor_compute::XorCompute::from_path(&ctx.device, &path, &mut ctx.shader_compiler),
        );

        let (w, h) = HdrBackBuffer::DEFAULT_RESOLUTION;
        let min_align = ctx.limits.min_storage_buffer_offset_alignment;
        let padding = (min_align - std::mem::size_of::<Offset>() as u32 % min_align) % min_align;
        let offsets = {
            let mut res = vec![];
            for y in 0..((h / TILE_SIZE) + 1) {
                for x in 0..((w / TILE_SIZE) + 1) {
                    res.extend(bytemuck::bytes_of(&Offset {
                        x: (x * TILE_SIZE) as f32,
                        y: (y * TILE_SIZE) as f32,
                    }));
                    res.extend(std::iter::repeat(0).take(padding as _));
                }
            }

            res
        };
        let aligned_offset = std::mem::size_of::<Offset>() as u32 + padding;
        let buffer_len = offsets.len() / aligned_offset as usize;

        let offset_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Offsets Buffer"),
                contents: bytemuck::cast_slice(&offsets),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let offset_buffer_bind_group_layout = ctx
            .device
            .create_bind_group_layout(&raycast::RaycastPipeline::OFFSET_BUFFER_DESC);
        let offset_buffer_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Offset Buffer Bind Group"),
            layout: &offset_buffer_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &offset_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<Offset>() as _),
                }),
            }],
        });

        let timestamp = ctx.device.create_query_set(&wgpu::QuerySetDescriptor {
            label: None,
            count: 2,
            ty: wgpu::QueryType::Timestamp,
        });
        let timestamp_period = ctx.queue.get_timestamp_period();
        let timestamp_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Query Buffer"),
            size: std::mem::size_of::<TimestampData>() as _,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        println!("Change rendering mode on F1");

        Self {
            xor_texture,
            raycast_single,
            raycast_tile,
            mode: Mode::SinglePass,

            aligned_offset,
            offset_buffer_bind_group,
            buffer_len,

            timestamp,
            timestamp_period,
            timestamp_buffer,
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

        if ctx.global_uniform.frame % 100 == 0 {
            let _ = self
                .timestamp_buffer
                .slice(..)
                .map_async(wgpu::MapMode::Read);
            {
                ctx.device.poll(wgpu::Maintain::Wait);
                let timestamp_view = self
                    .timestamp_buffer
                    .slice(..std::mem::size_of::<TimestampData>() as wgpu::BufferAddress)
                    .get_mapped_range();
                let timestamp_data: &TimestampData = bytemuck::from_bytes(&*timestamp_view);
                let nanoseconds =
                    (timestamp_data.end - timestamp_data.start) as f32 * self.timestamp_period;
                let time_period = std::time::Duration::from_nanos(nanoseconds as _);
                eprintln!(
                    "Time on raycast shader: {:?} ({:?})",
                    time_period, self.mode
                );
            }
            self.timestamp_buffer.unmap();
        }
    }

    fn update_input(&mut self, event: winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::KeyboardInput {
                input:
                    winit::event::KeyboardInput {
                        state: winit::event::ElementState::Pressed,
                        virtual_keycode: Some(winit::event::VirtualKeyCode::F1),
                        ..
                    },
                ..
            } => {
                self.mode = match self.mode {
                    Mode::SinglePass => Mode::Tile,
                    Mode::Tile => Mode::SinglePass,
                };
                println!("Switched to: {:?}", self.mode);
            }
            _ => {}
        }
    }

    fn render(&mut self, ctx: &vokselis::Context) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Volume Encoder"),
            });

        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Raycast Pass"),
        });

        cpass.write_timestamp(&self.timestamp, 0);

        match self.mode {
            Mode::SinglePass => {
                cpass.set_pipeline(&self.raycast_single.pipeline);

                cpass.set_bind_group(0, &ctx.global_uniform_binding.binding, &[]);
                cpass.set_bind_group(1, &ctx.camera_binding.bind_group, &[]);
                cpass.set_bind_group(2, &self.xor_texture.storage_bind_group, &[]);
                cpass.set_bind_group(3, &ctx.render_backbuffer.storage_bind_group, &[]);
                cpass.set_bind_group(4, &self.offset_buffer_bind_group, &[0]);
                let (width, height) = HdrBackBuffer::DEFAULT_RESOLUTION;
                cpass.dispatch(dispatch_optimal(width, 8), dispatch_optimal(height, 8), 1);
            }
            Mode::Tile => {
                cpass.set_pipeline(&self.raycast_tile.pipeline);

                cpass.set_bind_group(0, &ctx.global_uniform_binding.binding, &[]);
                cpass.set_bind_group(1, &ctx.camera_binding.bind_group, &[]);
                cpass.set_bind_group(2, &self.xor_texture.storage_bind_group, &[]);
                cpass.set_bind_group(3, &ctx.render_backbuffer.storage_bind_group, &[]);
                for offset in 0..self.buffer_len {
                    cpass.set_bind_group(
                        4,
                        &self.offset_buffer_bind_group,
                        &[offset as u32 * self.aligned_offset],
                    );
                    cpass.dispatch(
                        dispatch_optimal(TILE_SIZE, 16),
                        dispatch_optimal(TILE_SIZE, 16),
                        1,
                    );
                }
            }
        }
        cpass.write_timestamp(&self.timestamp, 1);
        drop(cpass);
        encoder.resolve_query_set(&self.timestamp, 0..2, &self.timestamp_buffer, 0);

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
        3.,
        -0.5,
        1.,
        (0., 0., 0.).into(),
        window_size.width as f32 / window_size.height as f32,
    );
    run::<Xor>(event_loop, window, Some(camera))
}
