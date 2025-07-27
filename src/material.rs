use crate::{texture, wgpu_traits::AsBindGroup};

pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub normal_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        diffuse_texture: texture::Texture,
        normal_texture: texture::Texture, // NEW!
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            label: Some(name),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
        });

        Self {
            name: name.to_string(),
            diffuse_texture,
            normal_texture,
            bind_group,
        }
    }

    pub fn create_default(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) -> anyhow::Result<Self> {
        let diffuse_texture = texture::Texture::create_default_diffuse(device, queue);
        let normal_texture = texture::Texture::create_default_normal(device, queue);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            label: None,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
        });

        Ok(Self {
            name: String::from("Default Material"),
            diffuse_texture,
            normal_texture,
            bind_group,
        })
    }
}

// impl AsBindGroup for Material {
//     // Note to self:
//     // We want to implement this for Material but materials don't have a uniform buffer, they have Textures,Samplers,Views
//     // Maybe we need two additional traits, both of which derive AsBindGroup: UniformBuffer, TextureBuffer
//     // We would move the uniform buffer related methods to the UniformBuffer trait and have material specific
//     // methods created in TextureBuffer
//     //
//     // Also maybe we should change get_bind_group_layout() to be in Self instead of self before we do that...
// }
