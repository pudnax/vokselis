use std::path::Path;

use super::xor_compute;

use vokselis::{
    camera::CameraBinding,
    context::{GlobalUniformBinding, HdrBackBuffer, Uniform},
    dispatch_optimal,
    shader_compiler::ShaderCompiler,
    ReloadablePipeline,
};

pub struct RaycastPipeline {
    pub pipeline: wgpu::ComputePipeline,
}

impl RaycastPipeline {
    pub fn from_path(
        device: &wgpu::Device,
        path: &Path,
        shader_compiler: &mut ShaderCompiler,
    ) -> Self {
        let shader = unsafe {
            device.create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                label: path.to_str(),
                source: shader_compiler.create_shader_module(path).unwrap().into(),
            })
        };
        Self::new_with_module(device, &shader)
    }

    pub fn new_with_module(device: &wgpu::Device, module: &wgpu::ShaderModule) -> Self {
        let pipeline = Self::make_pipeline(device, module);
        Self { pipeline }
    }

    fn make_pipeline(device: &wgpu::Device, module: &wgpu::ShaderModule) -> wgpu::ComputePipeline {
        let global_bind_group_layout = device.create_bind_group_layout(&Uniform::DESC);
        let camera_bind_group_layout = device.create_bind_group_layout(&CameraBinding::DESC);
        let volume_bind_group_layout =
            device.create_bind_group_layout(&xor_compute::XorCompute::DESC_COMPUTE);
        let output_texture_bind_group_layot =
            device.create_bind_group_layout(&HdrBackBuffer::DESC_COMPUTE);
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Raycast Pass Layout"),
            bind_group_layouts: &[
                &global_bind_group_layout,
                &camera_bind_group_layout,
                &volume_bind_group_layout,
                &output_texture_bind_group_layot,
            ],
            push_constant_ranges: &[],
        });
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Raycast Pipeline"),
            layout: Some(&layout),
            module,
            entry_point: "cs_main",
        })
    }
}

impl<'a> RaycastPipeline {
    pub fn record<'pass>(
        &'a self,
        cpass: &mut wgpu::ComputePass<'pass>,
        uniform_bind_group: &'a GlobalUniformBinding,
        camera_bind_group: &'a CameraBinding,
        volume_texture: &'a wgpu::BindGroup,
        output_texture: &'a wgpu::BindGroup,
    ) where
        'a: 'pass,
    {
        cpass.set_pipeline(&self.pipeline);

        cpass.set_bind_group(0, &uniform_bind_group.binding, &[]);
        cpass.set_bind_group(1, &camera_bind_group.bind_group, &[]);
        cpass.set_bind_group(2, &volume_texture, &[]);
        cpass.set_bind_group(3, &output_texture, &[]);
        let (width, height) = HdrBackBuffer::DEFAULT_RESOLUTION;
        cpass.dispatch(dispatch_optimal(width, 16), dispatch_optimal(height, 16), 1);
    }
}

impl ReloadablePipeline for RaycastPipeline {
    fn reload(&mut self, device: &wgpu::Device, module: &wgpu::ShaderModule) {
        self.pipeline = Self::make_pipeline(device, module);
    }
}
