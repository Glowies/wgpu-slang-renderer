use wgpu::{RenderPass, util::DeviceExt};

use crate::{
    create_render_pipeline, hdr, resources, texture, wgpu_include_slang_shader,
    wgpu_traits::AsBindGroup,
};

pub type ShCoefficients = Vec<[f32; 3]>;
pub type UniformShCoefficients = [[f32; 4]; 9];

fn uniformify_sh_coefficients(coeffs: &ShCoefficients) -> UniformShCoefficients {
    let mut result: UniformShCoefficients = [[0.0; 4]; 9];

    for (idx, val) in coeffs.iter().enumerate() {
        result[idx] = [val[0], val[1], val[2], 0.0];
    }

    result
}

pub struct SkyPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
    sky_texture: texture::Texture,
    sky_uniform: SkyUniform,
    pub properties: SkyProperties,
    uniform_buffer: Option<wgpu::Buffer>,
}

impl SkyPipeline {
    pub async fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        // let sky_path = "rogland_clear_night_cube.ktx2";
        // let sky_path = "monkstown_castle.ktx2";
        let sky_path = "large-corridor.ktx2";
        // let sky_path = "debug-sky-faces.ktx2";
        // let sky_path = "debug-sky-green-dot.ktx2";
        let sky_texture = resources::load_texture(sky_path, device, queue, Default::default())
            .await
            .expect("Failed to load sky texture.");
        let mip_count = sky_texture.texture.mip_level_count();

        let sh_path = format!("{sky_path}.bin");
        let sky_sh_coefficients = resources::load_sh_coefficients(&sh_path)
            .await
            .expect("Failed to load SH coefficients file for sky texture.");

        let properties = SkyProperties {
            sh_coefficients: sky_sh_coefficients,
            mip_count,
            ..Default::default()
        };

        let sky_uniform: SkyUniform = (&properties).into();

        let sky_bind_group_layout =
            Self::create_bind_group_layout(device, "Environment Bind Group Layout");

        let pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Sky Pipeline Layout"),
                bind_group_layouts: &[camera_bind_group_layout, &sky_bind_group_layout],
                push_constant_ranges: &[],
            });

            let shader = wgpu_include_slang_shader!("sky");
            create_render_pipeline(
                device,
                &layout,
                hdr::HDR_BUFFER_FORMAT,
                Some(texture::Texture::DEPTH_FORMAT),
                &[],
                wgpu::PrimitiveTopology::TriangleList,
                shader,
            )
        };

        let mut sky_pipeline = Self {
            pipeline,
            bind_group_layout: sky_bind_group_layout,
            bind_group: None,
            sky_texture,
            uniform_buffer: None,
            properties,
            sky_uniform,
        };

        sky_pipeline.init_all(device);

        sky_pipeline
    }

    pub fn draw_in_render_pass(
        &self,
        render_pass: &mut RenderPass,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, self.bind_group(), &[]);
        render_pass.draw(0..3, 0..1);
    }

    pub fn uniform_buffer(&self) -> &wgpu::Buffer {
        if self.uniform_buffer.is_none() {
            panic!("Uniform Buffer for HDR Pipeline has not been initialized!");
        }

        self.uniform_buffer.as_ref().unwrap()
    }
}

impl AsBindGroup for SkyPipeline {
    fn bind_group_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::Cube,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ]
    }

    fn init_bind_group(&mut self, device: &wgpu::Device) {
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Environment Bind Group"),
            layout: self.bind_group_layout(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.sky_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sky_texture.sampler),
                },
            ],
        }))
    }

    fn init_binding_resources(&mut self, device: &wgpu::Device) {
        self.uniform_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Sky Uniform Buffer"),
                contents: bytemuck::cast_slice(&[self.sky_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        );
    }

    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    fn bind_group(&self) -> &wgpu::BindGroup {
        if self.bind_group.is_none() {
            panic!("Bind Group for HdrPipeline has not been initialized.");
        }

        self.bind_group.as_ref().unwrap()
    }

    fn update_binding_resources(&mut self) {
        self.sky_uniform = (&self.properties).into();
    }

    fn queue_write_binding_resources(&mut self, queue: &wgpu::Queue) {
        self.update_binding_resources();

        queue.write_buffer(
            self.uniform_buffer(),
            0,
            bytemuck::cast_slice(&[self.sky_uniform]),
        );
    }
}

pub struct SkyProperties {
    sh_coefficients: ShCoefficients,
    mip_count: u32,
    pub exposure_ev: f32,
    pub debug_sh_coefficients: bool,
}

impl Default for SkyProperties {
    fn default() -> Self {
        Self {
            exposure_ev: -2.0,
            mip_count: 1,
            sh_coefficients: vec![[0.0; 3]; 9],
            debug_sh_coefficients: false,
        }
    }
}

impl From<&SkyProperties> for SkyUniform {
    fn from(value: &SkyProperties) -> Self {
        Self {
            sh_coefficients: uniformify_sh_coefficients(&value.sh_coefficients),
            mip_count: value.mip_count as f32,
            exposure_linear: f32::powf(2.0, value.exposure_ev),
            debug_sh: value.debug_sh_coefficients as u8 as f32,
            _padding: [0.0; 1],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SkyUniform {
    pub sh_coefficients: UniformShCoefficients,
    pub exposure_linear: f32,
    pub debug_sh: f32,
    pub mip_count: f32,
    pub _padding: [f32; 1],
}
