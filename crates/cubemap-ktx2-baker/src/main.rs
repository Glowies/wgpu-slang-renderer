use std::path::PathBuf;

use clap::Parser;
use cubemap_ktx2_baker::equirectangular_to_cubemap;

/// CLI tool to convert equirectangular cubemap image to ktx2 cube textures
/// with specular convolutions stored in its mips.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    input_path: PathBuf,

    /// Edge resolution of the output ktx2 cube texture.
    #[arg(long, default_value_t = 256)]
    num_bands: u32,

    output_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    let source_path = args.input_path;
    let face_size = args.num_bands;

    let source_image = image::open(source_path)
        .expect("Failed to open input file")
        .to_rgb32f();

    let faces = equirectangular_to_cubemap(&source_image, face_size);

    let face_names = ["+x.exr", "-x.exr", "+y.exr", "-y.exr", "+z.exr", "-z.exr"];

    for (i, face) in faces.iter().enumerate() {
        let output_path = args.output_path.join(face_names[i]);
        face.save(&output_path)
            .expect("Failed to save cube map face");
        println!("Saved {}", output_path.display());
    }
}
