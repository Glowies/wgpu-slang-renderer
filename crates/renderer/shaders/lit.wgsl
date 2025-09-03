struct _MatrixStorage_float4x4std140_0
{
    @align(16) data_0 : array<vec4<f32>, i32(4)>,
};

struct CameraUniform_std140_0
{
    @align(16) view_pos_0 : vec4<f32>,
    @align(16) view_0 : _MatrixStorage_float4x4std140_0,
    @align(16) view_proj_0 : _MatrixStorage_float4x4std140_0,
    @align(16) inv_proj_0 : _MatrixStorage_float4x4std140_0,
    @align(16) inv_view_0 : _MatrixStorage_float4x4std140_0,
};

@binding(0) @group(1) var<uniform> camera_0 : CameraUniform_std140_0;
@binding(0) @group(0) var textures_t_diffuse_0 : texture_2d<f32>;

@binding(1) @group(0) var textures_s_diffuse_0 : sampler;

@binding(2) @group(0) var textures_t_normal_0 : texture_2d<f32>;

@binding(3) @group(0) var textures_s_normal_0 : sampler;

@binding(4) @group(0) var textures_t_arm_0 : texture_2d<f32>;

@binding(5) @group(0) var textures_s_arm_0 : sampler;

struct LightUniform_std140_0
{
    @align(16) position_0 : vec3<f32>,
    @align(4) intensity_0 : f32,
    @align(16) color_0 : vec3<f32>,
};

@binding(0) @group(2) var<uniform> light_0 : LightUniform_std140_0;
struct SkyUniform_std140_0
{
    @align(16) sh_coefficients_0 : array<vec4<f32>, i32(9)>,
    @align(16) exposure_linear_0 : f32,
    @align(4) debug_sh_0 : f32,
};

struct SkyParameters_std140_0
{
    @align(16) properties_0 : SkyUniform_std140_0,
};

@binding(0) @group(3) var<uniform> sky_0 : SkyParameters_std140_0;
struct VertexOutput_0
{
    @builtin(position) clip_position_0 : vec4<f32>,
    @location(0) tex_coords_0 : vec2<f32>,
    @location(1) world_position_0 : vec3<f32>,
    @location(2) vertex_normal_0 : vec3<f32>,
    @location(3) vertex_tangent_0 : vec3<f32>,
    @location(4) vertex_bitangent_0 : vec3<f32>,
};

struct vertexInput_0
{
    @location(0) position_1 : vec3<f32>,
    @location(1) tex_coords_1 : vec2<f32>,
    @location(2) normal_0 : vec3<f32>,
    @location(3) tangent_0 : vec3<f32>,
    @location(4) bitangent_0 : vec3<f32>,
    @location(5) model_matrix_row0_0 : vec4<f32>,
    @location(6) model_matrix_row1_0 : vec4<f32>,
    @location(7) model_matrix_row2_0 : vec4<f32>,
    @location(8) model_matrix_row3_0 : vec4<f32>,
    @location(9) normal_matrix_row0_0 : vec3<f32>,
    @location(10) normal_matrix_row1_0 : vec3<f32>,
    @location(11) normal_matrix_row2_0 : vec3<f32>,
};

var<private> PI_0 : f32;

@vertex
fn vs_main( _S1 : vertexInput_0) -> VertexOutput_0
{
    PI_0 = 3.14159274101257324f;
    var _S2 : mat4x4<f32> = mat4x4<f32>(_S1.model_matrix_row0_0, _S1.model_matrix_row1_0, _S1.model_matrix_row2_0, _S1.model_matrix_row3_0);
    var _S3 : mat3x3<f32> = mat3x3<f32>(_S1.normal_matrix_row0_0, _S1.normal_matrix_row1_0, _S1.normal_matrix_row2_0);
    var out_0 : VertexOutput_0;
    out_0.vertex_normal_0 = (((_S1.normal_0) * (_S3)));
    out_0.vertex_tangent_0 = (((_S1.tangent_0) * (_S3)));
    out_0.vertex_bitangent_0 = (((_S1.bitangent_0) * (_S3)));
    var _S4 : vec4<f32> = (((vec4<f32>(_S1.position_1, 1.0f)) * (_S2)));
    out_0.clip_position_0 = (((_S4) * (mat4x4<f32>(camera_0.view_proj_0.data_0[i32(0)][i32(0)], camera_0.view_proj_0.data_0[i32(0)][i32(1)], camera_0.view_proj_0.data_0[i32(0)][i32(2)], camera_0.view_proj_0.data_0[i32(0)][i32(3)], camera_0.view_proj_0.data_0[i32(1)][i32(0)], camera_0.view_proj_0.data_0[i32(1)][i32(1)], camera_0.view_proj_0.data_0[i32(1)][i32(2)], camera_0.view_proj_0.data_0[i32(1)][i32(3)], camera_0.view_proj_0.data_0[i32(2)][i32(0)], camera_0.view_proj_0.data_0[i32(2)][i32(1)], camera_0.view_proj_0.data_0[i32(2)][i32(2)], camera_0.view_proj_0.data_0[i32(2)][i32(3)], camera_0.view_proj_0.data_0[i32(3)][i32(0)], camera_0.view_proj_0.data_0[i32(3)][i32(1)], camera_0.view_proj_0.data_0[i32(3)][i32(2)], camera_0.view_proj_0.data_0[i32(3)][i32(3)]))));
    out_0.world_position_0 = _S4.xyz;
    out_0.tex_coords_0 = _S1.tex_coords_1;
    return out_0;
}

fn D_GGX_0( NoH_0 : f32,  a_0 : f32) -> f32
{
    var _S5 : f32 = a_0 * a_0;
    var _S6 : f32 = (NoH_0 * _S5 - NoH_0) * NoH_0 + 1.0f;
    return _S5 / (PI_0 * _S6 * _S6);
}

fn F_Schlick_0( u_0 : f32,  f0_0 : vec3<f32>) -> vec3<f32>
{
    return f0_0 + (vec3<f32>(1.0f) - f0_0) * vec3<f32>(pow(1.0f - u_0, 5.0f));
}

fn V_SmithGGXCorrelated_0( NoV_0 : f32,  NoL_0 : f32,  a_1 : f32) -> f32
{
    var _S7 : f32 = a_1 * a_1;
    return 0.5f / (NoL_0 * sqrt((- NoV_0 * _S7 + NoV_0) * NoV_0 + _S7) + NoV_0 * sqrt((- NoL_0 * _S7 + NoL_0) * NoL_0 + _S7));
}

fn Fd_Lambert_0() -> f32
{
    return 1.0f / PI_0;
}

struct PixelProperties_0
{
     view_1 : vec3<f32>,
     normal_1 : vec3<f32>,
     roughness_0 : f32,
     diffuseColor_0 : vec3<f32>,
     min_reflectance_0 : vec3<f32>,
};

struct LightProperties_0
{
     direction_0 : vec3<f32>,
};

fn BRDF_0( pixel_0 : PixelProperties_0,  light_1 : LightProperties_0) -> vec3<f32>
{
    var _S8 : vec3<f32> = normalize(pixel_0.view_1 + light_1.direction_0);
    return vec3<f32>((D_GGX_0(clamp(dot(pixel_0.normal_1, _S8), 0.0f, 1.0f), pixel_0.roughness_0) * V_SmithGGXCorrelated_0(abs(dot(pixel_0.normal_1, pixel_0.view_1)) + 0.00000999999974738f, clamp(dot(pixel_0.normal_1, light_1.direction_0), 0.0f, 1.0f), pixel_0.roughness_0))) * F_Schlick_0(clamp(dot(light_1.direction_0, _S8), 0.0f, 1.0f), pixel_0.min_reflectance_0) + pixel_0.diffuseColor_0 * vec3<f32>(Fd_Lambert_0());
}

fn irradianceSH_0( n_0 : vec3<f32>,  sh_coefficients_1 : array<vec4<f32>, i32(9)>) -> vec3<f32>
{
    var _S9 : f32 = n_0.y;
    var _S10 : f32 = n_0.z;
    var _S11 : f32 = n_0.x;
    return (sh_coefficients_1[i32(0)] + sh_coefficients_1[i32(1)] * vec4<f32>(_S9) + sh_coefficients_1[i32(2)] * vec4<f32>(_S10) + sh_coefficients_1[i32(3)] * vec4<f32>(_S11) + sh_coefficients_1[i32(4)] * vec4<f32>((_S9 * _S11)) + sh_coefficients_1[i32(5)] * vec4<f32>((_S9 * _S10)) + sh_coefficients_1[i32(6)] * vec4<f32>((3.0f * _S10 * _S10 - 1.0f)) + sh_coefficients_1[i32(7)] * vec4<f32>((_S10 * _S11)) + sh_coefficients_1[i32(8)] * vec4<f32>((_S11 * _S11 - _S9 * _S9))).xyz;
}

struct pixelOutput_0
{
    @location(0) output_0 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) tex_coords_2 : vec2<f32>,
    @location(1) world_position_1 : vec3<f32>,
    @location(2) vertex_normal_1 : vec3<f32>,
    @location(3) vertex_tangent_1 : vec3<f32>,
    @location(4) vertex_bitangent_1 : vec3<f32>,
};

@fragment
fn fs_main( _S12 : pixelInput_0, @builtin(position) clip_position_1 : vec4<f32>) -> pixelOutput_0
{
    PI_0 = 3.14159274101257324f;
    var _S13 : vec3<f32> = normalize(_S12.vertex_normal_1);
    var _S14 : vec3<f32> = normalize(_S12.vertex_tangent_1 - vec3<f32>(dot(_S12.vertex_tangent_1, _S13)) * _S13);
    var _S15 : vec4<f32> = (textureSample((textures_t_diffuse_0), (textures_s_diffuse_0), (_S12.tex_coords_2)));
    var _S16 : vec4<f32> = (textureSample((textures_t_arm_0), (textures_s_arm_0), (_S12.tex_coords_2)));
    var _S17 : f32 = _S16.y;
    var _S18 : f32 = _S16.z;
    var _S19 : vec2<f32> = (textureSample((textures_t_normal_0), (textures_s_normal_0), (_S12.tex_coords_2))).xy * vec2<f32>(2.0f) - vec2<f32>(1.0f);
    var _S20 : vec3<f32> = normalize((((normalize(vec3<f32>(_S19, sqrt(1.0f - dot(_S19, _S19))))) * (mat3x3<f32>(_S14, normalize(cross(_S14, _S13)), _S13)))));
    var pixel_properties_0 : PixelProperties_0;
    pixel_properties_0.view_1 = normalize(camera_0.view_pos_0.xyz - _S12.world_position_1);
    pixel_properties_0.normal_1 = _S20;
    pixel_properties_0.roughness_0 = _S17 * _S17;
    var _S21 : f32 = 1.0f - _S18;
    var _S22 : vec3<f32> = _S15.xyz;
    pixel_properties_0.diffuseColor_0 = vec3<f32>(_S21) * _S22;
    pixel_properties_0.min_reflectance_0 = vec3<f32>((0.03999999910593033f * _S21)) + _S22 * vec3<f32>(_S18);
    var _S23 : vec3<f32> = light_0.position_0 - _S12.world_position_1;
    var _S24 : f32 = length(_S23);
    var _S25 : vec3<f32> = light_0.color_0 * vec3<f32>(light_0.intensity_0) * vec3<f32>((1.0f / (_S24 * _S24)));
    var light_properties_0 : LightProperties_0;
    light_properties_0.direction_0 = normalize(_S23);
    var _S26 : pixelOutput_0 = pixelOutput_0( vec4<f32>(BRDF_0(pixel_properties_0, light_properties_0) * _S25 + pixel_properties_0.diffuseColor_0 * (max(irradianceSH_0(_S20, sky_0.properties_0.sh_coefficients_0), vec3<f32>(0.0f, 0.0f, 0.0f)) * vec3<f32>(sky_0.properties_0.exposure_linear_0)) * vec3<f32>(Fd_Lambert_0()), _S15.w) );
    return _S26;
}

