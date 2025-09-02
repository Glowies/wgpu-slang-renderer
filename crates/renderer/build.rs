use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

const RESOURCE_PATH: &str = "res";
const SHADER_PATH: &str = "shaders";

fn main() -> Result<()> {
    // This tells Cargo to rerun this script if something in /res/ or /shaders/ changes.
    println!("cargo::rerun-if-changed={}/*", RESOURCE_PATH);
    println!("cargo::rerun-if-changed={}/*", SHADER_PATH);

    copy_resources()?;
    compile_slang_shaders()?;

    Ok(())
}

fn compile_slang_shaders() -> Result<()> {
    let err_msg = "Failed to run slangc. Make sure that shader-slang is installed and that `slangc` is included in your PATH.";
    let slang_status = Command::new("slangc")
        .args(["-v"])
        .status()
        .map_err(|_| Error::msg(err_msg))?;

    if !slang_status.success() {
        println!("cargo::error={err_msg}");
        bail!(err_msg);
    }

    // Init PathBuf that will be used to construct the shader output path
    let out_dir = env::var("OUT_DIR")?;

    let dir_entries = fs::read_dir(SHADER_PATH)?;

    for entry in dir_entries {
        // Only consider files the slang extension
        let path = entry?.path();
        if !(path.is_file() && path.extension().and_then(OsStr::to_str) == Some("slang")) {
            continue;
        }

        let mut out_path = PathBuf::from(&out_dir);
        out_path.push(path.clone());
        out_path.set_extension("wgsl");

        // make sure the parent directory exists
        fs::create_dir_all(out_path.parent().unwrap())?;

        let in_path_str = path.to_str().ok_or(Error::msg(
            "Failed to convert slang shader file path to Rust str",
        ))?;
        let out_path_str = out_path.to_str().ok_or(Error::msg(
            "Failed to convert shader output file path to Rust str",
        ))?;

        println!("Compiling slang shader {:?} to {:?}", path, out_path);

        let args = [in_path_str, "-target", "wgsl", "-o", out_path_str];

        let compilation_output = Command::new("slangc").args(args).output()?;

        if !compilation_output.status.success() {
            let err_header = format!("Failed to compile slang shader: {in_path_str}");
            let err_msg = format!("{}", String::from_utf8(compilation_output.stderr).unwrap());

            println!("cargo::error={err_header}");
            bail!("{err_header}\n{err_msg}");
        }
    }

    Ok(())
}

fn copy_resources() -> Result<()> {
    let out_dir = env::var("OUT_DIR")?;
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let paths_to_copy = vec!["res/"];
    copy_items(&paths_to_copy, out_dir, &copy_options)?;

    Ok(())
}
