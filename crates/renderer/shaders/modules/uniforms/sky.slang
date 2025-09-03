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
