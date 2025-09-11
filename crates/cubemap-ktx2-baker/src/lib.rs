use glam::Vec3;
use image::DynamicImage;
use std::f32::consts::PI;

pub fn equirectangular_to_cubemap(
    source_image: &DynamicImage,
    face_size: u32,
) -> Vec<DynamicImage> {
    let source_image = source_image.as_rgb32f().unwrap();

    let mut faces = vec![
        DynamicImage::new_rgb32f(face_size, face_size),
        DynamicImage::new_rgb32f(face_size, face_size),
        DynamicImage::new_rgb32f(face_size, face_size),
        DynamicImage::new_rgb32f(face_size, face_size),
        DynamicImage::new_rgb32f(face_size, face_size),
        DynamicImage::new_rgb32f(face_size, face_size),
    ];

    let width = source_image.width();
    let height = source_image.height();

    // Define the six cube faces
    let directions = [
        // Negative Z (Back)
        |x: f32, y: f32| Vec3::new(x, y, -1.0),
        // Positive Z (Front)
        |x: f32, y: f32| Vec3::new(-x, y, 1.0),
        // Negative X (Left)
        |x: f32, y: f32| Vec3::new(-y, -1.0, -x),
        // Positive X (Right)
        |x: f32, y: f32| Vec3::new(-y, 1.0, -x),
        // Positive Y (Top)
        |x: f32, y: f32| Vec3::new(1.0, y, x),
        // Negative Y (Bottom)
        |x: f32, y: f32| Vec3::new(-1.0, y, -x),
    ];

    for (face_index, dir_func) in directions.iter().enumerate() {
        let face = faces[face_index].as_mut_rgb32f().unwrap();
        for y in 0..face_size {
            for x in 0..face_size {
                let u_norm = 2.0 * ((x as f32 + 0.5) / face_size as f32) - 1.0;
                let v_norm = 2.0 * ((y as f32 + 0.5) / face_size as f32) - 1.0;

                let dir = dir_func(u_norm, v_norm).normalize();

                let theta = dir.z.atan2(dir.x);
                let phi = dir.y.asin();

                let u = (theta + PI) / (2.0 * PI);
                let v = (phi + PI / 2.0) / PI;

                let source_x = (u * (width as f32 - 1.0)).round() as u32;
                let source_y = (v * (height as f32 - 1.0)).round() as u32;

                let pixel = source_image.get_pixel(source_x, source_y);
                face.put_pixel(x, y, *pixel);
            }
        }
    }

    faces
}
