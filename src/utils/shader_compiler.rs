use std::path::Path;

use color_eyre::Result;
use naga::{
    back::spv::{self, BindingMap},
    front::wgsl,
    valid::{Capabilities, ValidationFlags, Validator},
};

pub struct ShaderCompiler {
    validator: Validator,
    writer: spv::Writer,
}

impl ShaderCompiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_shader_module(&mut self, path: &Path) -> Result<Vec<u32>> {
        let source = std::fs::read_to_string(&path)?;
        let module = match wgsl::parse_str(&source) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Shader compilation error:");
                e.emit_to_stderr(&source);
                return Err(e.into());
            }
        };
        let module_info = self.validator.validate(&module)?;
        let mut words = vec![];
        self.writer.write(&module, &module_info, None, &mut words)?;
        Ok(words)
    }
}

impl Default for ShaderCompiler {
    fn default() -> Self {
        let validator = Validator::new(ValidationFlags::all(), Capabilities::all());
        let options = get_options();
        let writer = spv::Writer::new(&options).unwrap();
        Self { validator, writer }
    }
}

// https://github.com/gfx-rs/wgpu/blob/master/wgpu-hal/src/vulkan/adapter.rs#L1166
fn get_options() -> spv::Options {
    let mut capabilities = vec![
        spv::Capability::Shader,
        spv::Capability::Matrix,
        spv::Capability::Sampled1D,
        spv::Capability::Image1D,
        spv::Capability::ImageQuery,
        spv::Capability::DerivativeControl,
        spv::Capability::SampledCubeArray,
        spv::Capability::SampleRateShading,
        //Note: this is requested always, no matter what the actual
        // adapter supports. It's not the responsibility of SPV-out
        // translation to handle the storage support for formats.
        spv::Capability::StorageImageExtendedFormats,
        //TODO: fill out the rest
    ];

    capabilities.push(spv::Capability::MultiView);

    let mut flags = spv::WriterFlags::empty();
    flags.set(
        spv::WriterFlags::DEBUG,
        true,
        // self.instance.flags.contains(crate::InstanceFlags::DEBUG),
    );
    flags.set(
        spv::WriterFlags::LABEL_VARYINGS,
        true, // self.phd_capabilities.properties.vendor_id != crate::auxil::db::qualcomm::VENDOR,
    );
    flags.set(
        spv::WriterFlags::FORCE_POINT_SIZE,
        //Note: we could technically disable this when we are compiling separate entry points,
        // and we know exactly that the primitive topology is not `PointList`.
        // But this requires cloning the `spv::Options` struct, which has heap allocations.
        true, // could check `super::Workarounds::SEPARATE_ENTRY_POINTS`
    );
    spv::Options {
        binding_map: BindingMap::new(),
        lang_version: (1, 0),
        flags,
        capabilities: Some(capabilities.iter().cloned().collect()),
        bounds_check_policies: naga::proc::BoundsCheckPolicies {
                    index: naga::proc::BoundsCheckPolicy::Unchecked,
                    buffer:
                    // if self.private_caps.robust_buffer_access {
                        naga::proc::BoundsCheckPolicy::Unchecked,
                    // } else {
                        // naga::proc::BoundsCheckPolicy::Restrict,
                    // },
                    image:
                    // if self.private_caps.robust_image_access {
                        naga::proc::BoundsCheckPolicy::Unchecked,
                    // } else {
                        // naga::proc::BoundsCheckPolicy::Restrict
                    // },
                },
    }
}
