use std::{
    fs::{create_dir_all, remove_dir_all},
    path::PathBuf,
    process::Command,
};

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

    /// Number of samples to take when computing each pixel of the cube map.
    #[arg(short, long, default_value_t = 1024)]
    sample_count: u32,

    output_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    if args.output_path.is_dir() {
        panic!("Given output path is a directory: {:#?}", args.output_path);
    }

    let source_path = args.input_path;
    let face_size = args.resolution;

    let temp_dir = std::env::current_dir().unwrap().join("temp/");
    let tiled_dir = temp_dir.join("tiled/");
    let scanline_dir = temp_dir.join("scanline/");
    create_dir_all(&tiled_dir).unwrap();
    create_dir_all(&scanline_dir).unwrap();

    let source_image = image::open(source_path).expect("Failed to open input file");

    let mips = equirectangular_to_prefiltered_cubemap(&source_image, face_size, args.sample_count)
        .unwrap();

    let face_names = ["+x.exr", "-x.exr", "+y.exr", "-y.exr", "+z.exr", "-z.exr"];
    let mut oiio_args: Vec<String> = Vec::with_capacity(6 * 4 * mips.len());
    let mut ktx_args: Vec<String> = Vec::with_capacity(6 * mips.len() + 13);
    ktx_args.extend(vec![
        "create".to_string(),
        "--format".to_string(),
        "R16G16B16A16_SFLOAT".to_string(),
        "--zstd".to_string(),
        "20".to_string(),
        "--assign-primaries".to_string(),
        "srgb".to_string(),
        "--assign-tf".to_string(),
        "linear".to_string(),
        "--cubemap".to_string(),
        "--levels".to_string(),
        mips.len().to_string(),
    ]);

    println!("Saving tiled EXR results.");
    for (mip_idx, faces) in mips.iter().enumerate() {
        for (i, face) in faces.iter().enumerate() {
            let file_name = format!("{}mip{}", mip_idx, face_names[i]);
            let tiled_path = tiled_dir.join(&file_name);
            let scanline_path = (&scanline_dir).join(&file_name);

            face.save(&tiled_path)
                .expect("Failed to save cube map face");

            let args_for_oiio = vec![
                tiled_path.to_str().unwrap().to_string(),
                "--scanline".to_string(),
                "-o".to_string(),
                scanline_path.to_str().unwrap().to_string(),
            ];
            oiio_args.extend(args_for_oiio);

            ktx_args.push(scanline_path.to_str().unwrap().to_string());
        }
    }

    println!("Converting tiled EXR files to scanline with command.");
    let oiio_output = Command::new("oiiotool").args(oiio_args).output().unwrap();
    if !oiio_output.status.success() {
        eprintln!(
            "ERROR: Failed to convert tiled EXRs to scanline EXRs: \n{}",
            String::from_utf8(oiio_output.stderr).unwrap()
        );
    }

    println!("Compiling EXR files into a ktx2 cube texture.");
    ktx_args.push(args.output_path.into_os_string().into_string().unwrap());
    let ktx_output = Command::new("ktx").args(ktx_args).output().unwrap();
    if !ktx_output.status.success() {
        eprintln!(
            "ERROR: Failed to bake cube texture as ktx2: \n{}",
            String::from_utf8(ktx_output.stderr).unwrap()
        );
    }

    remove_dir_all(temp_dir).unwrap();
}
