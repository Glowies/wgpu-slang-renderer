struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_row0: vec4<f32>,
    @location(6) model_matrix_row1: vec4<f32>,
    @location(7) model_matrix_row2: vec4<f32>,
    @location(8) model_matrix_row3: vec4<f32>,
}

struct LightUniform {
    position: vec3<f32>,
    intensity: f32,
    color: vec3<f32>,
}

@group(2) @binding(0)
var<uniform> light: LightUniform;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    let model_matrix = mat4x4<f32>(
        instance.model_matrix_row0,
        instance.model_matrix_row1,
        instance.model_matrix_row2,
        instance.model_matrix_row3,
    );
    
    var world_position = model_matrix * vec4<f32>(model.position, 1.0);
    // Apply the camera transforms and perspective
    // projection to the model position.
    out.clip_position = camera.view_proj * world_position;
    out.tex_coords = model.tex_coords;
    out.world_normal = model.normal;
    out.world_position = world_position.xyz;

    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var light_sum = vec3<f32>(0.0, 0.0, 0.0);
    
    // Ambient Light
    let ambient_factor = 0.01;
    let ambient_color = light.color * ambient_factor;
    light_sum += ambient_color;

    // Direct Light
    let point_to_light = light.position - in.world_position;
    let light_dir = normalize(point_to_light);
    let light_distance = length(point_to_light);
    var diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    diffuse_strength /= light_distance * light_distance;
    let diffuse_color = light.color * diffuse_strength;
    light_sum += diffuse_color;

    let obj_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    let result = light_sum * obj_color.xyz;
    return vec4<f32>(result, obj_color.a);
}
