use crate::utils::NonZeroSized;
use std::time::Duration;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

pub struct GlobalUniformBinding {
    pub binding: wgpu::BindGroup,
    buffer: wgpu::Buffer,
}

impl GlobalUniformBinding {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Uniform"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::bytes_of(&Uniform::default()),
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Global Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(Uniform::SIZE),
                },
                count: None,
            }],
        });
        let uniform = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Global Uniform Bind Group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Self {
            binding: uniform,
            buffer,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, uniform: &Uniform) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(uniform))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Uniform {
    pub pos: [f32; 3],
    pub frame: u32,
    pub resolution: [f32; 2],
    pub mouse: [f32; 2],
    pub mouse_pressed: u32,
    pub time: f32,
    pub time_delta: f32,
    _padding: f32,
    // pub record_period: f32,
    // _padding2: [f32; 3],
}

impl Default for Uniform {
    fn default() -> Self {
        Self {
            pos: [0.; 3],
            time: 0.,
            resolution: [1920.0, 780.],
            mouse: [0.; 2],
            mouse_pressed: false as _,
            frame: 0,
            time_delta: 1. / 60.,
            _padding: 0.,
            // record_period: 10.,
        }
    }
}

impl Uniform {
    pub const DESC: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
        label: Some("Global Uniform Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT.union(wgpu::ShaderStages::COMPUTE),
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(Uniform::SIZE),
            },
            count: None,
        }],
    };

    #[allow(dead_code)]
    pub fn new(
        pos: [f32; 3],
        resolution: [f32; 2],
        mouse: [f32; 2],
        mouse_pressed: u32,
        time: f32,
        time_delta: f32,
        frame: u32,
    ) -> Self {
        Self {
            pos,
            resolution,
            mouse,
            mouse_pressed,
            time,
            time_delta,
            frame,
            _padding: 0.,
        }
    }
}

impl std::fmt::Display for Uniform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let time = Duration::from_secs_f32(self.time);
        let time_delta = Duration::from_secs_f32(self.time_delta);
        write!(
            f,
            "position:\t{:?}\n\
              time:\t\t{:#.2?}\n\
              time delta:\t{:#.3?}, fps: {:#.2?}\n\
              width, height:\t{:?}\nmouse:\t\t{:.2?}\n\
              frame:\t\t{}\n",
            // record_period:\t{}\n",
            self.pos,
            time,
            time_delta,
            1. / self.time_delta,
            self.resolution,
            self.mouse,
            self.frame,
            // self.record_period
        )
    }
}
