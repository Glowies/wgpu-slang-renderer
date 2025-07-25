use cgmath::{Matrix, SquareMatrix};

use crate::input_handling::{ButtonState, InputData};

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
