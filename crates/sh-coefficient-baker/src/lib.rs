use glam::Vec3;
use image::{ImageBuffer, ImageReader, Rgb};
use std::f32::consts::PI;
use std::path::PathBuf;

pub fn load_cubemap_face(face_path: PathBuf) -> ImageBuffer<Rgb<f32>, Vec<f32>> {
    ImageReader::open(face_path)
        .expect("Unable to open image file")
        .decode()
        .expect("Unable to decode image file")
        .into_rgb32f()
}

// Source: https://github.com/DGriffin91/cubemap-spherical-harmonics/blob/main/src/lib.rs
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

fn compute_sh_basis(num_bands: usize, s: &Vec3) -> Vec<f64> {
    let mut sh_basis = Vec::new();

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

/// Returns spherical harmonics for input cube map.
/// Input should be 6 square images in the order: +x, -x, +y, -y, +z, -z
pub fn process(faces: &[ImageBuffer<Rgb<f32>, Vec<f32>>]) -> anyhow::Result<[[f32; 3]; 9]> {
    if faces.len() != 6 {
        anyhow::bail!("Expected 6 faces")
    }
    let size = faces[0].width();
    let mut cube_map_vecs = Vec::new();
    let sizef = size as f32;
    let cubemap_face_normals = get_cubemap_face_normals();

    // Forsyth's weights
    let weight1 = 4.0 / 17.0;
    let weight2 = 8.0 / 17.0;
    let weight3 = 15.0 / 17.0;
    let weight4 = 5.0 / 68.0;
    let weight5 = 15.0 / 68.0;

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

    let mut sh = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
    ];
    let mut weight_accum = 0.0;

    for (idx, face) in faces.iter().enumerate() {
        for y in 0..size {
            for x in 0..size {
                let mut color = Vec3::from(face.get_pixel(x, y).0);

                let tex_v = cube_map_vecs[idx][(y * size + x) as usize];

                let weight = solid_angle(x as f32, y as f32, sizef);

                color *= weight;

                sh[0] += color * weight1;

                sh[1] += color * weight2 * tex_v.x;
                sh[2] += color * weight2 * tex_v.y;
                sh[3] += color * weight2 * tex_v.z;

                sh[4] += color * weight3 * tex_v.x * tex_v.z;
                sh[5] += color * weight3 * tex_v.z * tex_v.y;
                sh[6] += color * weight3 * tex_v.y * tex_v.x;
                sh[7] += color * weight4 * (3.0 * tex_v.z * tex_v.z - 1.0);
                sh[8] += color * weight5 * (tex_v.x * tex_v.x - tex_v.y * tex_v.y);

                weight_accum += weight * 3.0;
            }
        }
    }

    let mut result = [[0.0; 3]; 9];
    for (idx, n) in sh.iter_mut().enumerate() {
        *n *= 4.0 * PI / weight_accum;
        result[idx] = [n.x, n.y, n.z];
    }

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
