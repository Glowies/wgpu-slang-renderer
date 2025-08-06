struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) idx: u32,
) -> VertexOutput {
    var out: VertexOutput;
    // Generate a triangle that covers the whole screen
    out.uv = vec2<f32>(
        f32((idx << 1u) & 2u),
        f32(idx & 2u),
    );
    out.clip_position = vec4<f32>(out.uv * 2.0 - 1.0, 0.0, 1.0);
    // We need to invert the y coordinate so the image
    // is not upside down
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

struct ViewUniform {
    exposure_linear: f32,
    _padding: vec3<f32>,
}

@group(0) @binding(0)
var hdr_texture: texture_2d<f32>;

@group(0) @binding(1)
var hdr_sampler: sampler;

@group(0) @binding(2)
var lut_texture: texture_3d<f32>;

@group(0) @binding(3)
var lut_sampler: sampler;

@group(0) @binding(4)
var<uniform> view_uniform: ViewUniform;

fn scene_linear_to_shaper_space(color: vec3<f32>) -> vec3<f32> {
    // constants from the ocio shaper space
    let log_side_slope = 0.0833308877282;
    let lin_side_slope = 4096.0;
    let lin_side_offset = 1.0;

    return log_side_slope * log2(lin_side_slope * color + lin_side_offset);
}

const TONY_MC_MAPFACE_LUT_DIMS: f32 = 48.0;

fn sample_tony_mc_mapface_lut(stimulus: vec3<f32>) -> vec3<f32> {
    var uv = (stimulus / (stimulus + 1.0)) * (f32(TONY_MC_MAPFACE_LUT_DIMS - 1.0) / f32(TONY_MC_MAPFACE_LUT_DIMS)) + 0.5 / f32(TONY_MC_MAPFACE_LUT_DIMS);
    return textureSampleLevel(lut_texture, lut_sampler, saturate(uv), 0.0).rgb;
}

fn tone_map(color: vec3<f32>) -> vec3<f32> {
    // return sample_tony_mc_mapface_lut(color);

    // OCIO Tone Map
    let shaper_color = scene_linear_to_shaper_space(color);

    return textureSampleLevel(lut_texture, lut_sampler, shaper_color, 0.0).rgb;
}

fn draw_lut(uv: vec2<f32>) -> vec3<f32> {
    let size = f32(textureDimensions(lut_texture).x);
    let x_scaled = (uv.x * size);
    let x_z = vec2(x_scaled % 1.0, floor(x_scaled) / size);
    let uvw = vec3(x_z.x, uv.y, x_z.y);
    return textureSampleLevel(lut_texture, lut_sampler, uvw, 0.0).rgb;
}

@fragment
fn fs_main(vs: VertexOutput) -> @location(0) vec4<f32> {
    let hdr = textureSample(hdr_texture, hdr_sampler, vs.uv);
    let color = hdr.rgb * view_uniform.exposure_linear;
    let sdr = tone_map(color);
    // let sdr = draw_lut(vs.uv);
    return vec4(sdr, hdr.a);
}
