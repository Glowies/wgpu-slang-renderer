use glam::Vec3;
use image::{ImageBuffer, ImageReader, Rgb};
use std::f32::consts::PI;
use std::f64::consts::PI as M_PI;
use std::ops::MulAssign;
use std::path::PathBuf;

// Sources:
// - https://github.com/DGriffin91/cubemap-spherical-harmonics/blob/main/src/lib.rs
// - https://www.ppsloan.org/publications/StupidSH36.pdf
// - https://google.github.io/filament/main/filament.html#sphericalharmonics

pub fn load_cubemap_face(face_path: PathBuf) -> ImageBuffer<Rgb<f32>, Vec<f32>> {
    ImageReader::open(face_path)
        .expect("Unable to open image file")
        .decode()
        .expect("Unable to decode image file")
        .into_rgb32f()
}

fn get_cubemap_face_normals() -> [[Vec3; 3]; 6] {
    [
        [
            // +x
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
        ],
        [
            // -x
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(-1.0, 0.0, 0.0),
        ],
        [
            // +y
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
        ],
        [
            // -y
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, -1.0, 0.0),
        ],
        [
            // +z
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ],
        [
            // -z
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
        ],
    ]
}

fn sh_index(m: isize, l: usize) -> usize {
    let l = l as isize;
    return (l * (l + 1) + m) as usize;
}

fn normalization_factor(m: isize, l: usize) -> f64 {
    let l_float = l as f64;
    let m_abs = m.abs() as usize;

    let left = (2.0 * l_float + 1.0) / (4.0 / M_PI);
    let right = factorial_division(l - m_abs, l + m_abs);

    (left * right).sqrt()
}

fn apply_normalization<T>(num_bands: usize, sh: &mut Vec<T>, apply_twice: bool)
where
    T: MulAssign<f32>,
{
    let (factor, power) = if apply_twice {
        ((2.0 as f64), 2)
    } else {
        ((2.0 as f64).sqrt(), 1)
    };

    // m==0 case
    for l in 0..num_bands {
        let m = 0;
        sh[sh_index(m, l)] *= normalization_factor(m, l).powi(power) as f32;
    }

    // m=/=0 case
    for l in 1..num_bands {
        for m in 1..(l + 1) {
            let pos_m = m as isize;
            let neg_m = -pos_m;
            sh[sh_index(neg_m, l)] *= (factor * normalization_factor(neg_m, l).powi(power)) as f32;
            sh[sh_index(pos_m, l)] *= (factor * normalization_factor(pos_m, l).powi(power)) as f32;
        }
    }
}

/// Computes the *non-normalized* SH basis. If we want the normalized SH basis
/// we need to multiply this result by sqrt(2) * K^m_l
fn compute_sh_basis(num_bands: usize, s: &Vec3) -> Vec<f64> {
    let mut sh_basis = vec![0.0; num_bands * num_bands];

    let s = s.normalize();
    let s_x: f64 = s.x.into();
    let s_y: f64 = s.y.into();
    let s_z: f64 = s.z.into();

    // handle m=0 separately, since it produces only one coefficient
    let mut pml_2: f64 = 0.0;
    let mut pml_1: f64 = 1.0;

    sh_basis[0] = pml_1;

    for l in 1..num_bands {
        let l_float: f64 = l as f64;
        let pml: f64 = ((2.0 * l_float - 1.0) * pml_1 * s_z - (l_float - 1.0) * pml_2) / l_float;
        pml_2 = pml_1;
        pml_1 = pml;
        sh_basis[sh_index(0, l)] = pml;
    }

    // now handle m=/=0
    let mut pmm: f64 = 1.0;
    for m in 1..num_bands {
        let m_float: f64 = m as f64;
        pmm = (1.0 - 2.0 * m_float) * pmm;
        let mut pml_2: f64 = pmm;
        let mut pml_1: f64 = (2.0 * m_float + 1.0) * pmm * s_z;

        let pos_m = m as isize;
        let neg_m = -pos_m;

        // l == m
        sh_basis[sh_index(neg_m, m)] = pml_2;
        sh_basis[sh_index(pos_m, m)] = pml_2;
        if m + 1 < num_bands {
            // l == m+1
            sh_basis[sh_index(neg_m, m + 1)] = pml_1;
            sh_basis[sh_index(pos_m, m + 1)] = pml_1;
            for l in (m + 2)..num_bands {
                let l_float: f64 = l as f64;
                let pml: f64 = ((2.0 * l_float - 1.0) * pml_1 * s_z
                    - (l_float + m_float - 1.0) * pml_2)
                    / (l_float - m_float);
                pml_2 = pml_1;
                pml_1 = pml;
                sh_basis[sh_index(neg_m, l)] = pml;
                sh_basis[sh_index(pos_m, l)] = pml;
            }
        }
    }

    let mut cm = s_x;
    let mut sm = s_y;

    for m in 1..(num_bands + 1) {
        for l in m..num_bands {
            let pos_m = m as isize;
            let neg_m = -pos_m;

            sh_basis[sh_index(neg_m, l)] *= sm;
            sh_basis[sh_index(pos_m, l)] *= cm;
        }
        let cm1: f64 = cm * s_x - sm * s_y;
        let sm1: f64 = sm * s_x + cm * s_y;
        cm = cm1;
        sm = sm1;
    }

    sh_basis
}

/// Returns the clamped < cos(theta) > SH coefficient for band l pre-multiplied by 1 / K(0,l).
/// This premultiplication is convenient as it cancels out the sqrt term that will be multiplied
/// during the convolution.
fn compute_truncated_cos_sh(l: usize) -> f64 {
    if l == 0 {
        return M_PI;
    } else if l == 1 {
        return 2.0 * M_PI / 3.0;
    } else if l % 2 == 1 {
        return 0.0;
    }
    let l_2 = l / 2;
    let a0: f64 = (if l_2 % 2 == 1 { 1.0 } else { -1.0 }) / (((l + 2) * (l - 1)) as f64);
    let a1: f64 = factorial_division(l, l_2) / (factorial_division(l_2, 1) * (1 << l) as f64);
    return 2.0 * M_PI * a0 * a1;
}

/// returns n! / d!
fn factorial_division(n: usize, d: usize) -> f64 {
    let mut d = d.max(1);
    let mut n = n.max(1);

    let mut r: f64 = 1.0;

    if n == d {
        // intentionally left blank
    } else if n > d {
        while n > d {
            r *= n as f64;
            n -= 1;
        }
    } else {
        while d > n {
            r *= d as f64;
            d -= 1;
        }
        r = 1.0 / r;
    }
    return r;
}

/// Returns spherical harmonics for input cube map.
/// Input should be 6 square images in the order: +x, -x, +y, -y, +z, -z
pub fn process(
    num_bands: usize,
    faces: &[ImageBuffer<Rgb<f32>, Vec<f32>>],
) -> anyhow::Result<[[f32; 3]; 9]> {
    if faces.len() != 6 {
        anyhow::bail!("Expected 6 faces")
    }
    let size = faces[0].width();
    let mut cube_map_vecs = Vec::new();
    let sizef = size as f32;
    let cubemap_face_normals = get_cubemap_face_normals();

    for (idx, face) in faces.iter().enumerate() {
        if face.width() != face.height() {
            anyhow::bail!("Expected face width and height to match")
        }
        let mut face_vecs = Vec::new();
        for v in 0..size {
            for u in 0..size {
                let fu = (2.0 * u as f32 / (sizef - 1.0)) - 1.0;
                let fv = (2.0 * v as f32 / (sizef - 1.0)) - 1.0;

                let vec_x = cubemap_face_normals[idx][0] * fu;
                let vec_y = cubemap_face_normals[idx][1] * fv;
                let vec_z = cubemap_face_normals[idx][2];

                face_vecs.push((vec_x + vec_y + vec_z).normalize())
            }
        }
        cube_map_vecs.push(face_vecs)
    }

    let mut sh = vec![Vec3::ZERO; num_bands * num_bands];
    let mut weight_accum = 0.0;

    for (face_idx, face) in faces.iter().enumerate() {
        for y in 0..size {
            for x in 0..size {
                let mut color = Vec3::from(face.get_pixel(x, y).0);
                let tex_v = cube_map_vecs[face_idx][(y * size + x) as usize];
                let weight = solid_angle(x as f32, y as f32, sizef);
                let sh_basis = compute_sh_basis(num_bands, &tex_v);

                color *= weight;

                for (sh_idx, coeff) in sh.iter_mut().enumerate() {
                    *coeff += color * sh_basis[sh_idx] as f32;
                }

                weight_accum += weight * 3.0;
            }
        }
    }

    let mut result = [[0.0; 3]; 9];
    for (idx, n) in sh.iter_mut().enumerate() {
        *n *= 4.0 * PI / weight_accum;
        result[idx] = [n.x, n.y, n.z];
    }

    apply_normalization(num_bands, &mut sh, true);

    Ok(result)
}

// Explanation: https://www.rorydriscoll.com/2012/01/15/cubemap-texel-solid-angle/
fn solid_angle(au: f32, av: f32, size: f32) -> f32 {
    //scale up to [-1, 1] range (inclusive), offset by 0.5 to point to texel center.
    let u = (2.0 * (au + 0.5) / size) - 1.0;
    let v = (2.0 * (av + 0.5) / size) - 1.0;

    let inv_size = 1.0 / size;

    // U and V are the -1..1 texture coordinate on the current face.
    // get projected area for this texel
    let x0 = u - inv_size;
    let y0 = v - inv_size;
    let x1 = u + inv_size;
    let y1 = v + inv_size;
    let angle =
        area_element(x0, y0) - area_element(x0, y1) - area_element(x1, y0) + area_element(x1, y1);

    return angle;
}

fn area_element(x: f32, y: f32) -> f32 {
    (x * y).atan2((x * x + y * y + 1.0).sqrt())
}
