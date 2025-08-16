use std::io::Read;

use anyhow::Error;
use ktx2::{Format, SupercompressionScheme};
use wgpu::TextureFormat;

pub fn ktx_to_wgpu_format(format: Option<Format>) -> anyhow::Result<TextureFormat> {
    match format {
        Some(Format::E5B9G9R9_UFLOAT_PACK32) => Ok(TextureFormat::Rgb9e5Ufloat),
        Some(Format::R16G16B16A16_SFLOAT) => Ok(TextureFormat::Rgba16Float),
        Some(Format::R8G8B8A8_SRGB) => Ok(TextureFormat::Rgba8UnormSrgb),
        Some(Format::R8G8B8A8_UNORM) => Ok(TextureFormat::Rgba8Unorm),
        Some(Format::R8G8_UNORM) => Ok(TextureFormat::Rg8Unorm),
        _ => Err(Error::msg(format!(
            "Unsupported KTX2 format: {format:?}. Cannot convert it to a wgpu texture format."
        ))),
    }
}

pub fn size_and_dims_from_header(
    header: ktx2::Header,
) -> (
    wgpu::Extent3d,
    wgpu::TextureDimension,
    wgpu::TextureViewDimension,
) {
    let is_cube = header.face_count == 6;

    let layer_count = if is_cube {
        header.face_count
    } else {
        header.pixel_depth.max(1)
    };

    let size = wgpu::Extent3d {
        width: header.pixel_width,
        height: header.pixel_height.max(1),
        depth_or_array_layers: layer_count,
    };

    let (dimension, view_dimension) = if header.pixel_height == 0 {
        (wgpu::TextureDimension::D1, wgpu::TextureViewDimension::D1)
    } else if is_cube {
        (wgpu::TextureDimension::D2, wgpu::TextureViewDimension::Cube)
    } else if header.pixel_depth == 0 {
        (wgpu::TextureDimension::D2, wgpu::TextureViewDimension::D2)
    } else {
        (wgpu::TextureDimension::D3, wgpu::TextureViewDimension::D3)
    };
    (size, dimension, view_dimension)
}

pub fn get_raw_level_data(reader: &ktx2::Reader<&[u8]>) -> anyhow::Result<Vec<Vec<u8>>> {
    let header = reader.header();
    let mut levels = Vec::new();
    if let Some(supercompression_scheme) = header.supercompression_scheme {
        for level in reader.levels() {
            match supercompression_scheme {
                SupercompressionScheme::Zstandard => {
                    let mut cursor = std::io::Cursor::new(level.data);
                    let mut decoder = ruzstd::decoding::StreamingDecoder::new(&mut cursor)?;
                    let mut decompressed = Vec::new();
                    decoder.read_to_end(&mut decompressed)?;
                    levels.push(decompressed);
                }
                _ => {
                    return Err(Error::msg(format!(
                        "Unsupported supercompression scheme: {supercompression_scheme:?}. Only zstd is supported.",
                    )));
                }
            }
        }
    } else {
        levels = reader.levels().map(|level| level.data.to_vec()).collect();
    }
    Ok(levels)
}
