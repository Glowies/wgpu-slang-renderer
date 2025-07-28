use wgpu::{Operations, util::RenderEncoder};

use crate::{create_render_pipeline, texture, wgpu_traits::AsBindGroup};

/// Owns the render texture and controls tonemapping
pub struct HdrPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
    texture: Option<texture::Texture>,
    width: u32,
    height: u32,
}

impl HdrPipeline {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let width = config.width.max(1);
        let height = config.height.max(1);

        let bind_group_layout =
            Self::create_bind_group_layout(device, "HDR Pipeline Bind Group Layout");

        let shader = wgpu::include_wgsl!("shaders/hdr.wgsl");
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

        let mut hdr_pipeline = Self {
            pipeline,
            bind_group_layout,
            width,
            height,
            bind_group: None,
            texture: None,
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
                view: &surface_texture_view,
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

    fn texture(&self) -> &texture::Texture {
        if let None = self.texture {
            panic!("Texture for HDR Pipeline has not been initialized!");
        }

        self.texture.as_ref().unwrap()
    }

    pub fn texture_view(&self) -> &wgpu::TextureView {
        &self.texture().view
    }

    pub fn texture_format(&self) -> wgpu::TextureFormat {
        self.texture().texture.format()
    }
}

impl AsBindGroup for HdrPipeline {
    fn bind_group_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
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
                    resource: wgpu::BindingResource::TextureView(&self.texture().view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.texture().sampler),
                },
            ],
        }));
    }

    fn init_binding_resources(&mut self, device: &wgpu::Device) {
        // We could use `Rgba32Float`, but that requires some extra
        // features to be enabled for rendering.
        let format = wgpu::TextureFormat::Rgba16Float;
        self.texture = Some(texture::Texture::create_2d_texture(
            device,
            self.width,
            self.height,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            wgpu::FilterMode::Nearest,
            Some("HDR Pipeline Texture"),
        ));
    }

    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    fn bind_group(&self) -> &wgpu::BindGroup {
        if let None = self.bind_group {
            panic!("Bind Group for HdrPipeline has not been initialized.");
        }

        self.bind_group.as_ref().unwrap()
    }

    fn update_binding_resources(&mut self) {
        // Do nothing
    }

    fn queue_write_binding_resources(&mut self, queue: &wgpu::Queue) {
        let _ = queue;
    }
}
