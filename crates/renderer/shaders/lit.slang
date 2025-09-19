import "modules/math.slang";
import "modules/common/camera.slang";
import "modules/common/light.slang";
import "modules/common/bsdf-properties.slang";
import "modules/filament-brdf.slang";

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

import "modules/common/sky.slang";

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
    float4 model_matrix_col0;
    float4 model_matrix_col1;
    float4 model_matrix_col2;
    float4 model_matrix_col3;
    // normal matrix
    float3 normal_matrix_col0;
    float3 normal_matrix_col1;
    float3 normal_matrix_col2;
}

[shader("vertex")]
VertexOutput vs_main(
    VertexInput model,
    InstanceInput instance,
) {
    var out: VertexOutput;

    // In slang, the vectors provided to the matrix constructor are
    // interpreted as *columns* of the vector
    let model_matrix = float4x4(
      instance.model_matrix_col0,
      instance.model_matrix_col1,
      instance.model_matrix_col2,
      instance.model_matrix_col3,
    );

    let normal_matrix = float3x3(
      instance.normal_matrix_col0,
      instance.normal_matrix_col1,
      instance.normal_matrix_col2,
    );

    out.vertex_normal = mul(model.normal, normal_matrix);
    out.vertex_tangent = mul(model.tangent, normal_matrix);
    out.vertex_bitangent = mul(model.bitangent, normal_matrix);
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
    
    let world_position = mul(float4(model.position, 1.0), model_matrix);
    // Apply the camera transforms and perspective
    // projection to the model position.
    out.clip_position = mul(world_position, camera.view_proj);
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
    let vertex_bitangent = normalize(cross(vertex_normal, vertex_tangent));

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
    // clamp roughness at a min to avoid precision issues with certain operations. See:
    // https://google.github.io/filament/main/filament.html#roughness-remapping-and-clamping
    const float MIN_PERCEPTUAL_ROUGHNESS = .089;
    let perceptualRoughness = max(arm.y, MIN_PERCEPTUAL_ROUGHNESS);
    let metallic = arm.z;
    let reflectance = 0.5;

    // Unpack XY normal according to docs for --normal-mode here:
    // https://github.khronos.org/KTX-Software/ktxtools/ktx_create.html
    let normal_xy = obj_normal.xy * 2.0 - 1.0;
    let normal_z = sqrt(1 - dot(normal_xy, normal_xy));
    let tangent_normal = normalize(float3(normal_xy, normal_z));
    let world_normal = normalize(mul(tangent_normal, tangent_to_world));
    
    // Gather pixel properties
    var pixel_properties: PixelProperties;
    pixel_properties.view = view_dir;
    pixel_properties.normal = world_normal;
    pixel_properties.vertexNormal = vertex_normal;
    pixel_properties.reflection = reflect(-view_dir, world_normal);
    pixel_properties.perceptualRoughness = perceptualRoughness;
    pixel_properties.roughness = perceptualRoughness * perceptualRoughness;
    pixel_properties.diffuseColor = (1.0 - metallic) * base_color.rgb;
    pixel_properties.minReflectance = 0.16 * reflectance * reflectance * (1.0 - metallic) + base_color.rgb * metallic;
    
    var light_sum = float3(0.0, 0.0, 0.0);

    // Add contribution from all lights

    // Gather light properties
    var light_properties: LightProperties;
    light_properties.posToLight = light.position - in.world_position;
    light_properties.color = light.color;
    light_properties.intensity = light.intensity;

    // temporarily hardcoded light properties
    light_properties.direction = float3(1.0, 0.0, 0.0);
    light_properties.invRadius = 1.0/20.0;
    light_properties.innerAngle = PI;
    light_properties.outerAngle = PI;

    light_sum += evaluatePunctualLight(pixel_properties, light_properties);

    // Sky contribution
    light_sum += evaluateIBL(pixel_properties);

    return float4(light_sum, base_color.a);
}
