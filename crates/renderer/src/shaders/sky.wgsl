struct Camera {
    view_pos: vec4<f32>,
    view: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var env_map_texture: texture_cube<f32>;

@group(1) @binding(1)
var env_map_sampler: sampler;

struct Sky {
    exposure_linear: f32,
    sh_coefficients: array<vec4<f32>, 9>,
}
@group(1) @binding(2)
var<uniform> sky: Sky;

struct VertexOutput {
    @builtin(position) frag_position: vec4<f32>,
    @location(0) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) id: u32,
) -> VertexOutput {
    let uv = vec2<f32>(vec2<u32>(
        id & 1u,
        (id >> 1u) & 1u,
    ));
    var out: VertexOutput;

    // create triangle TWICE the size of the screen
    out.clip_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);
    out.frag_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);
    return out;
}

fn irradianceSH(n: vec3<f32>) -> vec3<f32> {
    let result =
          sky.sh_coefficients[0]
        + sky.sh_coefficients[1] * (n.y)
        + sky.sh_coefficients[2] * (n.z)
        + sky.sh_coefficients[3] * (n.x)
        + sky.sh_coefficients[4] * (n.y * n.x)
        + sky.sh_coefficients[5] * (n.y * n.z)
        + sky.sh_coefficients[6] * (3.0 * n.z * n.z - 1.0)
        + sky.sh_coefficients[7] * (n.z * n.x)
        + sky.sh_coefficients[8] * (n.x * n.x - n.y * n.y);

    return result.xyz;
}

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    // convert clip space to camera space
    let view_pos_homogeneous = camera.inv_proj * in.clip_position;
    // undo perspective
    let view_ray_direction = view_pos_homogeneous.xyz / view_pos_homogeneous.w;

    // convert camera space to world space
    var ray_direction = normalize((camera.inv_view * vec4(view_ray_direction, 0.0)).xyz);

    var sample = textureSample(env_map_texture, env_map_sampler, ray_direction);
    if (sky.exposure_linear > 0.5) {
        sample = vec4(irradianceSH(ray_direction), 0.0);
    }

    return sample;
    // return sample * sky.exposure_linear;
}

