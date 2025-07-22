use cgmath::{Matrix, SquareMatrix};

use crate::input_handling::{ButtonState, InputData};

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        let translation = cgmath::Matrix4::from_translation(self.position);
        let rotation = cgmath::Matrix4::from(self.rotation);
        let scale = cgmath::Matrix4::from_nonuniform_scale(1.0, 1.0, 1.0);

        let rotation_scale = rotation * scale;

        let truncated = cgmath::Matrix3::from_cols(
            rotation_scale.x.truncate(), // Vector4 → Vector3
            rotation_scale.y.truncate(),
            rotation_scale.z.truncate(),
        );

        let normal_matrix = truncated.invert().unwrap().transpose();

        InstanceRaw {
            model: (translation * rotation_scale).into(),
            normal: normal_matrix.into(),
        }
    }
}

/// InstanceRaw is the representation of an instance for the GPU.
/// Instead of storing individual position and rotation fields, it
/// will simply store the full transformation matrix.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    // Remember that we CANNOT use the same model matrix to transform our normals
    // This leads to incorrect normals whenever we have scaling involved. We actually
    // need to transform our normals by the "inverse transpose" of the model matrix.
    normal: [[f32; 3]; 3],
}

impl InstanceRaw {
    const ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
        5 => Float32x4,
        6 => Float32x4,
        7 => Float32x4,
        8 => Float32x4,
        9 => Float32x3,
        10 => Float32x3,
        11 => Float32x3,
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(
            cgmath::Deg(self.fov_y),
            self.aspect,
            self.z_near,
            self.z_far,
        );

        // The constant here is important! The cgmath crate is built using the
        // OpenGL coordinate system, while WGPU is build using the coordinate
        // system of DirectX and Metal! We need to convert from OpenGL to WGPU
        // whenever we use cgmath.
        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn process_input(&mut self, input: &InputData) {
        if let ButtonState::Released(_) = input.mouse_button_left {
            return;
        }

        let sensitivity = 0.001;

        let mut mouse_delta = input.mouse_pos_delta;
        mouse_delta = (mouse_delta.0 * sensitivity, -mouse_delta.1 * sensitivity);

        // Update camera position
        self.target += (mouse_delta.0 as f32, mouse_delta.1 as f32, 0.0).into();
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // bytemuch can't make sense of cgmath matrices, so we need to
    // convert to a simple 4x4 array of f32's
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, cam: &Camera) {
        self.view_proj = cam.build_view_projection_matrix().into();
    }
}
