import "modules/common/camera.slang";
import "modules/common/sky.slang";

uniform CameraUniform camera;

ParameterBlock<SkyParameters> sky_params;

struct VertexOutput {
    float4 frag_position : SV_Position;
    float4 clip_position;
}

[shader("vertex")]
VertexOutput vs_main(
    uint id: SV_VertexID,
) {
    let uv = float2(uint2(
        id & 1u,
        (id >> 1u) & 1u,
    ));
    var out: VertexOutput;

    // create triangle TWICE the size of the screen
    out.clip_position = float4(uv * 4.0 - 1.0, 1.0, 1.0);
    out.frag_position = float4(uv * 4.0 - 1.0, 1.0, 1.0);
    return out;
}

[shader("fragment")]
float4 fs_main(
    VertexOutput in,
) {
    // convert clip space to camera space
    let view_pos_homogeneous = mul(in.clip_position, camera.inv_proj);
    // undo perspective
    let view_ray_direction = view_pos_homogeneous.xyz / view_pos_homogeneous.w;

    // convert camera space to world space
    let ray_direction = normalize((mul(float4(view_ray_direction, 0.0), camera.inv_view)).xyz);

    var sample = sky_params.env_map_texture.Sample(sky_params.env_map_sampler, ray_direction);
    if (sky_params.properties.debug_sh > 0.5) {
        sample = float4(irradianceSH(ray_direction, sky_params.properties.sh_coefficients), 0.0);
    }

    return sample * sky_params.properties.exposure_linear;
}

