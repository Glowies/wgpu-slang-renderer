// From the Filament design doc
// https://google.github.io/filament/Filament.html#table_symbols
// Symbol Definition
// v    View unit vector
// l    Incident light unit vector
// n    Surface normal unit vector
// h    Half unit vector between l and v
// f    BRDF
// f_d    Diffuse component of a BRDF
// f_r    Specular component of a BRDF
// α    Roughness, remapped from using input perceptualRoughness
// σ    Diffuse reflectance
// Ω    Spherical domain
// f0    Reflectance at normal incidence
// f90    Reflectance at grazing angle
// χ+(a)    Heaviside function (1 if a>0 and 0 otherwise)
// nior    Index of refraction (IOR) of an interface
// ⟨n⋅l⟩    Dot product clamped to [0..1]
// ⟨a⟩    Saturated value (clamped to [0..1])

const PI: f32 = 3.1415926535897932384626433832795;

struct CameraUniform {
    view_pos: vec4<f32>,
    view: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) vertex_normal: vec3<f32>,
    @location(3) vertex_tangent: vec3<f32>,
    @location(4) vertex_bitangent: vec3<f32>,
}

struct InstanceInput {
    // model matrix
    @location(5) model_matrix_row0: vec4<f32>,
    @location(6) model_matrix_row1: vec4<f32>,
    @location(7) model_matrix_row2: vec4<f32>,
    @location(8) model_matrix_row3: vec4<f32>,
    // normal matrix
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
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

    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );

    out.vertex_normal = (normal_matrix * model.normal);
    out.vertex_tangent = (normal_matrix * model.tangent);
    out.vertex_bitangent = (normal_matrix * model.bitangent);
    // ^^^ We need matrices to go from world space to tangent space because
    // our normals in our textures are in tangent space. There are two solutions:
    // 1. Use tangent_to_world to convert our normals to world space
    // 2. Use world_to_tangent to convert all positions for light calculations into
    //     tangent space.
    //
    // 1. needs to be done in the fragment shader because each frag has its own normals
    // 2. can be done in vertex shader since view and light positions don't change per fragment.
    //     we can calculate them once in the vertex shader, and let interpolation do the rest.
    //
    // However, in this case, I preferred to use solution 1. because I think it will make
    // implementing multiple lights easier in the future.
    
    let world_position = model_matrix * vec4<f32>(model.position, 1.0);
    // Apply the camera transforms and perspective
    // projection to the model position.
    out.clip_position = camera.view_proj * world_position;
    out.world_position = world_position.xyz;

    out.tex_coords = model.tex_coords;

    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var t_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;
@group(0) @binding(4)
var t_arm: texture_2d<f32>;
@group(0) @binding(5)
var s_arm: sampler;

@group(3) @binding(0)
var env_map_texture: texture_cube<f32>;
@group(3) @binding(1)
var env_map_sampler: sampler;

// Note: You CANNOT put the exposure_linear *before* the sh_coefficients
// array. This throws off the alignment of the array. (I guess each entry
// in this struct needs to be aligned to 16 bytes, but the exposure_linear
// f32 is only 4 bytes)
struct Sky {
    sh_coefficients: array<vec4<f32>, 9>,
    exposure_linear: f32,
}
@group(3) @binding(2)
var<uniform> sky: Sky;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let vertex_normal = normalize(in.vertex_normal);
    // Adjust the tangent and bitangent using the Gramm-Schmidt process
    // This makes sure that they are perpendicular to each other and the
    // normal of the surface.
    let vertex_tangent = normalize(in.vertex_tangent - dot(in.vertex_tangent, vertex_normal) * vertex_normal);
    let vertex_bitangent = normalize(cross(vertex_tangent, vertex_normal));

    let tangent_to_world = mat3x3<f32>(
        vertex_tangent,
        vertex_bitangent,
        vertex_normal,
    );

    // View Properties
    let view_dir = normalize(camera.view_pos.xyz - in.world_position);

    // PBR Texture Samples
    let base_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let obj_normal: vec4<f32> = textureSample(t_normal, s_normal, in.tex_coords);
    let arm: vec4<f32> = textureSample(t_arm, s_arm, in.tex_coords);
    let perceptualRoughness = arm.y;
    let metallic = arm.z;
    let reflectance = 0.5;

    // Unpack XY normal according to docs for --normal-mode here:
    // https://github.khronos.org/KTX-Software/ktxtools/ktx_create.html
    let normal_xy = obj_normal.xy * 2.0 - 1.0;
    let normal_z = sqrt(1 - dot(normal_xy, normal_xy));
    let tangent_normal = normalize(vec3(normal_xy, normal_z));
    let world_normal = normalize(tangent_to_world * tangent_normal);
    
    // Gather pixel properties
    var pixel_properties: PixelProperties;
    pixel_properties.view = view_dir;
    pixel_properties.normal = world_normal;
    pixel_properties.roughness = perceptualRoughness * perceptualRoughness;
    pixel_properties.diffuseColor = (1.0 - metallic) * base_color.rgb;
    pixel_properties.min_reflectance = 0.16 * reflectance * reflectance * (1.0 - metallic) + base_color.rgb * metallic;
    
    var light_sum = vec3<f32>(0.0, 0.0, 0.0);

    // Add contribution from all lights

    // Light Properties
    let point_to_light = light.position - in.world_position;
    let light_dir = normalize(point_to_light);
    let light_distance = length(point_to_light);
    let light_attenuation = 1.0 / (light_distance * light_distance);
    let light_color = light.color * light.intensity * light_attenuation;

    // Gather light properties
    var light_properties: LightProperties;
    light_properties.direction = light_dir;
    light_sum = BRDF(pixel_properties, light_properties) * light_color;

    // Sky contribution
    let sky_irradiance = max(irradianceSH(world_normal), vec3(0.0, 0.0, 0.0)) * sky.exposure_linear;
    let sky_diffuse = pixel_properties.diffuseColor * sky_irradiance * Fd_Lambert();
    light_sum += sky_diffuse;

    return vec4<f32>(light_sum, base_color.a);
}

fn D_GGX(NoH: f32, a: f32) -> f32 {
    let a2 = a * a;
    let f = (NoH * a2 - NoH) * NoH + 1.0;
    return a2 / (PI * f * f);
}

fn F_Schlick(u: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - u, 5.0);
}

fn V_SmithGGXCorrelated(NoV: f32, NoL: f32, a: f32) -> f32 {
    let a2 = a * a;
    let GGXL = NoV * sqrt((-NoL * a2 + NoL) * NoL + a2);
    let GGXV = NoL * sqrt((-NoV * a2 + NoV) * NoV + a2);
    return 0.5 / (GGXV + GGXL);
}

fn Fd_Lambert() -> f32 {
    return 1.0 / PI;
}


struct PixelProperties {
    view: vec3<f32>,
    normal: vec3<f32>,
    roughness: f32, // roughness = perceptualRoughness * perceptualRoughness
    diffuseColor: vec3<f32>, // diffuseColor = (1.0 - metallic) * baseColor.rgb
    // min_reflectance(f0) for dielectrics is a function of reflectance while for metal it comes directly from baseColor
    min_reflectance: vec3<f32>, // f0 = 0.16 * reflectance * reflectance * (1.0 - metallic) + baseColor * metallic;
}

struct LightProperties {
    direction: vec3<f32>,
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

fn BRDF(pixel: PixelProperties, light: LightProperties) -> vec3<f32> {
    // destruct pixel struct
    let diffuseColor = pixel.diffuseColor;
    let roughness = pixel.roughness;
    let f0 = pixel.min_reflectance;
    let n = pixel.normal;
    let v = pixel.view;

    // destruct light struct
    let l = light.direction;
    
    let h = normalize(v + l);

    let NoV = abs(dot(n, v)) + 1e-5;
    let NoL = clamp(dot(n, l), 0.0, 1.0);
    let NoH = clamp(dot(n, h), 0.0, 1.0);
    let LoH = clamp(dot(l, h), 0.0, 1.0);

    let D = D_GGX(NoH, roughness);
    let F = F_Schlick(LoH, f0);
    let V = V_SmithGGXCorrelated(NoV, NoL, roughness);

    // specular BRDF
    let Fr = (D * V) * F;

    // diffuse BRDF
    let Fd = diffuseColor * Fd_Lambert();

    return Fr + Fd;
}
