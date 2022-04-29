pub struct HdrBackBuffer {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,

    pub render_bind_group: wgpu::BindGroup,
    pub storage_bind_group: wgpu::BindGroup,
}

impl HdrBackBuffer {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
    pub const DEFAULT_RESOLUTION: (u32, u32) = (1280, 720);
    pub const DESC_COMPUTE: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Storage Texture Layour"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadWrite,
                    format: Self::FORMAT,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }],
        };
    pub const DESC_RENDER: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("BackBuffer: Render Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        };

    pub fn new(device: &wgpu::Device, (width, height): (u32, u32)) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture: HdrBackbuffer"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        });
        let texture_view = texture.create_view(&Default::default());

        let binding_resource = &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&texture_view),
        }];
        let render_bind_group_layout = device.create_bind_group_layout(&Self::DESC_RENDER);
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("BackBuffer: Render Bind Group"),
            layout: &render_bind_group_layout,
            entries: binding_resource,
        });

        let storage_bind_group_layout = device.create_bind_group_layout(&Self::DESC_COMPUTE);
        let storage_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("BackBuffer: Render Bind Group"),
            layout: &storage_bind_group_layout,
            entries: binding_resource,
        });

        Self {
            texture,
            texture_view,

            render_bind_group,
            storage_bind_group,
        }
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        Self::FORMAT
    }
}
