use crate::utils::NonZeroSized;
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_position: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}

pub struct CameraBinding {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl CameraBinding {
    pub const DESC: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
        label: Some("Camera Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT.union(wgpu::ShaderStages::COMPUTE),
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(CameraUniform::SIZE),
            },
            count: None,
        }],
    };

    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::bytes_of(&CameraUniform::default()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let layout = device.create_bind_group_layout(&Self::DESC);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self { buffer, bind_group }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, camera: &mut Camera) {
        if camera.updated {
            queue.write_buffer(
                &self.buffer,
                0,
                bytemuck::bytes_of(&camera.get_view_proj_matrix()),
            );
            camera.updated = false;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub zoom: f32,
    pub target: Vec3,
    pub eye: Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub up: Vec3,
    pub aspect: f32,

    updated: bool,
}

impl Camera {
    const ZFAR: f32 = 100.;
    const ZNEAR: f32 = 0.1;
    const FOVY: f32 = std::f32::consts::PI / 2.0;
    const UP: Vec3 = Vec3::Y;

    pub fn new(zoom: f32, pitch: f32, yaw: f32, target: Vec3, aspect: f32) -> Self {
        let mut camera = Self {
            zoom,
            pitch,
            yaw,
            eye: Vec3::ZERO,
            target,
            up: Self::UP,
            aspect,

            updated: false,
        };
        camera.fix_eye();
        camera
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh(Self::FOVY, self.aspect, Self::ZNEAR, Self::ZFAR);
        proj * view
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.3, Self::ZFAR / 2.);
        self.fix_eye();
        self.updated = true;
    }

    pub fn add_zoom(&mut self, delta: f32) {
        self.set_zoom(self.zoom + delta);
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch.clamp(
            -std::f32::consts::PI / 2.0 + f32::EPSILON,
            std::f32::consts::PI / 2.0 - f32::EPSILON,
        );
        self.fix_eye();
        self.updated = true;
    }

    pub fn add_pitch(&mut self, delta: f32) {
        self.set_pitch(self.pitch + delta);
    }

    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
        self.fix_eye();
        self.updated = true;
    }

    pub fn add_yaw(&mut self, delta: f32) {
        self.set_yaw(self.yaw + delta);
    }

    fn fix_eye(&mut self) {
        let pitch_cos = self.pitch.cos();
        self.eye = self.target
            - self.zoom
                * Vec3::new(
                    self.yaw.sin() * pitch_cos,
                    self.pitch.sin(),
                    self.yaw.cos() * pitch_cos,
                );
    }

    pub fn set_aspect(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
        self.updated = true;
    }

    pub fn get_view_proj_matrix(&self) -> CameraUniform {
        CameraUniform {
            view_position: [self.eye.x, self.eye.y, self.eye.z, 1.0],
            view_proj: self.build_view_projection_matrix().to_cols_array_2d(),
        }
    }
}
