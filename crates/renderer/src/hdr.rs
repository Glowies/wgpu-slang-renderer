use wgpu::{Operations, util::DeviceExt};

use crate::{
    create_render_pipeline, resources,
    texture::{self, TextureImportOptions},
    wgpu_traits::AsBindGroup,
};

// We could use `Rgba32Float`, but that requires some extra
// features to be enabled for rendering.
pub const HDR_BUFFER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// Owns the render texture and controls tonemapping
pub struct HdrPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
    uniform_buffer: Option<wgpu::Buffer>,
    render_texture: Option<texture::Texture>,
    display_view_lut_texture: texture::Texture,
    width: u32,
    height: u32,
    pub properties: HdrViewProperties,
    view_uniform: HdrViewUniform,
}

pub struct HdrViewProperties {
    pub exposure_ev: f32,
}

impl Default for HdrViewProperties {
    fn default() -> Self {
        Self { exposure_ev: 0.0 }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HdrViewUniform {
    pub exposure_linear: f32,
    // I don't quite understand this.. I needed to make this a 32 byte struct
    // because the struct in the shader expects 32 bytes, *even though its
    // just made up of a f32 and a vec3<f32>*. Shouldn't that be 16 bytes!?
    pub _padding: [f32; 7],
}

impl From<&HdrViewProperties> for HdrViewUniform {
    fn from(value: &HdrViewProperties) -> Self {
        Self {
            exposure_linear: f32::powf(2.0, value.exposure_ev),
            _padding: [0.0; 7],
        }
    }
}

impl HdrPipeline {
    pub async fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let width = config.width.max(1);
        let height = config.height.max(1);

        let display_view = resources::load_texture(
            "shaper_to_displayP3_48.ktx2",
            device,
            queue,
            TextureImportOptions {
                label: Some("Display View LUT"),
            },
        )
        .await
        .unwrap();

        let bind_group_layout =
            Self::create_bind_group_layout(device, "HDR Pipeline Bind Group Layout");

        let shader = wgpu::include_wgsl!(concat!(env!("OUT_DIR"), "/shaders/hdr.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("HDR Render Pipeline"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = create_render_pipeline(
            device,
            &pipeline_layout,
            config.format,
            None,
            // We'll use some math to generate the vertex data in
            // the shader, so we don't need any vertex buffers
            &[],
            wgpu::PrimitiveTopology::TriangleList,
            shader,
        );

        let properties = HdrViewProperties::default();
        let view_uniform: HdrViewUniform = (&properties).into();

        let mut hdr_pipeline = Self {
            pipeline,
            bind_group_layout,
            width,
            height,
            bind_group: None,
            render_texture: None,
            properties,
            view_uniform,
            uniform_buffer: None,
            display_view_lut_texture: display_view,
        };

        hdr_pipeline.init_all(device);

        hdr_pipeline
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.init_binding_resources(device);
        self.init_bind_group(device);
    }

    pub fn draw_to_surface(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        surface_texture_view: &wgpu::TextureView,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("HDR Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_texture_view,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, self.bind_group(), &[]);
        pass.draw(0..3, 0..1);
    }

    fn render_texture(&self) -> &texture::Texture {
        if self.render_texture.is_none() {
            panic!("Texture for HDR Pipeline has not been initialized!");
        }

        self.render_texture.as_ref().unwrap()
    }

    pub fn texture_view(&self) -> &wgpu::TextureView {
        &self.render_texture().view
    }

    pub fn texture_format(&self) -> wgpu::TextureFormat {
        self.render_texture().texture.format()
    }

    pub fn uniform_buffer(&self) -> &wgpu::Buffer {
        if self.uniform_buffer.is_none() {
            panic!("Uniform Buffer for HDR Pipeline has not been initialized!");
        }

        self.uniform_buffer.as_ref().unwrap()
    }
}

impl AsBindGroup for HdrPipeline {
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
                    view_dimension: wgpu::TextureViewDimension::D2,
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
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D3,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
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
            label: Some("HDR Pipeline Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.render_texture().view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.render_texture().sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(
                        &self.display_view_lut_texture.view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(
                        &self.display_view_lut_texture.sampler,
                    ),
                },
            ],
        }));
    }

    fn init_binding_resources(&mut self, device: &wgpu::Device) {
        self.render_texture = Some(texture::Texture::create_2d_texture(
            device,
            self.width,
            self.height,
            HDR_BUFFER_FORMAT,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            wgpu::FilterMode::Nearest,
            Some("HDR Pipeline Texture"),
        ));

        self.uniform_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("View Uniform Buffer"),
                contents: bytemuck::cast_slice(&[self.view_uniform]),
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
        self.view_uniform = (&self.properties).into();
    }

    fn queue_write_binding_resources(&mut self, queue: &wgpu::Queue) {
        self.update_binding_resources();

        queue.write_buffer(
            self.uniform_buffer(),
            0,
            bytemuck::cast_slice(&[self.view_uniform]),
        );
    }
}
