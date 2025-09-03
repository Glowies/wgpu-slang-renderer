import "modules/uniforms/camera.slang";
import "modules/uniforms/light.slang";

struct LightDebugParameters {
  CameraUniform camera;
  LightUniform light;
}

ParameterBlock<LightDebugParameters> params;

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
    out.clip_position = mul(params.camera.view_proj, float4(model.position * scale + params.light.position, 1.0));

    return out;
}

[shader("fragment")]
float4 fs_main(VertexOutput in) {
    return float4(params.light.color * params.light.intensity, 1.0);
}
