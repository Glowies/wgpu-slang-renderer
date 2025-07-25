use cgmath::*;
use std::f32::consts::FRAC_PI_2;

use crate::input_handling::{ButtonState, InputData};

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

pub struct Camera {
    position: cgmath::Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    pub projection: Projection,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
        projection: Projection,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            projection,
        }
    }

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
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_pos: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_pos = camera.position.to_homogeneous().into();
        self.view_proj = camera.calc_matrix().into();
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

    pub fn update_camera(&self, camera: &mut Camera) {
        camera.yaw = self.yaw;
        camera.pitch = self.pitch;

        let (yaw_sin, yaw_cos) = self.yaw.0.sin_cos();
        let (pitch_sin, pitch_cos) = self.pitch.0.sin_cos();
        let offset = Vector3::new(yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos).normalize()
            * -self.orbit_radius;

        camera.position = self.target + offset;
    }
}
