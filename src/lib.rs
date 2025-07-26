mod camera;
mod input_handling;
mod instance;
mod light;
mod model;
mod resources;
mod texture;
mod wgpu_traits;

use camera::{Camera, CameraUniform, OrbitCameraController, Projection};
use cgmath::{Deg, prelude::*};
use input_handling::Input;
use instance::{Instance, InstanceRaw};
use light::{DrawLight, Light, LightProperties};
use model::{DrawModel, Model, Vertex};
use std::{cmp, sync::Arc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;
use wgpu_traits::WgpuUniform;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

pub struct State {
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_controller: OrbitCameraController,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    clear_color: wgpu::Color,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    is_surface_configured: bool,
    queue: wgpu::Queue,
    lit_render_pipeline: wgpu::RenderPipeline,
    light_debug_render_pipeline: wgpu::RenderPipeline,
    surface: wgpu::Surface<'static>,
    window: Arc<Window>,
    input: Input,
    depth_texture: texture::Texture,
    obj_model: Model,
    light: Light,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        // The Instance is used to create the Surfaces and the Adapters
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::GL,
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
                ],
                label: Some("Texture Bind Group Layout"),
            });

        // We initialize the Depth Buffer here but it will get recreated everytime
        // the window is resized. The dimensions of the Depth Buffer has to
        // match the dimensions of the render.
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &surface_config, "Depth Texture");

        let projection = Projection::new(
            surface_config.width,
            surface_config.height,
            Deg(45.0),
            0.1,
            100.0,
        );

        let camera_controller =
            OrbitCameraController::new((0.0, 0.0, 0.0), 0.001, 0.01, 4.0, Deg(0.0), Deg(-32.0));
        let camera = Camera::new((0.0, 0.0, 0.0), Deg(0.0), Deg(0.0), projection);

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
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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

        let mut light = Light::new(LightProperties {
            position: [2.0, 2.0, 2.0].into(),
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
        });
        light.init_uniform_bind_group(&device);

        let lit_render_pipeline = {
            // create our Shader Module using the .wgsl file
            let shader_module_desc = wgpu::ShaderModuleDescriptor {
                label: Some("Lit Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/lit.wgsl").into()),
            };

            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &texture_bind_group_layout,
                        &camera_bind_group_layout,
                        light.bind_group_layout(),
                    ],
                    push_constant_ranges: &[],
                });

            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                surface_config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), InstanceRaw::desc()],
                shader_module_desc,
            )
        };

        let light_debug_render_pipeline = {
            let shader_module_desc = wgpu::ShaderModuleDescriptor {
                label: Some("Light Debug Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/light-debug.wgsl").into()),
            };

            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, light.bind_group_layout()],
                push_constant_ranges: &[],
            });

            create_render_pipeline(
                &device,
                &layout,
                surface_config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader_module_desc,
            )
        };

        let obj_model =
            resources::load_model("gem.obj", &queue, &device, &texture_bind_group_layout)
                .await
                .unwrap();

        const NUM_INSTANCES_PER_ROW: u32 = 16;
        const SPACE_BETWEEN: f32 = 3.0;

        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = cgmath::Vector3 { x, y: 0.0, z };

                    let rotation_factor = 10.0;
                    let rotation =
                        cgmath::Quaternion::from_angle_x(cgmath::Deg(x * rotation_factor))
                            * cgmath::Quaternion::from_angle_z(cgmath::Deg(z * rotation_factor));

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instances Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
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
            lit_render_pipeline,
            light_debug_render_pipeline,
            camera,
            camera_uniform,
            camera_bind_group,
            camera_buffer,
            camera_controller,
            input: Input::new(),
            instances,
            instance_buffer,
            depth_texture,
            obj_model,
            light,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width < 1 || height < 1 {
            return;
        }

        let max_size = self.device.limits().max_texture_dimension_2d;

        self.config.width = cmp::min(width, max_size);
        self.config.height = cmp::min(height, max_size);

        // We need to recreate the Depth Buffer everytime the window is resized
        // because the dimensions of it need to match the render dimensions
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.config, "Depth Texture");

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
        self.camera_controller.update_camera(&mut self.camera);
        self.camera
            .projection
            .resize(self.config.width, self.config.height);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    fn update_light(&mut self) {
        let position = self.light.properties.position;
        let transform =
            cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0));

        // rotate around y-axis by one degree
        self.light.properties.position = transform * position;

        // update the light buffer
        self.light.queue_write_buffer(&self.queue);
    }

    pub fn update(&mut self) {
        let input = self.input.data();

        self.camera_controller.process_input(input);
        self.update_camera();
        self.update_light();
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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        render_pass.set_pipeline(&self.lit_render_pipeline);
        render_pass.draw_model_instanced(
            &self.obj_model,
            0..self.instances.len() as u32,
            &self.camera_bind_group,
            self.light.bind_group(),
        );

        render_pass.set_pipeline(&self.light_debug_render_pipeline);
        render_pass.draw_light_model(
            &self.obj_model,
            &self.camera_bind_group,
            self.light.bind_group(),
        );

        // the .begin_render_pass() method mutably borrows `encoder`.
        // We need to drop that reference so that we can call
        // encoder.finish() down below.
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: vertex_layouts,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
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

            let parent_element = canvas.parent_element().unwrap();
            let _parent_bounds = parent_element.get_bounding_client_rect();

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

            let html_canvas_element: web_sys::HtmlCanvasElement = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

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

            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => state.input.handle_mouse_input(button, button_state),

            WindowEvent::MouseWheel { delta, .. } => state.input.handle_mouse_wheel(delta),

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
