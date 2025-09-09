use anyhow::*;
use std::sync::Arc;
use wgpu::{TexelCopyBufferLayout, TextureFormat};

use crate::{
    ktx2::{get_raw_level_data, ktx_to_wgpu_format, size_and_dims_from_header},
    resources,
};

fn buffer_layout_from_wgpu_format(
    format: TextureFormat,
    size: wgpu::Extent3d,
) -> anyhow::Result<TexelCopyBufferLayout> {
    match format {
        TextureFormat::Rgba16Float => Ok(TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(8 * size.width),
            rows_per_image: Some(size.height),
        }),
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm | TextureFormat::Rgb9e5Ufloat => {
            Ok(TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            })
        }
        TextureFormat::Rg8Unorm => Ok(TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(2 * size.width),
            rows_per_image: Some(size.height),
        }),
        _ => Err(anyhow!(
            "TexelCopyBufferLayout unknown for format: {:?}",
            format
        )),
    }
}

pub struct Texture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
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
        }
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        options: TextureImportOptions,
    ) -> Result<Self> {
        let TextureImportOptions { label, .. } = options;

        let reader = ktx2::Reader::new(bytes)
            .expect("Can't create Ktx2 reader. Textures need to be Ktx2 files.");
        Self::texture_from_ktx(device, queue, &reader, label)
    }

    pub fn texture_from_ktx(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        reader: &ktx2::Reader<&[u8]>,
        label: Option<&str>,
    ) -> Result<Self> {
        let header = reader.header();

        let format = ktx_to_wgpu_format(header.format)?;

        let (size, dimension, view_dimension) = size_and_dims_from_header(header);
        let mut size = size;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: header.level_count,
            sample_count: 1,
            dimension,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Handle supercompression
        let levels = get_raw_level_data(reader)?;

        // Copy each level (mip) one-by-one into the Texture buffer
        let mut curr_mip = 0;
        levels.iter().for_each(|level| {
            let copy_layout = buffer_layout_from_wgpu_format(format, size).unwrap();
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: curr_mip,
                    origin: wgpu::Origin3d::ZERO,
                },
                level,
                copy_layout,
                size,
            );

            // Make size adjustments per mip level
            curr_mip += 1;
            size.width /= 2;
            size.height /= 2;
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            dimension: Some(view_dimension),
            format: Some(format),
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
        })
    }
}

// We want the default values for all these types, so we don't
// need to explicitly impl Default
#[derive(Default)]
pub struct TextureImportOptions<'a> {
    pub label: Option<&'a str>,
}

pub struct FallbackTextures {
    base_color: Arc<Texture>,
    normal: Arc<Texture>,
    arm: Arc<Texture>,
}

impl FallbackTextures {
    pub async fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let base_color = Arc::new(
            resources::load_texture(
                "default-base-color-srgb.ktx2",
                device,
                queue,
                Default::default(),
            )
            .await
            .expect("Failed to load fallback default texture for Base Color."),
        );
        let normal = Arc::new(
            resources::load_texture("default-normal.ktx2", device, queue, Default::default())
                .await
                .expect("Failed to load fallback default texture for Normal."),
        );
        let arm = Arc::new(
            resources::load_texture("default-arm.ktx2", device, queue, Default::default())
                .await
                .expect("Failed to load fallback default texture for ARM."),
        );

        Self {
            base_color,
            normal,
            arm,
        }
    }

    pub fn base_color(&self) -> Arc<Texture> {
        self.base_color.clone()
    }

    pub fn normal(&self) -> Arc<Texture> {
        self.normal.clone()
    }

    pub fn arm(&self) -> Arc<Texture> {
        self.arm.clone()
    }
}
