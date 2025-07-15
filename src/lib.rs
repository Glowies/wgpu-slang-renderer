mod basic_types;
mod input_handling;
mod texture;

use basic_types::{Camera, CameraUniform};
use input_handling::Input;
use std::{cmp, sync::Arc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition, Size},
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

pub struct State {
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    clear_color: wgpu::Color,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,
    index_buffer: wgpu::Buffer,
    is_surface_configured: bool,
    num_indices: u32,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    surface: wgpu::Surface<'static>,
    vertex_buffer: wgpu::Buffer,
    window: Arc<Window>,
    input: Input,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        // The Instance is used to create the Surfaces and the Adapters
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        // The Surface represents the part of the window that we will
        // draw on. It needs to be specified when requesting an Adapter
        // so that we get an Adapter that can draw on this particular
        // Surface.
        let surface = instance.create_surface(window.clone()).unwrap();

        // The Adapter is our handler for the GPU. We can use it to
        // get information about the GPU hardware. It will be used
        // to create the Device and Queue later.
        //
        // Instead of .request_adapter() we can also use
        // .enumerate_adapters() to loop over all possible Adapters.
        // Although, .enumerate_adapters() is not available on WASM!
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;
        // When requesting an Adapter, we need to pass in some options:
        // - Power Preference: It's either LowPower or HighPerformance
        // - Compatible Surface: Specifies that the found Adapter should
        //    be capable of outputting to the given Surface.
        // - Force Fallback Adapter: Forces wgpu to use an Adapter that
        //    would work on ALL hardware. This typically mean software
        //    rendering instead of relying on hardware.

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // Not all features of WGPU are supported in WebGL
                // so we need to disable some for that target arch
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::downlevel_defaults()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);

        // For the surface format, we assume sRGB surface texture.
        // We could use a different format, but we would have to
        // account for it when drawing the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        // when we eventually have HDR support, we should use the following instead?
        // let surface_format = wgpu::TextureFormat::Rgba16Float;

        // create its SurfaceTextures.
        // The RENDER_ATTACHMENT Usage specifies that the SurfaceTexture
        // will be used to write to the screen.
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // load our image using the `image` crate.
        // Note that decoding JPEGs on WASM is not performant. It would be better
        // to use the browsers native decoders when building for WASM.
        let diffuse_bytes = include_bytes!("textures/happy-tree.png");
        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy_tree.png").unwrap();

        // We need a BindGroup to describe a set of resources (eg. texture) and how
        // they can be accessed by our shader. However, before we do that, we need
        // to define a BindGroupLayout that we can use to create a BindGroup.
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Texture Bind Group Layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Diffuse Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });

        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: (surface_config.width as f32) / (surface_config.height as f32),
            fov_y: 45.0,
            z_near: 0.1,
            z_far: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        // We need to pass our Camera information as a Uniform Buffer inside of
        // a Bind Group.
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        // this is a generic Buffer, so we need to tell the layout if this
                        // buffer has dynamic offset. (This is useful when you have buffer entires
                        // that can vary in size). For the camera, this size is constant so we
                        // set this to false. However, if we set it to true, we would have to pass
                        // in our manual offsets when we call render_pass.set_bind_group()
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Camera Bind Group Layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("Camera Bind Group"),
        });

        // create our Shader Module using the .wgsl file
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(basic_types::SAMPLE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(basic_types::SAMPLE_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = basic_types::SAMPLE_INDICES.len() as u32;

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[basic_types::Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // use format from our surface so that we can easily
                    // copy this result into the surface
                    format: surface_config.format,
                    // just replace the pixels that get written to
                    blend: Some(wgpu::BlendState::REPLACE),
                    // write all color channels
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            // the PrimitiveState describes how the vertices should be
            // interpreted when being converted to triangles
            primitive: wgpu::PrimitiveState {
                // TriangleList makes it so every 3 vertices in the result
                // will represent one triangle
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let clear_color = wgpu::Color {
            r: 0.2,
            g: 0.2,
            b: 0.2,
            a: 1.0,
        };

        Ok(Self {
            surface,
            device,
            queue,
            config: surface_config,
            is_surface_configured: false,
            window,
            clear_color,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_bind_group,
            diffuse_texture,
            camera,
            camera_uniform,
            camera_bind_group,
            camera_buffer,
            input: Input::new(),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width < 1 || height < 1 {
            return;
        }

        let max_size = self.device.limits().max_texture_dimension_2d;

        self.config.width = cmp::min(width, max_size);
        self.config.height = cmp::min(height, max_size);

        // This is where the Surface gets configured.
        // We need the Surface configured before we can do anything.
        self.surface.configure(&self.device, &self.config);
        self.is_surface_configured = true;
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            (KeyCode::KeyC, true) => {
                let factor = 1.5;
                self.clear_color.r *= factor;
                self.clear_color.g *= factor;
                self.clear_color.b *= factor;

                log::info!("Clear color is now: {}", self.clear_color.r);
            }
            _ => {}
        }
    }

    fn update_camera(&mut self) {
        self.camera.aspect = (self.config.width as f32) / (self.config.height as f32);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn update(&mut self) {
        self.update_camera();
        let input = self.input.data();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // This is where all the magic happens!
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;

        // This creates the View 'into' the Surface texture.
        // We need this View to control how the render code
        // interacts with the texture.
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // the CommandEncoder is the equivalent of the Command Buffer
        // from other graphics frameworks. The Encoder build a buffer
        // for the commands that will be sent to the GPU.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Create a render pass to clear the screen
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Render Pass"),
            // note that color_attachments is a "sparse" array.
            // This allows us to have multiple render targets but only
            // provide the ones that we care about.
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        // when using indexed vertices, you need to use the draw_indexed
        // method instead of the normal draw() method
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

        // the .begin_render_pass() method mutably borrows `encoder`.
        // We need to drop that reference so that we can call
        // encoder.finish() down below.
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<State>>,
    state: Option<State>,
}

impl Default for App {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        let event_loop: EventLoop<State> = EventLoop::with_user_event().build().unwrap();
        App::new(
            #[cfg(target_arch = "wasm32")]
            &event_loop,
        )
    }
}

impl App {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<State>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            state: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
        }
    }

    fn handle_redraw(state: &mut State) {
        state.update();
        match state.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                // reconfigure the Surface if it is lost or outdated
                let size = state.window.inner_size();
                state.resize(size.width, size.height);
            }
            Err(e) => {
                log::error!("Unable to render {e}");
            }
        }
        state.input.reset_frame();
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element: web_sys::HtmlCanvasElement = canvas.unchecked_into();

            // Attempt to set Extended tone mapping for the canvas
            // Most of these types are considered Unstable so they need to be compiled
            // with the RUSTFLAGS=--cfg=web_sys_unstable_apis env value
            // let canvas_context: web_sys::GpuCanvasContext = html_canvas_element
            //     .get_context("webgpu")
            //     .unwrap_throw()
            //     .unwrap_throw()
            //     .unchecked_into();
            // let canvas_config = canvas_context.get_configuration().unwrap_throw();
            // let canvas_tone_mapping = web_sys::GpuCanvasToneMapping::new();
            // canvas_tone_mapping.set_mode(web_sys::GpuCanvasToneMappingMode::Extended);
            // canvas_config.set_tone_mapping(&canvas_tone_mapping);

            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        window_attributes.inner_size = Some(Size::Logical(LogicalSize {
            width: 512.0,
            height: 512.0,
        }));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.state = Some(pollster::block_on(State::new(window)).unwrap());
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(
                        // Send an event that will be handled by the user_event
                        // method in ApplicationHandler
                        proxy
                            .send_event(State::new(window).await.expect("Unable to create canvas!"))
                            .is_ok()
                    )
                });
            }
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        #[cfg(target_arch = "wasm32")]
        {
            event.window.request_redraw();
            event.resize(
                event.window.inner_size().width,
                event.window.inner_size().height,
            );
        }
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // early exit if app does not have a State yet
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => Self::handle_redraw(state),
            // Match for the KeyboardInput pattern and extract the
            // code and state variables from the pattern
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            WindowEvent::CursorMoved {
                position: PhysicalPosition { x, y },
                ..
            } => state.input.handle_cursor_moved((x, y)),
            _ => {}
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
    event_loop.run_app(&mut app)?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn web_run() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    // execute the main run() function
    run().unwrap_throw();

    Ok(())
}
