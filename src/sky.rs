use crate::{create_render_pipeline, hdr, resources, texture, wgpu_traits::AsBindGroup};

struct SkyPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
    sky_texture: texture::Texture,
}

impl SkyPipeline {
    pub async fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let sky_texture = resources::load_texture(
            "rogland_clear_night_256cube.ktx2",
            device,
            queue,
            Default::default(),
        )
        .await
        .expect("Failed to load sky texture.");

        let sky_bind_group_layout =
            Self::create_bind_group_layout(device, "Environment Bind Group Layout");

        let pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Sky Pipeline Layout"),
                bind_group_layouts: &[camera_bind_group_layout, &sky_bind_group_layout],
                push_constant_ranges: &[],
            });

            let shader = wgpu::include_wgsl!("shaders/sky.wgsl");
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
        };

        sky_pipeline.init_all(device);

        sky_pipeline
    }
}

impl AsBindGroup for SkyPipeline {
    fn bind_group_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::Cube,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
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
                    resource: wgpu::BindingResource::TextureView(&self.sky_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sky_texture.sampler),
                },
            ],
        }))
    }

    fn init_binding_resources(&mut self, device: &wgpu::Device) {
        let _ = device;
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
        todo!()
    }

    fn queue_write_binding_resources(&mut self, queue: &wgpu::Queue) {
        let _ = queue;
    }
}
