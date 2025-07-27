use cgmath::*;
use std::f32::consts::FRAC_PI_2;
use wgpu::{BindGroup, BindGroupLayout, Buffer, util::DeviceExt};

use crate::{
    input_handling::{ButtonState, InputData},
    wgpu_traits::AsBindGroup,
};

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

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
        OPENGL_TO_WGPU_MATRIX * perspective(self.fov_y, self.aspect, self.z_near, self.z_far)
    }
}

pub struct CameraProperties {
    position: cgmath::Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    pub projection: Projection,
}

impl CameraProperties {
    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        let proj = self.projection.calc_matrix();

        proj * Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }
}

pub struct Camera {
    pub properties: CameraProperties,
    uniform: CameraUniform,

    // AsBindGroup fields
    bind_group_layout: Option<BindGroupLayout>,
    bind_group: Option<BindGroup>,
    uniform_buffer: Option<Buffer>,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
        projection: Projection,
    ) -> Self {
        let properties = CameraProperties {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            projection,
        };

        let mut uniform = CameraUniform::default();
        uniform.update_view_proj(&properties);

        Self {
            properties,
            uniform,
            bind_group_layout: None,
            bind_group: None,
            uniform_buffer: None,
        }
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_pos: [f32; 4],
    // bytemuch can't make sense of cgmath matrices, so we need to
    // convert to a simple 4x4 array of f32's
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn update_view_proj(&mut self, camera: &CameraProperties) {
        self.view_pos = camera.position.to_homogeneous().into();
        self.view_proj = camera.calc_matrix().into();
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_pos: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
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
    fn init_bind_group_layout(&mut self, device: &wgpu::Device) {
        self.bind_group_layout = Some(device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
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
            },
        ));
    }

    fn init_bind_group(&mut self, device: &wgpu::Device) {
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout(),
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
        if let None = self.bind_group_layout {
            panic!("Bind Group Layout for Camera has not been initialized.");
        }

        self.bind_group_layout.as_ref().unwrap()
    }

    fn bind_group(&self) -> &wgpu::BindGroup {
        if let None = self.bind_group {
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
