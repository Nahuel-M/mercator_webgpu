use wgpu::{Device, Extent3d, Queue, Sampler, Texture};

pub struct EarthTexture {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl EarthTexture {
    pub fn new(device: &Device, queue: &Queue) -> Self {
        // use image::GenericImageView;
        // let equirectangular_earth = include_bytes!("EarthEquirectangular2k.jpg");
        // let equirectangular_earth = image::load_from_memory(equirectangular_earth).unwrap();
        // let dimensions = equirectangular_earth.dimensions();
        // let equirectangular_earth = equirectangular_earth.to_rgba8();
        // fs::write("earth_unpacked", equirectangular_earth).unwrap();
        let equirectangular_earth = include_bytes!("earth_unpacked.buf");
        let dimensions = (2048, 1024);

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let earth_texture = EarthTexture::create_texture(device, texture_size);

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &earth_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &equirectangular_earth[0..],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let earth_texture_view = earth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let earth_sampler = EarthTexture::create_sampler(device);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&earth_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&earth_sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        Self {
            bind_group_layout,
            bind_group,
        }
    }

    pub fn create_texture(device: &Device, size: Extent3d) -> Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("equirectangular_earth"),
            view_formats: &[],
        })
    }

    pub fn create_sampler(device: &Device) -> Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }
}
