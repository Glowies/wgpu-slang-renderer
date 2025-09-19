import "modules/common/camera.slang";
import "modules/common/light.slang";
import "modules/math.slang";

ParameterBlock<CameraUniform> camera;
ParameterBlock<LightUniform> light;

struct VertexInput {
    uint vertexID: SV_VertexID;
}

struct VertexOutput {
    float4 clip_position: SV_Position;
}

[shader("vertex")]
VertexOutput vs_main(
    VertexInput vertexIn,
){
    var out: VertexOutput;

    
    let vertexCount = 32.0; // needs to match VERTEX_COUNT in light.rs
    let angle = vertexIn.vertexID * 2.0 * PI / (vertexCount - 1.0);
    let radius = .32;
    let toCenter = sign(float(vertexIn.vertexID) % 3.0);
    let offsetView = float4(cos(angle), sin(angle), 0.0, 0.0) * toCenter;
    let offsetWorld = mul(offsetView, camera.inv_view) * radius;

    
    let position = float4(light.position + offsetWorld.xyz, 1.0);
    out.clip_position = mul(position, camera.view_proj);

    return out;
}

[shader("fragment")]
float4 fs_main(VertexOutput in) {
    return float4(light.color * light.intensity, 1.0);
}
