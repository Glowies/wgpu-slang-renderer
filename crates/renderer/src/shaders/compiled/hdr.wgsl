struct ViewUniform_std140_0
{
    @align(16) exposure_linear_0 : f32,
};

struct ViewParameters_std140_0
{
    @align(16) view_uniform_0 : ViewUniform_std140_0,
};

@binding(0) @group(0) var<uniform> params_0 : ViewParameters_std140_0;
@binding(3) @group(0) var params_lut_texture_0 : texture_3d<f32>;

@binding(4) @group(0) var params_lut_sampler_0 : sampler;

struct VertexStageOutput_0
{
    @location(0) color_0 : vec3<f32>,
    @location(1) uv_0 : vec2<f32>,
    @builtin(position) sv_position_0 : vec4<f32>,
};

var<private> PI_0 : f32;

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
    PI_0 = 3.14159274101257324f;
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

fn compute_max_saturation_0( a_0 : f32,  b_0 : f32) -> f32
{
    var k0_0 : f32;
    var k1_0 : f32;
    var k2_0 : f32;
    var k3_0 : f32;
    var k4_0 : f32;
    var wl_0 : f32;
    var wm_0 : f32;
    var ws_0 : f32;
    if((-1.88170325756072998f * a_0 - 0.809364914894104f * b_0) > 1.0f)
    {
        k0_0 = 1.19086277484893799f;
        k1_0 = 1.76576733589172363f;
        k2_0 = 0.5966264009475708f;
        k3_0 = 0.75515198707580566f;
        k4_0 = 0.56771242618560791f;
        wl_0 = 4.07674169540405273f;
        wm_0 = -3.30771160125732422f;
        ws_0 = 0.23096993565559387f;
    }
    else
    {
        if((1.81444108486175537f * a_0 - 1.19445276260375977f * b_0) > 1.0f)
        {
            k0_0 = 0.73956513404846191f;
            k1_0 = -0.45954403281211853f;
            k2_0 = 0.08285427093505859f;
            k3_0 = 0.12541070580482483f;
            k4_0 = 0.14503203332424164f;
            wl_0 = -1.26843798160552979f;
            wm_0 = 2.60975742340087891f;
            ws_0 = -0.34131938219070435f;
        }
        else
        {
            k0_0 = 1.35733652114868164f;
            k1_0 = -0.00915799010545015f;
            k2_0 = -1.15130209922790527f;
            k3_0 = -0.50559604167938232f;
            k4_0 = 0.00692166993394494f;
            wl_0 = -0.0041960864327848f;
            wm_0 = -0.70341861248016357f;
            ws_0 = 1.70761466026306152f;
        }
    }
    var S_0 : f32 = k0_0 + k1_0 * a_0 + k2_0 * b_0 + k3_0 * a_0 * a_0 + k4_0 * a_0 * b_0;
    var k_l_0 : f32 = 0.39633777737617493f * a_0 + 0.21580375730991364f * b_0;
    var k_m_0 : f32 = -0.10556134581565857f * a_0 - 0.06385417282581329f * b_0;
    var k_s_0 : f32 = -0.08948417752981186f * a_0 - 1.29148554801940918f * b_0;
    var l_0 : f32 = 1.0f + S_0 * k_l_0;
    var m_0 : f32 = 1.0f + S_0 * k_m_0;
    var s_0 : f32 = 1.0f + S_0 * k_s_0;
    var f_0 : f32 = wl_0 * (l_0 * l_0 * l_0) + wm_0 * (m_0 * m_0 * m_0) + ws_0 * (s_0 * s_0 * s_0);
    var f1_0 : f32 = wl_0 * (3.0f * k_l_0 * l_0 * l_0) + wm_0 * (3.0f * k_m_0 * m_0 * m_0) + ws_0 * (3.0f * k_s_0 * s_0 * s_0);
    return S_0 - f_0 * f1_0 / (f1_0 * f1_0 - 0.5f * f_0 * (wl_0 * (6.0f * k_l_0 * k_l_0 * l_0) + wm_0 * (6.0f * k_m_0 * k_m_0 * m_0) + ws_0 * (6.0f * k_s_0 * k_s_0 * s_0)));
}

fn oklab_to_lin_rec709_0( c_0 : vec3<f32>) -> vec3<f32>
{
    var _S7 : f32 = c_0.x;
    var _S8 : f32 = c_0.y;
    var _S9 : f32 = c_0.z;
    var l_1 : f32 = _S7 + 0.39633777737617493f * _S8 + 0.21580375730991364f * _S9;
    var m_1 : f32 = _S7 - 0.10556134581565857f * _S8 - 0.06385417282581329f * _S9;
    var s_1 : f32 = _S7 - 0.08948417752981186f * _S8 - 1.29148554801940918f * _S9;
    var l_2 : f32 = l_1 * l_1 * l_1;
    var m_2 : f32 = m_1 * m_1 * m_1;
    var s_2 : f32 = s_1 * s_1 * s_1;
    return vec3<f32>(4.07674169540405273f * l_2 - 3.30771160125732422f * m_2 + 0.23096993565559387f * s_2, -1.26843798160552979f * l_2 + 2.60975742340087891f * m_2 - 0.34131938219070435f * s_2, -0.0041960864327848f * l_2 - 0.70341861248016357f * m_2 + 1.70761466026306152f * s_2);
}

fn cbrt_0( in_0 : f32) -> f32
{
    return pow(in_0, 0.3333333432674408f);
}

fn find_cusp_0( a_1 : f32,  b_1 : f32) -> vec2<f32>
{
    var S_cusp_0 : f32 = compute_max_saturation_0(a_1, b_1);
    var _S10 : vec3<f32> = oklab_to_lin_rec709_0(vec3<f32>(1.0f, S_cusp_0 * a_1, S_cusp_0 * b_1));
    var L_cusp_0 : f32 = cbrt_0(1.0f / max(max(_S10.x, _S10.y), _S10.z));
    return vec2<f32>(L_cusp_0, L_cusp_0 * S_cusp_0);
}

fn to_ST_0( cusp_0 : vec2<f32>) -> vec2<f32>
{
    var L_0 : f32 = cusp_0.x;
    var C_0 : f32 = cusp_0.y;
    return vec2<f32>(C_0 / L_0, C_0 / (1.0f - L_0));
}

fn toe_inv_0( x_0 : f32) -> f32
{
    return (x_0 * x_0 + 0.20600000023841858f * x_0) / (1.17087376117706299f * (x_0 + 0.02999999932944775f));
}

fn fmax_0( x_1 : f32,  y_0 : f32) -> f32
{
    return max(x_1, y_0);
}

fn okhsv_to_lin_rec709_0( hsv_0 : vec3<f32>) -> vec3<f32>
{
    var h_0 : f32 = hsv_0.x;
    var s_3 : f32 = hsv_0.y;
    var v_0 : f32 = hsv_0.z;
    var a_2 : f32 = cos(2.0f * PI_0 * h_0);
    var b_2 : f32 = sin(2.0f * PI_0 * h_0);
    var ST_max_0 : vec2<f32> = to_ST_0(find_cusp_0(a_2, b_2));
    var T_max_0 : f32 = ST_max_0.y;
    var _S11 : f32 = 0.5f + T_max_0 - T_max_0 * (1.0f - 0.5f / ST_max_0.x) * s_3;
    var L_v_0 : f32 = 1.0f - s_3 * 0.5f / _S11;
    var C_v_0 : f32 = s_3 * T_max_0 * 0.5f / _S11;
    var L_1 : f32 = v_0 * L_v_0;
    var L_vt_0 : f32 = toe_inv_0(L_v_0);
    var C_vt_0 : f32 = C_v_0 * L_vt_0 / L_v_0;
    var L_new_0 : f32 = toe_inv_0(L_1);
    var _S12 : vec3<f32> = oklab_to_lin_rec709_0(vec3<f32>(L_vt_0, a_2 * C_vt_0, b_2 * C_vt_0));
    var scale_L_0 : f32 = cbrt_0(1.0f / fmax_0(fmax_0(_S12.x, _S12.y), fmax_0(_S12.z, 0.0f)));
    var C_1 : f32 = v_0 * C_v_0 * L_new_0 / L_1 * scale_L_0;
    return oklab_to_lin_rec709_0(vec3<f32>(L_new_0 * scale_L_0, C_1 * a_2, C_1 * b_2));
}

struct ColorSweepSettings_0
{
     ev_min_0 : f32,
     ev_max_0 : f32,
     ev_step_0 : f32,
     hue_min_0 : f32,
     hue_max_0 : f32,
     hue_step_0 : f32,
};

fn color_sweep_0( uv_2 : vec2<f32>,  settings_0 : ColorSweepSettings_0) -> vec3<f32>
{
    return okhsv_to_lin_rec709_0(vec3<f32>((settings_0.hue_min_0 + floor(uv_2.y * ((settings_0.hue_max_0 - settings_0.hue_min_0) / settings_0.hue_step_0)) * settings_0.hue_step_0) / 360.0f, 1.0f, 1.0f)) * vec3<f32>(exp2(settings_0.ev_min_0 + floor(uv_2.x * ((settings_0.ev_max_0 - settings_0.ev_min_0) / settings_0.ev_step_0)) * settings_0.ev_step_0));
}

fn ocio_rec709_to_acescct_0( inPixel_0 : vec4<f32>) -> vec4<f32>
{
    var outColor_0 : vec4<f32> = inPixel_0;
    var _S13 : vec3<f32> = inPixel_0.xyz;
    var res_0 : vec4<f32> = (((mat4x4<f32>(0.6130974292755127f, 0.07019372284412384f, 0.0206155925989151f, 0.0f, 0.33952313661575317f, 0.91635388135910034f, 0.10956977307796478f, 0.0f, 0.04737945273518562f, 0.01345239859074354f, 0.86981463432312012f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f)) * (vec4<f32>(_S13.x, _S13.y, _S13.z, inPixel_0.w))));
    var _S14 : vec3<f32> = vec3<f32>(res_0.x, res_0.y, res_0.z);
    outColor_0.x = _S14.x;
    outColor_0.y = _S14.y;
    outColor_0.z = _S14.z;
    outColor_0[i32(3)] = res_0.w;
    const minValue_0 : vec3<f32> = vec3<f32>(1.17549435082228751e-38f, 1.17549435082228751e-38f, 1.17549435082228751e-38f);
    const linear_segment_slope_0 : vec3<f32> = vec3<f32>(10.5402374267578125f, 10.5402374267578125f, 10.5402374267578125f);
    const linear_segment_offset_0 : vec3<f32> = vec3<f32>(0.07290557026863098f, 0.07290557026863098f, 0.07290557026863098f);
    const log_slope_0 : vec3<f32> = vec3<f32>(0.08234560489654541f, 0.08234560489654541f, 0.08234560489654541f);
    const log_offset_0 : vec3<f32> = vec3<f32>(0.5547945499420166f, 0.5547945499420166f, 0.5547945499420166f);
    var _S15 : f32;
    if((outColor_0.xyz[i32(0)]) > 0.0078125f)
    {
        _S15 = 1.0f;
    }
    else
    {
        _S15 = 0.0f;
    }
    var _S16 : f32;
    if((outColor_0.xyz[i32(1)]) > 0.0078125f)
    {
        _S16 = 1.0f;
    }
    else
    {
        _S16 = 0.0f;
    }
    var _S17 : f32;
    if((outColor_0.xyz[i32(2)]) > 0.0078125f)
    {
        _S17 = 1.0f;
    }
    else
    {
        _S17 = 0.0f;
    }
    var isAboveBreak_0 : vec3<f32> = vec3<f32>(_S15, _S16, _S17);
    var _S18 : vec3<f32> = isAboveBreak_0 * (log_slope_0 * log(max(minValue_0, outColor_0.xyz)) + log_offset_0) + (vec3<f32>(1.0f, 1.0f, 1.0f) - isAboveBreak_0) * (outColor_0.xyz * linear_segment_slope_0 + linear_segment_offset_0);
    outColor_0.x = _S18.x;
    outColor_0.y = _S18.y;
    outColor_0.z = _S18.z;
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
    @location(0) _S19 : vec3<f32>,
    @location(1) _S20 : vec2<f32>,
};

struct pixelInput_1
{
     coarseVertex_1 : CoarseVertex_0,
};

@fragment
fn fs_main( _S21 : pixelInput_0) -> Fragment_0
{
    PI_0 = 3.14159274101257324f;
    var _S22 : pixelInput_1;
    _S22.coarseVertex_1._S1 = _S21._S19;
    _S22.coarseVertex_1._S2 = _S21._S20;
    var settings_1 : ColorSweepSettings_0;
    settings_1.ev_min_0 = -8.0f;
    settings_1.ev_max_0 = 8.0f;
    settings_1.ev_step_0 = 0.25f;
    settings_1.hue_min_0 = 0.0f;
    settings_1.hue_max_0 = 360.0f;
    settings_1.hue_step_0 = 15.0f;
    var output_1 : Fragment_0;
    output_1.color_2 = tone_map_0(vec4<f32>(color_sweep_0(_S22.coarseVertex_1._S2, settings_1) * vec3<f32>(params_0.view_uniform_0.exposure_linear_0), 1.0f));
    return output_1;
}

