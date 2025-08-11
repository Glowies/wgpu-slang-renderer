use std::sync::Arc;

use crate::{
    texture::{self, FallbackTextures},
    wgpu_traits::AsBindGroup,
};

pub struct Material {
    pub name: String,
    pub diffuse_texture: Arc<texture::Texture>,
    pub normal_texture: Arc<texture::Texture>,
    pub arm_texture: Arc<texture::Texture>,

    // AsBindGroup fields
    bind_group: Option<wgpu::BindGroup>,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl Material {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        diffuse_texture: Arc<texture::Texture>,
        normal_texture: Arc<texture::Texture>,
        arm_texture: Arc<texture::Texture>,
    ) -> Self {
        let bind_group_layout = Self::create_bind_group_layout(device, name);
        let mut material = Self {
            name: name.to_string(),
            diffuse_texture,
            normal_texture,
            arm_texture,
            bind_group: None,
            bind_group_layout,
        };

        material.init_bind_group(device);

        material
    }

    pub fn create_default(device: &wgpu::Device, fallback_textures: &FallbackTextures) -> Self {
        let diffuse_texture = fallback_textures.base_color();
        let normal_texture = fallback_textures.normal();
        let arm_texture = fallback_textures.arm();

        let bind_group_layout =
            Self::create_bind_group_layout(device, "Default Material Bind Group Layout");
        let mut material = Self {
            name: String::from("Default Material"),
            diffuse_texture,
            normal_texture,
            arm_texture,
            bind_group: None,
            bind_group_layout,
        };

        material.init_bind_group(device);

        material
    }
}

impl AsBindGroup for Material {
    fn bind_group_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
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
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ]
    }

    fn init_bind_group(&mut self, device: &wgpu::Device) {
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: self.bind_group_layout(),
            label: Some(&self.name),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.normal_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&self.arm_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.arm_texture.sampler),
                },
            ],
        }));
    }

    fn init_binding_resources(&mut self, device: &wgpu::Device) {
        let _ = device;
        // Binding resources are already initialized in the Texture fields
    }

    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    fn bind_group(&self) -> &wgpu::BindGroup {
        if self.bind_group.is_none() {
            panic!(
                "Bind Group Layout for Material ({}) has not been initialized.",
                self.name
            );
        }

        self.bind_group.as_ref().unwrap()
    }

    fn update_binding_resources(&mut self) {}

    fn queue_write_binding_resources(&mut self, queue: &wgpu::Queue) {
        let _ = queue;
    }
}
