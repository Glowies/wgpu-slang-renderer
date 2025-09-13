// A big chunk of the algorithm here was implemented with the help of this page: https://learnopengl.com/PBR/IBL/Specular-IBL
use anyhow::Result;
use glam::{Vec2, Vec3};
use image::{DynamicImage, Rgb, Rgb32FImage, imageops::sample_bilinear};
use std::f32::consts::PI;

pub fn equirectangular_to_prefiltered_cubemap(
    source_image: &DynamicImage,
    face_size: u32,
    sample_count: u32,
) -> Result<Vec<Vec<DynamicImage>>> {
    if !face_size.is_power_of_two() {
        anyhow::bail!("Provided face size is not a power of two.");
    }
    let mip_count = face_size.ilog2() + 1;

    let mut result = Vec::with_capacity(mip_count as usize);
    let roughness_gap = 1.0 / (mip_count + 1) as f32;

    let mut face_size = face_size;

    for i in 0..mip_count {
        let roughness = roughness_gap * i as f32;
        let faces = equirectangular_to_prefiltered_cubemap_level(
            source_image,
            face_size,
            roughness,
            sample_count,
        );

        result.push(faces);

        face_size /= 2;
    }

    Ok(result)
}

fn equirectangular_to_prefiltered_cubemap_level(
    source_image: &DynamicImage,
    face_size: u32,
    roughness: f32,
    sample_count: u32,
) -> Vec<DynamicImage> {
    let source_image = source_image.to_rgb32f();

    let mut faces = vec![
        DynamicImage::new_rgb32f(face_size, face_size),
        DynamicImage::new_rgb32f(face_size, face_size),
        DynamicImage::new_rgb32f(face_size, face_size),
        DynamicImage::new_rgb32f(face_size, face_size),
        DynamicImage::new_rgb32f(face_size, face_size),
        DynamicImage::new_rgb32f(face_size, face_size),
    ];

    // Define the six cube faces
    let directions = [
        |x: f32, y: f32| Vec3::new(-x, y, -1.0),
        |x: f32, y: f32| Vec3::new(x, y, 1.0),
        |x: f32, y: f32| Vec3::new(y, -1.0, -x),
        |x: f32, y: f32| Vec3::new(-y, 1.0, -x),
        |x: f32, y: f32| Vec3::new(1.0, y, -x),
        |x: f32, y: f32| Vec3::new(-1.0, y, x),
    ];

    for (face_index, dir_func) in directions.iter().enumerate() {
        let face = faces[face_index].as_mut_rgb32f().unwrap();
        for y in 0..face_size {
            for x in 0..face_size {
                let u_norm = 2.0 * ((x as f32 + 0.5) / face_size as f32) - 1.0;
                let v_norm = 2.0 * ((y as f32 + 0.5) / face_size as f32) - 1.0;

                let dir = dir_func(u_norm, v_norm).normalize();

                let color = prefilter_convolution(dir, roughness, &source_image, sample_count);
                let pixel = Rgb::from(color.to_array());

                face.put_pixel(x, y, pixel);
            }
        }
    }

    faces
}

fn sample_equirect_at_dir(dir: Vec3, equirectangular_map: &Rgb32FImage) -> Vec3 {
    let (u, v) = dir_to_equirect_uv(dir);
    let pixel = sample_bilinear(equirectangular_map, u, v).unwrap();
    Vec3::new(pixel.0[0], pixel.0[1], pixel.0[2])
}

fn dir_to_equirect_uv(dir: Vec3) -> (f32, f32) {
    let theta = dir.z.atan2(dir.x);
    let phi = dir.y.asin();

    let u = (theta + PI) / (2.0 * PI);
    let v = (phi + PI / 2.0) / PI;
    (u, v)
}

fn radical_inverse_vd_c(bits: u32) -> f32 {
    let mut bits = bits;
    bits = (bits << 16u32) | (bits >> 16u32);
    bits = ((bits & 0x55555555u32) << 1u32) | ((bits & 0xAAAAAAAAu32) >> 1u32);
    bits = ((bits & 0x33333333u32) << 2u32) | ((bits & 0xCCCCCCCCu32) >> 2u32);
    bits = ((bits & 0x0F0F0F0Fu32) << 4u32) | ((bits & 0xF0F0F0F0u32) >> 4u32);
    bits = ((bits & 0x00FF00FFu32) << 8u32) | ((bits & 0xFF00FF00u32) >> 8u32);
    return bits as f32 * 2.3283064365386963e-10; // / 0x100000000
}

fn hammersley(i: u32, n: u32) -> Vec2 {
    Vec2::new((i as f32) / (n as f32), radical_inverse_vd_c(i))
}

fn importance_sample_ggx(x_i: Vec2, n: Vec3, roughness: f32) -> Vec3 {
    let a = roughness * roughness;

    let phi = 2.0 * PI * x_i.x;
    let cos_theta = ((1.0 - x_i.y) / (1.0 + (a * a - 1.0) * x_i.y)).sqrt();
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

    // from spherical coordinates to cartesian coordinates
    let h = Vec3::new(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta);

    // from tangent-space vector to world-space sample vector
    let up = if n.z.abs() < 0.999 {
        Vec3::new(0.0, 0.0, 1.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };
    let tangent = up.cross(n).normalize();
    let bitangent = n.cross(tangent);

    let sample_vec = tangent * h.x + bitangent * h.y + n * h.z;
    return sample_vec.normalize();
}

fn prefilter_convolution(
    n: Vec3,
    roughness: f32,
    equirectangular_map: &Rgb32FImage,
    sample_count: u32,
) -> Vec3 {
    let n = n.normalize();
    let r = n;
    let v = r;

    let mut total_weight = 0.0;
    let mut prefiltered_color = Vec3::ZERO;
    for i in 0..sample_count {
        let x_i = hammersley(i, sample_count);
        let h = importance_sample_ggx(x_i, n, roughness);
        let l = (2.0 * v.dot(h) * h - v).normalize();

        let n_dot_l = n.dot(l).max(0.0);
        if n_dot_l > 0.0 {
            prefiltered_color += sample_equirect_at_dir(l, equirectangular_map) * n_dot_l;
            total_weight += n_dot_l;
        }
    }

    prefiltered_color / total_weight
}
