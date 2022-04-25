use std::path::Path;

use crate::{
    camera::CameraBinding,
    context::{global_ubo::GlobalUniformBinding, Uniform},
    utils::shader_compiler::ShaderCompiler,
    watcher::ReloadablePipeline,
};

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

impl<'a> BasicPipeline {
    pub fn record<'pass>(
        &'a self,
        rpass: &mut wgpu::RenderPass<'pass>,
        uniform_bind_group: &'a GlobalUniformBinding,
        camera_bind_group: &'a CameraBinding,
    ) where
        'a: 'pass,
    {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &uniform_bind_group.binding, &[]);
        rpass.set_bind_group(1, &camera_bind_group.bind_group, &[]);
        rpass.draw(0..3, 0..1);
    }
}

impl ReloadablePipeline for BasicPipeline {
    fn reload(&mut self, device: &wgpu::Device, module: &wgpu::ShaderModule) {
        *self = Self::new_with_module(device, self.surface_format, module);
    }
}
