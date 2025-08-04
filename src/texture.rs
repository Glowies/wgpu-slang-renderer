use anyhow::*;
use image::{GenericImage, GenericImageView, Rgba};
use ktx2::SupercompressionScheme;

pub struct Texture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: wgpu::Extent3d,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_2d_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        mag_filter: wgpu::FilterMode,
        label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        Self::create_texture(
            device,
            label,
            size,
            format,
            usage,
            wgpu::TextureDimension::D2,
            mag_filter,
        )
    }

    pub fn create_texture(
        device: &wgpu::Device,
        label: Option<&str>,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        dimension: wgpu::TextureDimension,
        mag_filter: wgpu::FilterMode,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension,
            format,
            usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            size,
        }
    }

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // we actually don't need a Sampler for the Depth Texture but our implementation
        // of the Texture struct requires it. However, having this sampler allows us
        // to directly render the depth buffer if we ever want to.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            // VVVV this is important when we want to render the depth texture directly
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            size,
        }
    }

    pub fn create_default_diffuse(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut image = image::DynamicImage::new_rgb8(1, 1);
        image.put_pixel(0, 0, Rgba::from([255, 0, 255, 255]));

        let options = TextureImportOptions {
            label: Some("Default Diffuse Texture"),
            is_linear: false,
            ..Default::default()
        };

        Self::from_image(device, queue, &image, options).unwrap()
    }

    pub fn create_default_normal(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut image = image::DynamicImage::new_rgb8(1, 1);
        image.put_pixel(0, 0, Rgba::from([128, 128, 255, 255]));

        let options = TextureImportOptions {
            label: Some("Default Normal Texture"),
            is_linear: true,
            ..Default::default()
        };

        Self::from_image(device, queue, &image, options).unwrap()
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        options: TextureImportOptions,
    ) -> Result<Self> {
        let TextureImportOptions { label, is_lut, .. } = options;

        match is_lut {
            true => {
                let reader = ktx2::Reader::new(bytes)
                    .expect("Can't create reader. LUT textures need to be Ktx2 files.");
                Self::from_image_lut(device, queue, &reader, label)
            }
            false => {
                let img = image::load_from_memory(bytes)?;
                Self::from_image(device, queue, &img, options)
            }
        }
    }

    pub fn from_image_lut(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        reader: &ktx2::Reader<&[u8]>,
        label: Option<&str>,
    ) -> Result<Self> {
        let header = reader.header();
        let cubesize = header.pixel_width;
        println!("{:#?}", header);
        // TODO: Make sure the format matches the format from the Ktx2 file
        let format = wgpu::TextureFormat::Rgb9e5Ufloat;
        let size = wgpu::Extent3d {
            width: cubesize,
            height: cubesize,
            depth_or_array_layers: cubesize,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Handle supercompression
        let mut levels = Vec::new();
        if let Some(supercompression_scheme) = header.supercompression_scheme {
            for (_level_index, level) in reader.levels().enumerate() {
                match supercompression_scheme {
                    SupercompressionScheme::Zstandard => {
                        levels.push(zstd::decode_all(level.data)?);
                    }
                    _ => {
                        return Err(Error::msg(format!(
                            "Unsupported supercompression scheme: {supercompression_scheme:?}. Only zstd is supported.",
                        )));
                    }
                }
            }
        } else {
            levels = reader.levels().map(|level| level.data.to_vec()).collect();
        }

        // Collect all level data into a contiguous buffer
        let mut image_data = Vec::new();
        image_data.reserve_exact(levels.iter().map(Vec::len).sum());
        levels.iter().for_each(|level| image_data.extend(level));

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &image_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * cubesize),
                rows_per_image: Some(cubesize),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            dimension: Some(wgpu::TextureViewDimension::D3),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
            size,
        })
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        options: TextureImportOptions,
    ) -> Result<Self> {
        let TextureImportOptions {
            label, is_linear, ..
        } = options;

        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();
        let format = if is_linear {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
            size,
        })
    }
}

pub struct TextureImportOptions<'a> {
    pub label: Option<&'a str>,
    pub is_lut: bool,
    pub is_linear: bool,
}

impl<'a> Default for TextureImportOptions<'a> {
    fn default() -> Self {
        Self {
            label: None,
            is_lut: false,
            is_linear: false,
        }
    }
}
