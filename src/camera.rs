use cgmath::*;
use std::f32::consts::FRAC_PI_2;
use wgpu::{BindGroup, BindGroupLayout, Buffer, util::DeviceExt};

use crate::{
    input_handling::{ButtonState, InputData},
    wgpu_traits::AsBindGroup,
};

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub struct Projection {
    pub aspect: f32,
    pub fov_y: Rad<f32>,
    pub z_near: f32,
    pub z_far: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(
        width: u32,
        height: u32,
        fov_y: F,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fov_y: fov_y.into(),
            z_near,
            z_far,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        perspective(self.fov_y, self.aspect, self.z_near, self.z_far)
    }
}

pub struct CameraProperties {
    pub position: cgmath::Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub projection: Projection,
}

impl CameraProperties {
    /// Calculate the View Matrix (this is without the projection)
    pub fn calc_view_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }

    /// Calculate the Projection Matrix for perspective.
    pub fn calc_proj_matrix(&self) -> Matrix4<f32> {
        self.projection.calc_matrix()
    }
}

pub struct Camera {
    pub properties: CameraProperties,
    uniform: CameraUniform,

    // AsBindGroup fields
    bind_group_layout: BindGroupLayout,
    bind_group: Option<BindGroup>,
    uniform_buffer: Option<Buffer>,
}

impl Camera {
    pub fn new(properties: CameraProperties, device: &wgpu::Device) -> Self {
        let mut uniform = CameraUniform::default();
        uniform.update_view_proj(&properties);

        let bind_group_layout = Self::create_bind_group_layout(device, "Camera Bind Group Layout");
        let mut camera = Self {
            properties,
            uniform,
            bind_group_layout,
            bind_group: None,
            uniform_buffer: None,
        };

        camera.init_all(device);

        camera
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_pos: [f32; 4],
    view: [[f32; 4]; 4],
    view_proj: [[f32; 4]; 4],
    inv_proj: [[f32; 4]; 4],
    inv_view: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn update_view_proj(&mut self, camera: &CameraProperties) {
        self.view_pos = camera.position.to_homogeneous().into();
        let proj = camera.calc_proj_matrix();
        let view = camera.calc_view_matrix();
        let view_proj = proj * view;

        self.view = view.into();
        self.view_proj = view_proj.into();

        // View Matrix is always Orthonormal (each column is perpendicular
        // to each other). So the transpose is guaranteed to be the inverse
        self.inv_view = view.transpose().into();
        // Things aren't so easy for the Projection Matrix though :D
        self.inv_proj = proj.invert().unwrap().into();
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_pos: [0.0; 4],
            view: cgmath::Matrix4::identity().into(),
            view_proj: cgmath::Matrix4::identity().into(),
            inv_proj: cgmath::Matrix4::identity().into(),
            inv_view: cgmath::Matrix4::identity().into(),
        }
    }
}

pub struct OrbitCameraController {
    target: cgmath::Point3<f32>,
    orbit_sensitivity: f32,
    zoom_sensitivity: f32,
    orbit_radius: f32,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

impl OrbitCameraController {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        target: V,
        orbit_sensitivity: f32,
        zoom_sensitivity: f32,
        orbit_radius: f32,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            target: target.into(),
            orbit_sensitivity,
            zoom_sensitivity,
            orbit_radius,
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn process_input(&mut self, input: &InputData) {
        // Process Zoom Controls
        let zoom_sensitivity = self.zoom_sensitivity;

        self.orbit_radius += zoom_sensitivity * input.mouse_wheel_delta;

        let radius_min = 0.1;
        if self.orbit_radius < radius_min {
            self.orbit_radius = radius_min;
        }

        // Process Orbit Controls
        if let ButtonState::Released(_) = input.mouse_button_left {
            return;
        }

        let orbit_sensitivity = self.orbit_sensitivity;

        let mut mouse_delta = (
            input.mouse_pos_delta.0 as f32,
            input.mouse_pos_delta.1 as f32,
        );
        mouse_delta = (
            mouse_delta.0 * orbit_sensitivity,
            -mouse_delta.1 * orbit_sensitivity,
        );

        self.yaw.0 += mouse_delta.0;
        self.pitch.0 += mouse_delta.1;

        if self.pitch < -Rad(SAFE_FRAC_PI_2) {
            self.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if self.pitch > Rad(SAFE_FRAC_PI_2) {
            self.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }

    pub fn update_camera(&self, camera: &mut CameraProperties) {
        camera.yaw = self.yaw;
        camera.pitch = self.pitch;

        let (yaw_sin, yaw_cos) = self.yaw.0.sin_cos();
        let (pitch_sin, pitch_cos) = self.pitch.0.sin_cos();
        let offset = Vector3::new(yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos).normalize()
            * -self.orbit_radius;

        camera.position = self.target + offset;
    }
}

impl AsBindGroup for Camera {
    fn bind_group_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![wgpu::BindGroupLayoutEntry {
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
        }]
    }

    fn init_bind_group(&mut self, device: &wgpu::Device) {
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: self.bind_group_layout(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),
            }],
            label: Some("Camera Bind Group"),
        }));
    }

    fn init_binding_resources(&mut self, device: &wgpu::Device) {
        self.uniform_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[self.uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        );
    }

    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    fn bind_group(&self) -> &wgpu::BindGroup {
        if self.bind_group.is_none() {
            panic!("Bind Group for Camera has not been initialized.");
        }

        self.bind_group.as_ref().unwrap()
    }

    fn update_binding_resources(&mut self) {
        self.uniform.update_view_proj(&self.properties);
    }

    fn queue_write_binding_resources(&mut self, queue: &wgpu::Queue) {
        self.update_binding_resources();

        queue.write_buffer(
            self.uniform_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }
}
