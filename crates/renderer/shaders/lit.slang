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

import "modules/math.slang";
import "modules/uniforms/camera.slang";
import "modules/uniforms/light.slang";
import "modules/uniforms/sky.slang";

struct MaterialTextureSet {
  Texture2D t_diffuse;
  SamplerState s_diffuse;
  Texture2D t_normal;
  SamplerState s_normal;
  Texture2D t_arm;
  SamplerState s_arm;
}

// Uniforms
ParameterBlock<MaterialTextureSet> textures;
ParameterBlock<CameraUniform> camera;
ParameterBlock<LightUniform> light;
ParameterBlock<SkyParameters> sky;

struct PixelProperties {
    float3 view;
    float3 normal;
    float roughness; // roughness = perceptualRoughness * perceptualRoughness
    float3 diffuseColor; // diffuseColor = (1.0 - metallic) * baseColor.rgb
    // min_reflectance(f0) for dielectrics is a function of reflectance while for metal it comes directly from baseColor
    float3 min_reflectance; // f0 = 0.16 * reflectance * reflectance * (1.0 - metallic) + baseColor * metallic;
}

struct LightProperties {
    float3 direction; 
}

struct VertexInput {
    float3 position;
    float2 tex_coords;
    float3 normal;
    float3 tangent;
    float3 bitangent;
}

// Location 0 to 4
struct VertexOutput {
    float4 clip_position: SV_Position;
    float2 tex_coords;
    float3 world_position;
    float3 vertex_normal;
    float3 vertex_tangent;
    float3 vertex_bitangent;
}

// Location 5 to 11
struct InstanceInput {
    // model matrix
    float4 model_matrix_row0;
    float4 model_matrix_row1;
    float4 model_matrix_row2;
    float4 model_matrix_row3;
    // normal matrix
    float3 normal_matrix_row0;
    float3 normal_matrix_row1;
    float3 normal_matrix_row2;
}

[shader("vertex")]
VertexOutput vs_main(
    VertexInput model,
    InstanceInput instance,
) {
    var out: VertexOutput;

    
    let model_matrix = float4x4(
      instance.model_matrix_row0,
      instance.model_matrix_row1,
      instance.model_matrix_row2,
      instance.model_matrix_row3,
    );

    let normal_matrix = float3x3(
      instance.normal_matrix_row0,
      instance.normal_matrix_row1,
      instance.normal_matrix_row2,
    );

    out.vertex_normal = mul(normal_matrix, model.normal);
    out.vertex_tangent = mul(normal_matrix, model.tangent);
    out.vertex_bitangent = mul(normal_matrix, model.bitangent);
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
    
    let world_position = mul(model_matrix, float4(model.position, 1.0));
    // Apply the camera transforms and perspective
    // projection to the model position.
    out.clip_position = mul(camera.view_proj, world_position);
    out.world_position = world_position.xyz;

    out.tex_coords = model.tex_coords;

    return out;
}

[shader("fragment")]
float4 fs_main(VertexOutput in) {
    let vertex_normal = normalize(in.vertex_normal);
    // Adjust the tangent and bitangent using the Gramm-Schmidt process
    // This makes sure that they are perpendicular to each other and the
    // normal of the surface.
    let vertex_tangent = normalize(in.vertex_tangent - dot(in.vertex_tangent, vertex_normal) * vertex_normal);
    let vertex_bitangent = normalize(cross(vertex_tangent, vertex_normal));

    let tangent_to_world = float3x3(
        vertex_tangent,
        vertex_bitangent,
        vertex_normal,
    );

    // View Properties
    let view_dir = normalize(camera.view_pos.xyz - in.world_position);

    // PBR Texture Samples
    let base_color = textures.t_diffuse.Sample(textures.s_diffuse, in.tex_coords);
    let obj_normal = textures.t_normal.Sample(textures.s_normal, in.tex_coords);
    let arm = textures.t_arm.Sample(textures.s_arm, in.tex_coords);
    let perceptualRoughness = arm.y;
    let metallic = arm.z;
    let reflectance = 0.5;

    // Unpack XY normal according to docs for --normal-mode here:
    // https://github.khronos.org/KTX-Software/ktxtools/ktx_create.html
    let normal_xy = obj_normal.xy * 2.0 - 1.0;
    let normal_z = sqrt(1 - dot(normal_xy, normal_xy));
    let tangent_normal = normalize(float3(normal_xy, normal_z));
    let world_normal = normalize(mul(tangent_to_world, tangent_normal));
    
    // Gather pixel properties
    var pixel_properties: PixelProperties;
    pixel_properties.view = view_dir;
    pixel_properties.normal = world_normal;
    pixel_properties.roughness = perceptualRoughness * perceptualRoughness;
    pixel_properties.diffuseColor = (1.0 - metallic) * base_color.rgb;
    pixel_properties.min_reflectance = 0.16 * reflectance * reflectance * (1.0 - metallic) + base_color.rgb * metallic;
    
    var light_sum = float3(0.0, 0.0, 0.0);

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
    let sky_irradiance = max(
      irradianceSH(world_normal, sky.properties.sh_coefficients),
      float3(0.0, 0.0, 0.0)
    ) * sky.properties.exposure_linear;
    let sky_diffuse = pixel_properties.diffuseColor * sky_irradiance * Fd_Lambert();
    light_sum += sky_diffuse;

    return float4(light_sum, base_color.a);
}

float D_GGX(float NoH, float a) {
    let a2 = a * a;
    let f = (NoH * a2 - NoH) * NoH + 1.0;
    return a2 / (PI * f * f);
}

float3 F_Schlick(float u, float3 f0) {
    return f0 + (float3(1.0) - f0) * pow(1.0 - u, 5.0);
}

float V_SmithGGXCorrelated(float NoV, float NoL, float a) {
    let a2 = a * a;
    let GGXL = NoV * sqrt((-NoL * a2 + NoL) * NoL + a2);
    let GGXV = NoL * sqrt((-NoV * a2 + NoV) * NoV + a2);
    return 0.5 / (GGXV + GGXL);
}

float Fd_Lambert() {
    return 1.0 / PI;
}

float3 BRDF(PixelProperties pixel, LightProperties light) {
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
