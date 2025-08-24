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

struct ColorSweepSettings {
    ev_min: f32,
    ev_max: f32,
    ev_step: f32,
    hue_min: f32,
    hue_max: f32,
    hue_step: f32,
}

fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, saturate(p - K.xxx), c.y);
}

fn color_sweep(uv: vec2<f32>, settings: ColorSweepSettings) -> vec3<f32> {
    let ev_min = settings.ev_min;
    let ev_max = settings.ev_max;
    let ev_step = settings.ev_step;
    let ev_step_count = (ev_max - ev_min) / ev_step;

    let hue_min = settings.hue_min;
    let hue_max = settings.hue_max;
    let hue_step = settings.hue_step;
    let hue_step_count = (hue_max - hue_min) / hue_step;

    let ev_idx = floor(uv.x * ev_step_count);
    let ev = ev_min + ev_idx * ev_step;
    let value = exp2(ev);

    let hue_idx = floor(uv.y * hue_step_count);
    let hue = hue_min + hue_idx * hue_step;
    let hue_normalized = hue / 360.0;

    let hsv = vec3(hue_normalized, 1.0, value);

    return hsv2rgb(hsv);
}

@fragment
fn fs_main(vs: VertexOutput) -> @location(0) vec4<f32> {
    let hdr_sample = textureSample(hdr_texture, hdr_sampler, vs.uv); 
    var hdr_color = hdr_sample.xyz;

    // Color Sweep Debug
    if (true) {
        var settings: ColorSweepSettings;
        settings.ev_min = -10.0;
        settings.ev_max = 5.0;
        settings.ev_step = 0.25;
        settings.hue_min = 0.0;
        settings.hue_max = 360.0;
        settings.hue_step = 15.0;

        hdr_color = color_sweep(vs.uv, settings);
    }
    
    hdr_color *= view_uniform.exposure_linear;

    var sdr = tone_map(hdr_color);

    // let sdr = draw_lut(vs.uv);
    return vec4(sdr, hdr_sample.a);
}
