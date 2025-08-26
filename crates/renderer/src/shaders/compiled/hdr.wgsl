struct ViewUniform_std140_0
{
    @align(16) exposure_linear_0 : f32,
};

struct ViewParameters_std140_0
{
    @align(16) view_uniform_0 : ViewUniform_std140_0,
};

@binding(0) @group(0) var<uniform> params_0 : ViewParameters_std140_0;
@binding(1) @group(0) var params_hdr_texture_0 : texture_2d<f32>;

@binding(2) @group(0) var params_hdr_sampler_0 : sampler;

@binding(3) @group(0) var params_lut_texture_0 : texture_3d<f32>;

@binding(4) @group(0) var params_lut_sampler_0 : sampler;

struct VertexStageOutput_0
{
    @location(0) color_0 : vec3<f32>,
    @location(1) uv_0 : vec2<f32>,
    @builtin(position) sv_position_0 : vec4<f32>,
};

struct CoarseVertex_0
{
     _S1 : vec3<f32>,
     _S2 : vec2<f32>,
};

struct VertexStageOutput_1
{
     coarseVertex_0 : CoarseVertex_0,
     _S3 : vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index_0 : u32) -> VertexStageOutput_0
{
    var _S4 : f32 = f32((index_0 & (u32(2))));
    var _S5 : vec2<f32> = vec2<f32>(f32((((index_0 << (u32(1)))) & (u32(2)))), _S4);
    var uv_1 : vec2<f32> = _S5;
    var output_0 : VertexStageOutput_1;
    output_0.coarseVertex_0._S1 = vec3<f32>(_S5, 1.0f);
    output_0._S3 = vec4<f32>(_S5 * vec2<f32>(2.0f) - vec2<f32>(1.0f), 0.0f, 1.0f);
    uv_1[i32(1)] = 1.0f - _S4;
    output_0.coarseVertex_0._S2 = uv_1;
    var _S6 : VertexStageOutput_0;
    _S6.color_0 = output_0.coarseVertex_0._S1;
    _S6.uv_0 = output_0.coarseVertex_0._S2;
    _S6.sv_position_0 = output_0._S3;
    return _S6;
}

fn scene_linear_to_shaper_space_0( color_1 : vec3<f32>) -> vec3<f32>
{
    return vec3<f32>(0.08333088457584381f) * log2(vec3<f32>(4096.0f) * color_1 + vec3<f32>(100.0f)) + vec3<f32>(-0.60000002384185791f);
}

fn tone_map_0( color_2 : vec3<f32>) -> vec3<f32>
{
    return (textureSampleLevel((params_lut_texture_0), (params_lut_sampler_0), (scene_linear_to_shaper_space_0(color_2)), (0.0f))).xyz;
}

struct Fragment_0
{
    @location(0) color_3 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) _S7 : vec3<f32>,
    @location(1) _S8 : vec2<f32>,
};

struct pixelInput_1
{
     coarseVertex_1 : CoarseVertex_0,
};

@fragment
fn fs_main( _S9 : pixelInput_0) -> Fragment_0
{
    var _S10 : pixelInput_1;
    _S10.coarseVertex_1._S1 = _S9._S7;
    _S10.coarseVertex_1._S2 = _S9._S8;
    var output_1 : Fragment_0;
    output_1.color_3 = vec4<f32>(tone_map_0((textureSampleLevel((params_hdr_texture_0), (params_hdr_sampler_0), (_S10.coarseVertex_1._S2), (0.0f))).xyz * vec3<f32>(params_0.view_uniform_0.exposure_linear_0)), 1.0f);
    return output_1;
}

