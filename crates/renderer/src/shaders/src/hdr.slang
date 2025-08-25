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

Texture2D<float4> hdr_texture;

SamplerState hdr_sampler;

// Fragment Shader

[shader("fragment")]
Fragment fs_main(
    CoarseVertex coarseVertex : CoarseVertex) : SV_Target
{
    float3 color = coarseVertex.color;
    float2 uv = coarseVertex.uv;


    float4 hdr_sample = hdr_texture.SampleLevel(hdr_sampler, uv, 0);
    var hdr_color = hdr_sample.xyz;

    Fragment output;
    output.color = float4(hdr_color, 1.0);
    return output;
}
