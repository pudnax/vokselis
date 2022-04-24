use std::path::Path;

use wgpu::util::DeviceExt;

use crate::{
    camera::CameraBinding,
    state::{
        global_ubo::GlobalUniformBinding, hdr_backbuffer::HdrBackBuffer,
        volume_texture::VolumeTexture, Uniform,
    },
    utils::shader_compiler::ShaderCompiler,
    watcher::ReloadablePipeline,
};

pub struct RaycastPipeline {
    pub pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: usize,
}

impl RaycastPipeline {
    pub fn from_path(
        device: &wgpu::Device,
        path: &Path,
        shader_compiler: &mut ShaderCompiler,
    ) -> Self {
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: path.to_str(),
            source: wgpu::ShaderSource::Wgsl(std::fs::read_to_string(path).unwrap().into()),
        });
        // pub fn from_path(device: &wgpu::Device, path: &Path, compiler: &mut ShaderCompiler) -> Self {
        //     let shader = unsafe {
        //         device.create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
        //             label: path.to_str(),
        //             source: compiler.create_shader_module(path).unwrap().into(),
        //         })
        //     };
        Self::new_with_module(device, &shader)
    }

    pub fn new_with_module(device: &wgpu::Device, module: &wgpu::ShaderModule) -> Self {
        let vertices = [
            1., 1., 0., 0., 1., 0., 1., 1., 1., 0., 1., 1., 0., 0., 1., 0., 1., 0., 0., 0., 0., 1.,
            1., 0., 1., 0., 0., 1., 1., 1., 1., 0., 1., 0., 0., 1., 1., 0., 0., 0., 0., 0.,
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Volume Vertex Buffer"),
            contents: bytemuck::cast_slice::<f32, _>(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let vertex_count = vertices.len() / 3;

        let pipeline = Self::make_pipeline(device, module);
        Self {
            pipeline,
            vertex_buffer,
            vertex_count,
        }
    }

    fn make_pipeline(device: &wgpu::Device, module: &wgpu::ShaderModule) -> wgpu::RenderPipeline {
        let global_bind_group_layout = device.create_bind_group_layout(&Uniform::DESC);
        let camera_bind_group_layout = device.create_bind_group_layout(&CameraBinding::DESC);
        let texture_bind_group_layout = device.create_bind_group_layout(&VolumeTexture::DESC);
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Screen Pass Layout"),
            bind_group_layouts: &[
                &global_bind_group_layout,
                &camera_bind_group_layout,
                &texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Raycast Pipeline"),
            layout: Some(&layout),
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: "fs_main",
                targets: &[HdrBackBuffer::FORMAT.into()],
            }),
            vertex: wgpu::VertexState {
                module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 3 * 4,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                cull_mode: Some(wgpu::Face::Front),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        pipeline
    }
}

impl<'a> RaycastPipeline {
    pub fn record<'pass>(
        &'a self,
        rpass: &mut wgpu::RenderPass<'pass>,
        uniform_bind_group: &'a GlobalUniformBinding,
        camera_bind_group: &'a CameraBinding,
        volume_texture: &'a VolumeTexture,
    ) where
        'a: 'pass,
    {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        rpass.set_bind_group(0, &uniform_bind_group.binding, &[]);
        rpass.set_bind_group(1, &camera_bind_group.bind_group, &[]);
        rpass.set_bind_group(2, &volume_texture.bind_group, &[]);
        rpass.draw(0..self.vertex_count as _, 0..1);
    }
}

impl ReloadablePipeline for RaycastPipeline {
    fn reload(&mut self, device: &wgpu::Device, module: &wgpu::ShaderModule) {
        self.pipeline = Self::make_pipeline(device, module);
    }
}
