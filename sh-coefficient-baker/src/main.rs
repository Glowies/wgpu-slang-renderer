use clap::Parser;
use postcard::to_io;
use sh_coefficient_baker::{load_cubemap_face, process};
use std::{fs::File, path::PathBuf};

/// CLI tool to convert .exr cubemap faces into 9 spherical harmonics coefficients.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    face_pos_x: PathBuf,
    face_neg_x: PathBuf,
    face_pos_y: PathBuf,
    face_neg_y: PathBuf,
    face_pos_z: PathBuf,
    face_neg_z: PathBuf,

    output_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    let image_pos_x = load_cubemap_face(args.face_pos_x);
    let image_neg_x = load_cubemap_face(args.face_neg_x);
    let image_pos_y = load_cubemap_face(args.face_pos_y);
    let image_neg_y = load_cubemap_face(args.face_neg_y);
    let image_pos_z = load_cubemap_face(args.face_pos_z);
    let image_neg_z = load_cubemap_face(args.face_neg_z);

    let faces = [
        image_pos_x,
        image_neg_x,
        image_pos_y,
        image_neg_y,
        image_pos_z,
        image_neg_z,
    ];

    let sh_coefs = process(&faces).expect("Failed to extract SH coefficients from cube map faces.");

    let file = File::create(&args.output_path).expect("Failed to create output file");

    to_io(&sh_coefs, file).expect("Failed to serialize and write results into output file");

    // TO DESERIALIZE:
    // let mut result_file = File::open(&args.output_path).unwrap();
    // let mut result_bytes = Vec::new();
    // result_file.read_to_end(&mut result_bytes).unwrap();
    // let result: [[f32; 3]; 9] = from_bytes(&result_bytes).unwrap();

    // println!("{:#?}", result);
}
