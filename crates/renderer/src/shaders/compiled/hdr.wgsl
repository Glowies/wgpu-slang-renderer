struct ViewUniform_std140_0
{
    @align(16) exposure_linear_0 : f32,
};

struct ViewParameters_std140_0
{
    @align(16) view_uniform_0 : ViewUniform_std140_0,
};

@binding(0) @group(0) var<uniform> params_0 : ViewParameters_std140_0;
const ocio_grading_rgbcurve_coefs_1_0 : array<f32, i32(42)> = array<f32, i32(42)>( 0.52177268266677856f, 0.06544870883226395f, 0.27260473370552063f, 0.1239112913608551f, 0.0858645960688591f, -0.01711625047028065f, 0.03384167328476906f, -0.19483496248722076f, -0.2016889750957489f, -0.47698327898979187f, -0.2760046124458313f, -0.13913913071155548f, -0.09226308763027191f, -0.06659094989299774f, 0.0f, 0.48030880093574524f, 0.54055649042129517f, 0.79149812459945679f, 0.90556252002716064f, 0.98460370302200317f, 0.96884763240814209f, 1.0f, 0.87078344821929932f, 0.73702126741409302f, 0.42068111896514893f, 0.23763206601142883f, 0.14535361528396606f, 0.08416377753019333f, -1.69896996021270752f, -1.58843505382537842f, -1.35350000858306885f, -1.04694998264312744f, -0.65640002489089966f, -0.22141000628471375f, 0.22814401984214783f, 0.68124121427536011f, 0.99142187833786011f, 1.25800001621246338f, 1.44994997978210449f, 1.55910003185272217f, 1.62259995937347412f, 1.66065454483032227f );
const ocio_grading_rgbcurve_knots_1_0 : array<f32, i32(15)> = array<f32, i32(15)>( -2.54062366485595703f, -2.08035731315612793f, -1.62009084224700928f, -1.15982437133789062f, -0.69955801963806152f, -0.23929157853126526f, 0.22097483277320862f, 0.68124121427536011f, 1.01284635066986084f, 1.3444514274597168f, 1.67605650424957275f, 2.00766158103942871f, 2.33926653861999512f, 2.67087173461914062f, 3.00247669219970703f );
const ocio_grading_rgbcurve_coefsOffsets_1_0 : array<i32, i32(8)> = array<i32, i32(8)>( i32(-1), i32(0), i32(-1), i32(0), i32(-1), i32(0), i32(0), i32(42) );
const ocio_grading_rgbcurve_knotsOffsets_1_0 : array<i32, i32(8)> = array<i32, i32(8)>( i32(-1), i32(0), i32(-1), i32(0), i32(-1), i32(0), i32(0), i32(15) );
const ocio_grading_rgbcurve_coefs_0_0 : array<f32, i32(24)> = array<f32, i32(24)>( 0.18597044050693512f, 0.40377888083457947f, -0.07485050708055496f, -0.18583370745182037f, -0.19212943315505981f, -0.19314683973789215f, -0.05010509490966797f, -0.05112241953611374f, 0.0f, 0.55982685089111328f, 1.77532243728637695f, 1.54999995231628418f, 0.87870168685913086f, 0.53122317790985107f, 0.18282587826251984f, 0.09187229722738266f, -4.0f, -3.57868838310241699f, -1.82131326198577881f, 0.68124121427536011f, 2.87457752227783203f, 3.51206254959106445f, 3.83406209945678711f, 3.95872402191162109f );
const ocio_grading_rgbcurve_knots_0_0 : array<f32, i32(9)> = array<f32, i32(9)>( -5.2601776123046875f, -3.75502753257751465f, -2.2498774528503418f, -0.7447274923324585f, 1.06145250797271729f, 1.96573483943939209f, 2.86763238906860352f, 3.77526044845581055f, 4.6738123893737793f );
const ocio_grading_rgbcurve_coefsOffsets_0_0 : array<i32, i32(8)> = array<i32, i32(8)>( i32(-1), i32(0), i32(-1), i32(0), i32(-1), i32(0), i32(0), i32(24) );
const ocio_grading_rgbcurve_knotsOffsets_0_0 : array<i32, i32(8)> = array<i32, i32(8)>( i32(-1), i32(0), i32(-1), i32(0), i32(-1), i32(0), i32(0), i32(9) );
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

fn hsv2rgb_0( c_0 : vec3<f32>) -> vec3<f32>
{
    const _S7 : vec3<f32> = vec3<f32>(1.0f, 1.0f, 1.0f);
    return vec3<f32>(c_0.z) * mix(_S7, saturate(abs(fract(c_0.xxx + vec3<f32>(1.0f, 0.66666668653488159f, 0.3333333432674408f)) * vec3<f32>(6.0f) - vec3<f32>(3.0f, 3.0f, 3.0f)) - _S7), vec3<f32>(c_0.y));
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
    return hsv2rgb_0(vec3<f32>((settings_0.hue_min_0 + floor(uv_2.y * ((settings_0.hue_max_0 - settings_0.hue_min_0) / settings_0.hue_step_0)) * settings_0.hue_step_0) / 360.0f, 1.0f, exp2(settings_0.ev_min_0 + floor(uv_2.x * ((settings_0.ev_max_0 - settings_0.ev_min_0) / settings_0.ev_step_0)) * settings_0.ev_step_0)));
}

fn ocio_grading_rgbcurve_evalBSplineCurve_0_0( curveIdx_0 : i32,  x_0 : f32) -> f32
{
    var _S8 : i32 = curveIdx_0 * i32(2);
    var _S9 : i32 = _S8 + i32(1);
    var coefsSets_0 : i32 = ocio_grading_rgbcurve_coefsOffsets_0_0[_S9] / i32(3);
    if(coefsSets_0 == i32(0))
    {
        return x_0;
    }
    var _S10 : i32 = ocio_grading_rgbcurve_knotsOffsets_0_0[_S8] + ocio_grading_rgbcurve_knotsOffsets_0_0[_S9];
    var _S11 : i32 = _S10 - i32(1);
    if(x_0 <= (ocio_grading_rgbcurve_knots_0_0[ocio_grading_rgbcurve_knotsOffsets_0_0[_S8]]))
    {
        return (x_0 - ocio_grading_rgbcurve_knots_0_0[ocio_grading_rgbcurve_knotsOffsets_0_0[_S8]]) * ocio_grading_rgbcurve_coefs_0_0[ocio_grading_rgbcurve_coefsOffsets_0_0[_S8] + coefsSets_0] + ocio_grading_rgbcurve_coefs_0_0[ocio_grading_rgbcurve_coefsOffsets_0_0[_S8] + coefsSets_0 * i32(2)];
    }
    else
    {
        if(x_0 >= (ocio_grading_rgbcurve_knots_0_0[_S11]))
        {
            var _S12 : i32 = ocio_grading_rgbcurve_coefsOffsets_0_0[_S8] + coefsSets_0 - i32(1);
            var _S13 : i32 = ocio_grading_rgbcurve_coefsOffsets_0_0[_S8] + coefsSets_0 * i32(2) - i32(1);
            var t_0 : f32 = ocio_grading_rgbcurve_knots_0_0[_S11] - ocio_grading_rgbcurve_knots_0_0[_S10 - i32(2)];
            return (x_0 - ocio_grading_rgbcurve_knots_0_0[_S11]) * (2.0f * ocio_grading_rgbcurve_coefs_0_0[_S12] * t_0 + ocio_grading_rgbcurve_coefs_0_0[_S13]) + ((ocio_grading_rgbcurve_coefs_0_0[_S12] * t_0 + ocio_grading_rgbcurve_coefs_0_0[_S13]) * t_0 + ocio_grading_rgbcurve_coefs_0_0[ocio_grading_rgbcurve_coefsOffsets_0_0[_S8] + coefsSets_0 * i32(3) - i32(1)]);
        }
    }
    var i_0 : i32 = i32(0);
    for(;;)
    {
        if(i_0 < (ocio_grading_rgbcurve_knotsOffsets_0_0[_S9] - i32(2)))
        {
        }
        else
        {
            break;
        }
        if(x_0 < (ocio_grading_rgbcurve_knots_0_0[ocio_grading_rgbcurve_knotsOffsets_0_0[_S8] + i_0 + i32(1)]))
        {
            break;
        }
        i_0 = i_0 + i32(1);
    }
    var t_1 : f32 = x_0 - ocio_grading_rgbcurve_knots_0_0[ocio_grading_rgbcurve_knotsOffsets_0_0[_S8] + i_0];
    return (ocio_grading_rgbcurve_coefs_0_0[ocio_grading_rgbcurve_coefsOffsets_0_0[_S8] + i_0] * t_1 + ocio_grading_rgbcurve_coefs_0_0[ocio_grading_rgbcurve_coefsOffsets_0_0[_S8] + coefsSets_0 + i_0]) * t_1 + ocio_grading_rgbcurve_coefs_0_0[ocio_grading_rgbcurve_coefsOffsets_0_0[_S8] + coefsSets_0 * i32(2) + i_0];
}

fn ocio_grading_rgbcurve_evalBSplineCurve_1_0( curveIdx_1 : i32,  x_1 : f32) -> f32
{
    var _S14 : i32 = curveIdx_1 * i32(2);
    var _S15 : i32 = _S14 + i32(1);
    var coefsSets_1 : i32 = ocio_grading_rgbcurve_coefsOffsets_1_0[_S15] / i32(3);
    if(coefsSets_1 == i32(0))
    {
        return x_1;
    }
    var _S16 : i32 = ocio_grading_rgbcurve_knotsOffsets_1_0[_S14] + ocio_grading_rgbcurve_knotsOffsets_1_0[_S15];
    var _S17 : i32 = _S16 - i32(1);
    if(x_1 <= (ocio_grading_rgbcurve_knots_1_0[ocio_grading_rgbcurve_knotsOffsets_1_0[_S14]]))
    {
        return (x_1 - ocio_grading_rgbcurve_knots_1_0[ocio_grading_rgbcurve_knotsOffsets_1_0[_S14]]) * ocio_grading_rgbcurve_coefs_1_0[ocio_grading_rgbcurve_coefsOffsets_1_0[_S14] + coefsSets_1] + ocio_grading_rgbcurve_coefs_1_0[ocio_grading_rgbcurve_coefsOffsets_1_0[_S14] + coefsSets_1 * i32(2)];
    }
    else
    {
        if(x_1 >= (ocio_grading_rgbcurve_knots_1_0[_S17]))
        {
            var _S18 : i32 = ocio_grading_rgbcurve_coefsOffsets_1_0[_S14] + coefsSets_1 - i32(1);
            var _S19 : i32 = ocio_grading_rgbcurve_coefsOffsets_1_0[_S14] + coefsSets_1 * i32(2) - i32(1);
            var t_2 : f32 = ocio_grading_rgbcurve_knots_1_0[_S17] - ocio_grading_rgbcurve_knots_1_0[_S16 - i32(2)];
            return (x_1 - ocio_grading_rgbcurve_knots_1_0[_S17]) * (2.0f * ocio_grading_rgbcurve_coefs_1_0[_S18] * t_2 + ocio_grading_rgbcurve_coefs_1_0[_S19]) + ((ocio_grading_rgbcurve_coefs_1_0[_S18] * t_2 + ocio_grading_rgbcurve_coefs_1_0[_S19]) * t_2 + ocio_grading_rgbcurve_coefs_1_0[ocio_grading_rgbcurve_coefsOffsets_1_0[_S14] + coefsSets_1 * i32(3) - i32(1)]);
        }
    }
    var i_1 : i32 = i32(0);
    for(;;)
    {
        if(i_1 < (ocio_grading_rgbcurve_knotsOffsets_1_0[_S15] - i32(2)))
        {
        }
        else
        {
            break;
        }
        if(x_1 < (ocio_grading_rgbcurve_knots_1_0[ocio_grading_rgbcurve_knotsOffsets_1_0[_S14] + i_1 + i32(1)]))
        {
            break;
        }
        i_1 = i_1 + i32(1);
    }
    var t_3 : f32 = x_1 - ocio_grading_rgbcurve_knots_1_0[ocio_grading_rgbcurve_knotsOffsets_1_0[_S14] + i_1];
    return (ocio_grading_rgbcurve_coefs_1_0[ocio_grading_rgbcurve_coefsOffsets_1_0[_S14] + i_1] * t_3 + ocio_grading_rgbcurve_coefs_1_0[ocio_grading_rgbcurve_coefsOffsets_1_0[_S14] + coefsSets_1 + i_1]) * t_3 + ocio_grading_rgbcurve_coefs_1_0[ocio_grading_rgbcurve_coefsOffsets_1_0[_S14] + coefsSets_1 * i32(2) + i_1];
}

fn ocio_display_view_transform_0( inPixel_0 : vec4<f32>) -> vec4<f32>
{
    var outColor_0 : vec4<f32> = inPixel_0;
    var _S20 : vec3<f32> = inPixel_0.xyz;
    var res_0 : vec4<f32> = (((mat4x4<f32>(0.43963298201560974f, 0.08977644145488739f, 0.01754117012023926f, 0.0f, 0.38298869132995605f, 0.81343942880630493f, 0.11154655367136002f, 0.0f, 0.1773783266544342f, 0.09678412973880768f, 0.87091225385665894f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f)) * (vec4<f32>(_S20.x, _S20.y, _S20.z, inPixel_0.w))));
    var _S21 : vec3<f32> = vec3<f32>(res_0.x, res_0.y, res_0.z);
    outColor_0.x = _S21.x;
    outColor_0.y = _S21.y;
    outColor_0.z = _S21.z;
    outColor_0[i32(3)] = res_0.w;
    var YC_0 : f32 = (outColor_0.xyz.z + outColor_0.xyz.y + outColor_0.xyz.x + 1.75f * sqrt(outColor_0.xyz.z * (outColor_0.xyz.z - outColor_0.xyz.y) + outColor_0.xyz.y * (outColor_0.xyz.y - outColor_0.xyz.x) + outColor_0.xyz.x * (outColor_0.xyz.x - outColor_0.xyz.z))) / 3.0f;
    var maxval_0 : f32 = max(outColor_0.xyz.x, max(outColor_0.xyz.y, outColor_0.xyz.z));
    var x_2 : f32 = ((max(1.00000001335143196e-10f, maxval_0) - max(1.00000001335143196e-10f, min(outColor_0.xyz.x, min(outColor_0.xyz.y, outColor_0.xyz.z)))) / max(0.00999999977648258f, maxval_0) - 0.40000000596046448f) * 5.0f;
    var t_4 : f32 = max(0.0f, 1.0f - 0.5f * abs(x_2));
    var GlowGain_0 : f32 = 0.05000000074505806f * (0.5f * (1.0f + f32(sign(x_2)) * (1.0f - t_4 * t_4)));
    var _S22 : vec3<f32> = outColor_0.xyz * vec3<f32>(mix(mix(GlowGain_0, GlowGain_0 * (0.07999999821186066f / YC_0 - 0.5f), f32(YC_0 > 0.05333333089947701f)), 0.0f, f32(YC_0 > 0.15999999642372131f))) + outColor_0.xyz;
    outColor_0.x = _S22.x;
    outColor_0.y = _S22.y;
    outColor_0.z = _S22.z;
    var knot_coord_0 : f32 = clamp(2.0f + atan2(1.73205077648162842f * (outColor_0.xyz.y - outColor_0.xyz.z), 2.0f * outColor_0.xyz.x - (outColor_0.xyz.y + outColor_0.xyz.z)) * 1.69765269756317139f, 0.0f, 4.0f);
    var j_0 : i32 = i32(min(knot_coord_0, 3.0f));
    var t_5 : f32 = knot_coord_0 - f32(j_0);
    var _S23 : f32 = t_5 * t_5;
    var maxval_1 : f32 = max(outColor_0.xyz.x, max(outColor_0.xyz.y, outColor_0.xyz.z));
    outColor_0[i32(1)] = outColor_0.xyz.x + dot(mix(mix(mix(vec4<f32>(0.25f, 0.0f, 0.0f, 0.0f), vec4<f32>(-0.75f, 0.75f, 0.75f, 0.25f), vec4<f32>(f32(j_0 == i32(1)))), vec4<f32>(0.75f, -1.5f, 0.0f, 1.0f), vec4<f32>(f32(j_0 == i32(2)))), vec4<f32>(-0.25f, 0.75f, -0.75f, 0.25f), vec4<f32>(f32(j_0 == i32(3)))), vec4<f32>(_S23 * t_5, _S23, t_5, 1.0f)) * ((max(1.00000001335143196e-10f, maxval_1) - max(1.00000001335143196e-10f, min(outColor_0.xyz.x, min(outColor_0.xyz.y, outColor_0.xyz.z)))) / max(0.00999999977648258f, maxval_1)) * (0.02999999932944775f - outColor_0.xyz.x) * 0.18000000715255737f;
    const _S24 : vec3<f32> = vec3<f32>(0.0f, 0.0f, 0.0f);
    var _S25 : vec3<f32> = max(_S24, outColor_0.xyz);
    outColor_0.x = _S25.x;
    outColor_0.y = _S25.y;
    outColor_0.z = _S25.z;
    var res_1 : vec4<f32> = (((mat4x4<f32>(1.4514392614364624f, -0.07655377686023712f, 0.00831614807248116f, 0.0f, -0.2365107536315918f, 1.17622971534729004f, -0.0060324496589601f, 0.0f, -0.21492856740951538f, -0.09967592358589172f, 0.99771630764007568f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f)) * (vec4<f32>(outColor_0.xyz.x, outColor_0.xyz.y, outColor_0.xyz.z, outColor_0.w))));
    var _S26 : vec3<f32> = vec3<f32>(res_1.x, res_1.y, res_1.z);
    outColor_0.x = _S26.x;
    outColor_0.y = _S26.y;
    outColor_0.z = _S26.z;
    outColor_0[i32(3)] = res_1.w;
    var _S27 : vec3<f32> = max(_S24, outColor_0.xyz);
    outColor_0.x = _S27.x;
    outColor_0.y = _S27.y;
    outColor_0.z = _S27.z;
    var res_2 : vec4<f32> = (((mat4x4<f32>(0.97088915109634399f, 0.01088914833962917f, 0.01088914833962917f, 0.0f, 0.02696327120065689f, 0.98696327209472656f, 0.02696327120065689f, 0.0f, 0.00214758072979748f, 0.00214758072979748f, 0.96214759349822998f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f)) * (vec4<f32>(outColor_0.xyz.x, outColor_0.xyz.y, outColor_0.xyz.z, outColor_0.w))));
    var _S28 : vec3<f32> = vec3<f32>(res_2.x, res_2.y, res_2.z);
    outColor_0.x = _S28.x;
    outColor_0.y = _S28.y;
    outColor_0.z = _S28.z;
    outColor_0[i32(3)] = res_2.w;
    var _S29 : vec3<f32> = max(vec3<f32>(1.17549435082228751e-38f, 1.17549435082228751e-38f, 1.17549435082228751e-38f), outColor_0.xyz);
    outColor_0.x = _S29.x;
    outColor_0.y = _S29.y;
    outColor_0.z = _S29.z;
    var _S30 : vec3<f32> = log(outColor_0.xyz) * vec3<f32>(0.43429446220397949f, 0.43429446220397949f, 0.43429446220397949f);
    outColor_0.x = _S30.x;
    outColor_0.y = _S30.y;
    outColor_0.z = _S30.z;
    outColor_0[i32(1)] = ocio_grading_rgbcurve_evalBSplineCurve_0_0(i32(0), outColor_0.xyz.x);
    outColor_0[i32(1)] = ocio_grading_rgbcurve_evalBSplineCurve_0_0(i32(1), outColor_0.xyz.y);
    outColor_0[i32(1)] = ocio_grading_rgbcurve_evalBSplineCurve_0_0(i32(2), outColor_0.xyz.z);
    outColor_0[i32(1)] = ocio_grading_rgbcurve_evalBSplineCurve_0_0(i32(3), outColor_0.xyz.x);
    outColor_0[i32(1)] = ocio_grading_rgbcurve_evalBSplineCurve_0_0(i32(3), outColor_0.xyz.y);
    outColor_0[i32(1)] = ocio_grading_rgbcurve_evalBSplineCurve_0_0(i32(3), outColor_0.xyz.z);
    outColor_0[i32(1)] = ocio_grading_rgbcurve_evalBSplineCurve_1_0(i32(0), outColor_0.xyz.x);
    outColor_0[i32(1)] = ocio_grading_rgbcurve_evalBSplineCurve_1_0(i32(1), outColor_0.xyz.y);
    outColor_0[i32(1)] = ocio_grading_rgbcurve_evalBSplineCurve_1_0(i32(2), outColor_0.xyz.z);
    outColor_0[i32(1)] = ocio_grading_rgbcurve_evalBSplineCurve_1_0(i32(3), outColor_0.xyz.x);
    outColor_0[i32(1)] = ocio_grading_rgbcurve_evalBSplineCurve_1_0(i32(3), outColor_0.xyz.y);
    outColor_0[i32(1)] = ocio_grading_rgbcurve_evalBSplineCurve_1_0(i32(3), outColor_0.xyz.z);
    var _S31 : vec3<f32> = pow(vec3<f32>(10.0f, 10.0f, 10.0f), outColor_0.xyz);
    outColor_0.x = _S31.x;
    outColor_0.y = _S31.y;
    outColor_0.z = _S31.z;
    var res_3 : vec4<f32> = vec4<f32>(-0.00041684033931233f, -0.00041684033931233f, -0.00041684033931233f, 0.0f) + vec4<f32>(0.02084201760590076f, 0.02084201760590076f, 0.02084201760590076f, 1.0f) * vec4<f32>(outColor_0.xyz.x, outColor_0.xyz.y, outColor_0.xyz.z, outColor_0.w);
    var _S32 : vec3<f32> = vec3<f32>(res_3.x, res_3.y, res_3.z);
    outColor_0.x = _S32.x;
    outColor_0.y = _S32.y;
    outColor_0.z = _S32.z;
    outColor_0[i32(3)] = res_3.w;
    var _S33 : vec3<f32> = outColor_0.xyz * vec3<f32>(pow(max(1.00000001335143196e-10f, 0.27222871780395508f * outColor_0.xyz.x + 0.67408174276351929f * outColor_0.xyz.y + 0.05368951708078384f * outColor_0.xyz.z), -0.01889997720718384f));
    outColor_0.x = _S33.x;
    outColor_0.y = _S33.y;
    outColor_0.z = _S33.z;
    var res_4 : vec4<f32> = (((mat4x4<f32>(1.60475337505340576f, -0.10208246111869812f, -0.00326711172237992f, 0.0f, -0.53108096122741699f, 1.10813415050506592f, -0.07275542616844177f, 0.0f, -0.07367248833179474f, -0.00605167029425502f, 1.07602250576019287f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f)) * (vec4<f32>(outColor_0.xyz.x, outColor_0.xyz.y, outColor_0.xyz.z, outColor_0.w))));
    var _S34 : vec3<f32> = vec3<f32>(res_4.x, res_4.y, res_4.z);
    outColor_0.x = _S34.x;
    outColor_0.y = _S34.y;
    outColor_0.z = _S34.z;
    outColor_0[i32(3)] = res_4.w;
    const slope_0 : vec4<f32> = vec4<f32>(12.92321014404296875f, 12.92321014404296875f, 12.92321014404296875f, 1.0f);
    const scale_0 : vec4<f32> = vec4<f32>(1.0549999475479126f, 1.0549999475479126f, 1.0549999475479126f, 1.00000095367431641f);
    const offset_0 : vec4<f32> = vec4<f32>(0.05499999970197678f, 0.05499999970197678f, 0.05499999970197678f, 9.99999997475242708e-07f);
    const gamma_0 : vec4<f32> = vec4<f32>(0.4166666567325592f, 0.4166666567325592f, 0.4166666567325592f, 0.99999898672103882f);
    var _S35 : f32;
    if((outColor_0[i32(0)]) > 0.00303993467241526f)
    {
        _S35 = 1.0f;
    }
    else
    {
        _S35 = 0.0f;
    }
    var _S36 : f32;
    if((outColor_0[i32(1)]) > 0.00303993467241526f)
    {
        _S36 = 1.0f;
    }
    else
    {
        _S36 = 0.0f;
    }
    var _S37 : f32;
    if((outColor_0[i32(2)]) > 0.00303993467241526f)
    {
        _S37 = 1.0f;
    }
    else
    {
        _S37 = 0.0f;
    }
    var _S38 : f32;
    if((outColor_0[i32(3)]) > 1.0f)
    {
        _S38 = 1.0f;
    }
    else
    {
        _S38 = 0.0f;
    }
    var isAboveBreak_0 : vec4<f32> = vec4<f32>(_S35, _S36, _S37, _S38);
    var res_5 : vec4<f32> = isAboveBreak_0 * (pow(max(vec4<f32>(0.0f, 0.0f, 0.0f, 0.0f), outColor_0), gamma_0) * scale_0 - offset_0) + (vec4<f32>(1.0f, 1.0f, 1.0f, 1.0f) - isAboveBreak_0) * (outColor_0 * slope_0);
    var _S39 : vec3<f32> = vec3<f32>(res_5.x, res_5.y, res_5.z);
    outColor_0.x = _S39.x;
    outColor_0.y = _S39.y;
    outColor_0.z = _S39.z;
    outColor_0[i32(3)] = res_5.w;
    return outColor_0;
}

struct Fragment_0
{
    @location(0) color_1 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) _S40 : vec3<f32>,
    @location(1) _S41 : vec2<f32>,
};

struct pixelInput_1
{
     coarseVertex_1 : CoarseVertex_0,
};

@fragment
fn fs_main( _S42 : pixelInput_0) -> Fragment_0
{
    var _S43 : pixelInput_1;
    _S43.coarseVertex_1._S1 = _S42._S40;
    _S43.coarseVertex_1._S2 = _S42._S41;
    var settings_1 : ColorSweepSettings_0;
    settings_1.ev_min_0 = -8.0f;
    settings_1.ev_max_0 = 8.0f;
    settings_1.ev_step_0 = 0.25f;
    settings_1.hue_min_0 = 0.0f;
    settings_1.hue_max_0 = 360.0f;
    settings_1.hue_step_0 = 15.0f;
    var output_1 : Fragment_0;
    output_1.color_1 = ocio_display_view_transform_0(vec4<f32>(color_sweep_0(_S43.coarseVertex_1._S2, settings_1) * vec3<f32>(params_0.view_uniform_0.exposure_linear_0), 1.0f));
    return output_1;
}

