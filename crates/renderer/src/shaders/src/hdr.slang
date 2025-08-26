import "modules/color.slang";

// Per-vertex attributes to be assembled from bound vertex buffers.
struct AssembledVertex
{
    uint    index    : SV_VertexID;
};

// Output of the vertex shader, and input to the fragment shader.
struct CoarseVertex
{
    float3 color;
    float2 uv;
};

// Output of the fragment shader
struct Fragment
{
    float4 color;
};

// Vertex  Shader

struct VertexStageOutput
{
    CoarseVertex    coarseVertex    : CoarseVertex;
    float4          sv_position     : SV_Position;
};

[shader("vertex")]
VertexStageOutput vs_main(
    AssembledVertex assembledVertex)
{
    VertexStageOutput output;

    uint idx = assembledVertex.index;

    // Generate a triangle that covers the whole screen
    var uv = float2(
        float((idx << 1u) & 2u),
        float(idx & 2u),
    );

    output.coarseVertex.color = float3(uv, 1.0);
    output.sv_position = float4(uv * 2.0 - 1.0, 0.0, 1.0);
    uv.y = 1.0 - uv.y;
    output.coarseVertex.uv = uv;

    return output;
}

struct ViewParameters {
    Texture2D<float4> hdr_texture;
    SamplerState hdr_sampler;

    Texture3D<float4> lut_texture;
    SamplerState lut_sampler;

    ViewUniform view_uniform;
}

struct ViewUniform {
    float exposure_linear;
}

ParameterBlock<ViewParameters> params;

float3 draw_lut(float2 uv) {
    var dim: uint3;
    params.lut_texture.GetDimensions(dim.x, dim.y, dim.z);
    let size = float(dim.x);
    let x_scaled = (uv.x * size);
    let x_z = float2(x_scaled % 1.0, floor(x_scaled) / size);
    let uvw = float3(x_z.x, uv.y, x_z.y);
    return params.lut_texture.SampleLevel(params.lut_sampler, uvw, 0.0).rgb;
}

float3 tone_map(float3 color) {
    let shaper_color = scene_linear_to_shaper_space(color);

    return params.lut_texture.SampleLevel(params.lut_sampler, shaper_color, 0.0).rgb;
}

// Fragment Shader
[shader("fragment")]
Fragment fs_main(
    CoarseVertex coarseVertex : CoarseVertex) : SV_Target
{
    float3 color = coarseVertex.color;
    float2 uv = coarseVertex.uv;

    float4 hdr_sample = params.hdr_texture.SampleLevel(params.hdr_sampler, uv, 0);
    var hdr_color = hdr_sample.xyz;

    // Debug View: Color Sweep
    if (false) {
        var settings: ColorSweepSettings;
        settings.ev_min = -8.0;
        settings.ev_max = 8.0;
        settings.ev_step = 0.25;
        settings.hue_min = 0.0;
        settings.hue_max = 360.0;
        settings.hue_step = 15.0;

        hdr_color = color_sweep(uv, settings);
    }

    hdr_color *= params.view_uniform.exposure_linear;

    var encoded_color = tone_map(hdr_color);

    // Debug View: Draw LUT
    if (false) {
        encoded_color = draw_lut(uv);
    }

    Fragment output;
    output.color = float4(encoded_color, 1.0);
    return output;
}
