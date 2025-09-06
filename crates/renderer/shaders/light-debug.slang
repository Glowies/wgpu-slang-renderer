import "modules/common/camera.slang";
import "modules/common/light.slang";

ParameterBlock<CameraUniform> camera;
ParameterBlock<LightUniform> light;

struct VertexInput {
    float3 position;
    float2 tex_coords;
}

struct VertexOutput {
    float4 clip_position: SV_Position;
}

[shader("vertex")]
VertexOutput vs_main(
    VertexInput model,
){
    var out: VertexOutput;
    let scale = 0.25;
    out.clip_position = mul(float4(model.position * scale + light.position, 1.0), camera.view_proj);

    return out;
}

[shader("fragment")]
float4 fs_main(VertexOutput in) {
    return float4(light.color * light.intensity, 1.0);
}
