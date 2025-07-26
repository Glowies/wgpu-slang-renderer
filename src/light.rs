use std::ops::Range;

use wgpu::{BindGroup, BindGroupLayout, Buffer, Queue, util::DeviceExt};

use crate::model::{Mesh, Model};
use crate::wgpu_traits::WgpuUniform;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub intensity: f32,
    pub color: [f32; 3],
    // The alignment of WGSL structs need to be powers of 2.
    // In this case, our vec3 would have a size of 12 bytes, so we need
    // to align to the next largest power of two: 16.
    // An alternative is to just use vec4's even if we need only 3 channels.
    pub _padding: u32,
}

pub struct LightProperties {
    pub position: cgmath::Vector3<f32>,
    pub color: [f32; 3],
    pub intensity: f32,
}

impl Default for LightProperties {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0].into(),
            intensity: 1.0,
            color: [1.0, 1.0, 1.0],
        }
    }
}

pub struct Light {
    pub properties: LightProperties,
    pub uniform: LightUniform,
    buffer: Option<Buffer>,
    bind_group_layout: Option<BindGroupLayout>,
    bind_group: Option<BindGroup>,
}

impl Default for Light {
    fn default() -> Self {
        let properties = LightProperties::default();
        let uniform = Self::uniform_from_properties(&properties);

        Self {
            properties,
            uniform,
            buffer: None,
            bind_group_layout: None,
            bind_group: None,
        }
    }
}

impl Light {
    pub fn new(properties: LightProperties) -> Self {
        let uniform = Self::uniform_from_properties(&properties);

        Self {
            properties,
            uniform,
            buffer: None,
            bind_group_layout: None,
            bind_group: None,
        }
    }

    pub fn queue_write_buffer(&mut self, queue: &Queue) {
        self.update_uniform();

        queue.write_buffer(
            self.buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }

    fn update_uniform(&mut self) {
        self.uniform = Self::uniform_from_properties(&self.properties);
    }

    fn uniform_from_properties(properties: &LightProperties) -> LightUniform {
        LightUniform {
            position: properties.position.into(),
            intensity: properties.intensity,
            color: properties.color,
            _padding: 0,
        }
    }
}

impl WgpuUniform for Light {
    fn init_uniform_buffer(&mut self, device: &wgpu::Device) {
        self.buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[self.uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        );
    }

    fn init_bind_group_layout(&mut self, device: &wgpu::Device) {
        self.bind_group_layout = Some(device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            },
        ));
    }

    fn init_bind_group(&mut self, device: &wgpu::Device) {
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: self.bind_group_layout.as_ref().unwrap(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.buffer.as_ref().unwrap().as_entire_binding(),
            }],
        }));
    }

    fn uniform_buffer(&self) -> &Buffer {
        if let None = self.buffer {
            panic!("Uniform Buffer for Light has not been initialized.");
        }

        self.buffer.as_ref().unwrap()
    }

    fn bind_group_layout(&self) -> &BindGroupLayout {
        if let None = self.bind_group_layout {
            panic!("Bind Group Layout for Light has not been initialized.");
        }

        self.bind_group_layout.as_ref().unwrap()
    }

    fn bind_group(&self) -> &BindGroup {
        if let None = self.bind_group {
            panic!("Bind Group for Light has not been initialized.");
        }

        self.bind_group.as_ref().unwrap()
    }
}

// model.rs
pub trait DrawLight<'a> {
    fn draw_light_mesh(
        &mut self,
        mesh: &'a Mesh,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_light_model(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_light_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawLight<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_light_mesh(
        &mut self,
        mesh: &'b Mesh,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_light_mesh_instanced(mesh, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, camera_bind_group, &[]);
        self.set_bind_group(1, light_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_light_model(
        &mut self,
        model: &'b Model,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_light_model_instanced(model, 0..1, camera_bind_group, light_bind_group);
    }
    fn draw_light_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            self.draw_light_mesh_instanced(
                mesh,
                instances.clone(),
                camera_bind_group,
                light_bind_group,
            );
        }
    }
}
