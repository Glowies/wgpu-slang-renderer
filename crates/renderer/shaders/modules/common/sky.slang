module "sky";

// Note: You CANNOT put the exposure_linear *before* the sh_coefficients
// array. This throws off the alignment of the array. (I guess each entry
// in this struct needs to be aligned to 16 bytes, but the exposure_linear
// f32 is only 4 bytes)
public struct SkyUniform {
    public float4 sh_coefficients[9];
    public float exposure_linear;
    public float debug_sh;
}

public struct SkyParameters {
  public TextureCube env_map_texture;
  public SamplerState env_map_sampler;
  public SkyUniform properties;
}

public float3 irradianceSH(float3 n, float4 sh_coefficients[9]) {
    let result =
          sh_coefficients[0]
        + sh_coefficients[1] * (n.y)
        + sh_coefficients[2] * (n.z)
        + sh_coefficients[3] * (n.x)
        + sh_coefficients[4] * (n.y * n.x)
        + sh_coefficients[5] * (n.y * n.z)
        + sh_coefficients[6] * (3.0 * n.z * n.z - 1.0)
        + sh_coefficients[7] * (n.z * n.x)
        + sh_coefficients[8] * (n.x * n.x - n.y * n.y);

    return result.xyz;
}
