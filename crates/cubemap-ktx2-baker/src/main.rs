use std::path::PathBuf;

use clap::Parser;
use cubemap_ktx2_baker::equirectangular_to_prefiltered_cubemap;

/// CLI tool to convert equirectangular cubemap image to ktx2 cube textures
/// with specular convolutions stored in its mips.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    input_path: PathBuf,

    /// Edge resolution of the output ktx2 cube texture.
    #[arg(short, long, default_value_t = 256)]
    resolution: u32,

    output_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    let source_path = args.input_path;
    let face_size = args.resolution;

    let source_image = image::open(source_path).expect("Failed to open input file");

    let mips = equirectangular_to_prefiltered_cubemap(&source_image, face_size).unwrap();

    let face_names = ["+x.exr", "-x.exr", "+y.exr", "-y.exr", "+z.exr", "-z.exr"];

    for (mip_idx, faces) in mips.iter().enumerate() {
        for (i, face) in faces.iter().enumerate() {
            let file_name = format!("{}mip{}", mip_idx, face_names[i]);
            let output_path = args.output_path.join(file_name);
            face.save(&output_path)
                .expect("Failed to save cube map face");
            println!("Saved {}", output_path.display());
        }
    }
}
