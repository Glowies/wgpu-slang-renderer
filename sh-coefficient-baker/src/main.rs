use std::path::PathBuf;

use clap::Parser;
use sh_coefficient_baker::{load_cubemap_face, process};

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

    let sh_coefs = process(&faces);

    println!("{:#?}", sh_coefs);
}
