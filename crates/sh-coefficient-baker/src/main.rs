use clap::Parser;
use postcard::to_io;
use sh_coefficient_baker::{load_cubemap_face, process};
use std::{fs::File, path::PathBuf};

/// CLI tool to convert .exr cubemap faces into spherical harmonics coefficients with given number of bands.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    face_pos_x: PathBuf,
    face_neg_x: PathBuf,
    face_pos_y: PathBuf,
    face_neg_y: PathBuf,
    face_pos_z: PathBuf,
    face_neg_z: PathBuf,

    /// How many bands of SH coefficients should be generated.
    #[arg(long, default_value_t = 3)]
    num_bands: usize,

    /// Apply a clamped cosine convolution after finding SH coefficients, to convert the given cubemap,
    /// into an irradiance representation.
    #[arg(short, long)]
    compute_irradiance: bool,

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

    println!(
        "Computing {} bands of SH coefficients and with{} irradiance convolution.",
        args.num_bands,
        if args.compute_irradiance { "" } else { "out" },
    );

    let sh_coefs = process(args.num_bands, args.compute_irradiance, &faces)
        .expect("Failed to extract SH coefficients from cube map faces.");

    let file = File::create(&args.output_path).expect("Failed to create output file");

    println!("Writing Vec of SH coefficients into {:?}", args.output_path);
    to_io(&sh_coefs, file).expect("Failed to serialize and write results into output file");

    // TO DESERIALIZE:
    // let mut result_file = File::open(&args.output_path).unwrap();
    // let mut result_bytes = Vec::new();
    // result_file.read_to_end(&mut result_bytes).unwrap();
    // let result: [[f32; 3]; 9] = from_bytes(&result_bytes).unwrap();

    // println!("{:#?}", result);
}
