module "sky";

import "../filament-brdf.slang";
import "bsdf-properties.slang";

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

public ParameterBlock<SkyParameters> sky_params;

public float3 irradianceSH(float3 n) {
    let result =
          sky_params.properties.sh_coefficients[0]
        + sky_params.properties.sh_coefficients[1] * (n.y)
        + sky_params.properties.sh_coefficients[2] * (n.z)
        + sky_params.properties.sh_coefficients[3] * (n.x)
        + sky_params.properties.sh_coefficients[4] * (n.y * n.x)
        + sky_params.properties.sh_coefficients[5] * (n.y * n.z)
        + sky_params.properties.sh_coefficients[6] * (3.0 * n.z * n.z - 1.0)
        + sky_params.properties.sh_coefficients[7] * (n.z * n.x)
        + sky_params.properties.sh_coefficients[8] * (n.x * n.x - n.y * n.y);

    return result.xyz;
}

float2 prefilteredDFG_LUT(float perceptualRoughness, float NoV) {
    // return dfgLut.SampleLevel(sampler, float2(NoV, perceptualRoughness), 0.0).rg;
    return float2(0.0);
}

float3 evaluateSpecularIBL(float3 r, float perceptualRoughness) {
    // This assumes a 256x256 cubemap, with 9 mip levels
    float lod = 8.0 * perceptualRoughness;
    // decodeEnvironmentMap() either decodes RGBM or is a no-op if the
    // cubemap is stored in a float texture
    // return decodeEnvironmentMap(textureCubeLodEXT(environmentMap, r, lod));
    return float3(0.0);
}

public float3 evaluateIBL(PixelProperties pixel) {
    let n = pixel.normal;
    let v = pixel.view;
    let diffuseColor = pixel.diffuseColor;
    let perceptualRoughness = pixel.perceptualRoughness;
    let f0 = pixel.minReflectance;
    const float f90 = 1.0;
    let NoV = max(dot(n, v), 0.0);
    let r = pixel.reflection;

    // Specular indirect
    float3 indirectSpecular = evaluateSpecularIBL(r, perceptualRoughness);
    float2 env = prefilteredDFG_LUT(perceptualRoughness, NoV);
    float3 specularColor = f0 * env.x + f90 * env.y;

    // horizon occlusion with falloff
    float horizon = min(1.0 + dot(r, pixel.vertexNormal), 1.0);
    indirectSpecular *= horizon * horizon;

    // Diffuse indirect
    // We multiply by the Lambertian BRDF to compute radiance from irradiance
    // With the Disney BRDF we would have to remove the Fresnel term that
    // depends on NoL (it would be rolled into the SH). The Lambertian BRDF
    // can be baked directly in the SH to save a multiplication here
    float3 indirectDiffuse = max(irradianceSH(n), 0.0) * Fd_Lambert();

    // Indirect contribution
    let skyContribution = diffuseColor * indirectDiffuse + indirectSpecular * specularColor;
    return skyContribution * sky_params.properties.exposure_linear;
}
