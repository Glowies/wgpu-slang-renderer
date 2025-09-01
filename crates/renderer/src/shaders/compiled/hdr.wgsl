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

fn ocio_rec709_to_acescct_0( inPixel_0 : vec4<f32>) -> vec4<f32>
{
    var outColor_0 : vec4<f32> = inPixel_0;
    var _S7 : vec3<f32> = inPixel_0.xyz;
    var res_0 : vec4<f32> = (((mat4x4<f32>(0.6130974292755127f, 0.07019372284412384f, 0.0206155925989151f, 0.0f, 0.33952313661575317f, 0.91635388135910034f, 0.10956977307796478f, 0.0f, 0.04737945273518562f, 0.01345239859074354f, 0.86981463432312012f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f)) * (vec4<f32>(_S7.x, _S7.y, _S7.z, inPixel_0.w))));
    var _S8 : vec3<f32> = vec3<f32>(res_0.x, res_0.y, res_0.z);
    outColor_0.x = _S8.x;
    outColor_0.y = _S8.y;
    outColor_0.z = _S8.z;
    outColor_0[i32(3)] = res_0.w;
    const minValue_0 : vec3<f32> = vec3<f32>(1.17549435082228751e-38f, 1.17549435082228751e-38f, 1.17549435082228751e-38f);
    const linear_segment_slope_0 : vec3<f32> = vec3<f32>(10.5402374267578125f, 10.5402374267578125f, 10.5402374267578125f);
    const linear_segment_offset_0 : vec3<f32> = vec3<f32>(0.07290557026863098f, 0.07290557026863098f, 0.07290557026863098f);
    const log_slope_0 : vec3<f32> = vec3<f32>(0.08234560489654541f, 0.08234560489654541f, 0.08234560489654541f);
    const log_offset_0 : vec3<f32> = vec3<f32>(0.5547945499420166f, 0.5547945499420166f, 0.5547945499420166f);
    var _S9 : f32;
    if((outColor_0.xyz[i32(0)]) > 0.0078125f)
    {
        _S9 = 1.0f;
    }
    else
    {
        _S9 = 0.0f;
    }
    var _S10 : f32;
    if((outColor_0.xyz[i32(1)]) > 0.0078125f)
    {
        _S10 = 1.0f;
    }
    else
    {
        _S10 = 0.0f;
    }
    var _S11 : f32;
    if((outColor_0.xyz[i32(2)]) > 0.0078125f)
    {
        _S11 = 1.0f;
    }
    else
    {
        _S11 = 0.0f;
    }
    var isAboveBreak_0 : vec3<f32> = vec3<f32>(_S9, _S10, _S11);
    var _S12 : vec3<f32> = isAboveBreak_0 * (log_slope_0 * log(max(minValue_0, outColor_0.xyz)) + log_offset_0) + (vec3<f32>(1.0f, 1.0f, 1.0f) - isAboveBreak_0) * (outColor_0.xyz * linear_segment_slope_0 + linear_segment_offset_0);
    outColor_0.x = _S12.x;
    outColor_0.y = _S12.y;
    outColor_0.z = _S12.z;
    return outColor_0;
}

fn tone_map_0( color_1 : vec4<f32>) -> vec4<f32>
{
    return vec4<f32>((textureSampleLevel((params_lut_texture_0), (params_lut_sampler_0), (ocio_rec709_to_acescct_0(color_1).xyz.xzy), (0.0f))).xyz, color_1.w);
}

struct Fragment_0
{
    @location(0) color_2 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) _S13 : vec3<f32>,
    @location(1) _S14 : vec2<f32>,
};

struct pixelInput_1
{
     coarseVertex_1 : CoarseVertex_0,
};

@fragment
fn fs_main( _S15 : pixelInput_0) -> Fragment_0
{
    var _S16 : pixelInput_1;
    _S16.coarseVertex_1._S1 = _S15._S13;
    _S16.coarseVertex_1._S2 = _S15._S14;
    var output_1 : Fragment_0;
    output_1.color_2 = tone_map_0(vec4<f32>((textureSampleLevel((params_hdr_texture_0), (params_hdr_sampler_0), (_S16.coarseVertex_1._S2), (0.0f))).xyz * vec3<f32>(params_0.view_uniform_0.exposure_linear_0), 1.0f));
    return output_1;
}

