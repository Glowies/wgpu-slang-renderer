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
    public float mip_count;
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

// Source: https://knarkowicz.wordpress.com/2014/12/27/analytical-dfg-term-for-ibl/
float3 envDFGPolynomial(float3 f0, float roughness, float ndotv) {
    float gloss_4rt = 1.0 - roughness; 
    float gloss = gloss_4rt * gloss_4rt * gloss_4rt * gloss_4rt;
    float x = gloss;
    float y = ndotv;
 
    float b1 = -0.1688;
    float b2 = 1.895;
    float b3 = 0.9903;
    float b4 = -4.853;
    float b5 = 8.404;
    float b6 = -5.069;
    float bias = saturate( min( b1 * x + b2 * x * x, b3 + b4 * y + b5 * y * y + b6 * y * y * y ) );
 
    float d0 = 0.6045;
    float d1 = 1.699;
    float d2 = -0.5228;
    float d3 = -3.603;
    float d4 = 1.404;
    float d5 = 0.1939;
    float d6 = 2.661;
    float delta = saturate( d0 + d1 * x + d2 * y + d3 * x * x + d4 * x * y + d5 * y * y + d6 * x * x * x );
    float scale = delta - bias;
 
    bias *= saturate( 50.0 * f0.y );
    return f0 * scale + bias;
}

float3 evaluateSpecularIBL(float3 r, float perceptualRoughness) {
    // This assumes a cubemap with 2^(n) pixels per edge with max number of mips generated
    float n = sky_params.properties.mip_count - 1.0;
    float lod = n * perceptualRoughness;
    return sky_params.env_map_texture.SampleLevel(sky_params.env_map_sampler, r, lod).rgb;
}

public float3 evaluateIBL(PixelProperties pixel) {
    let n = pixel.normal;
    let v = pixel.view;
    let diffuseColor = pixel.diffuseColor;
    let perceptualRoughness = pixel.perceptualRoughness;
    let roughness = pixel.roughness;
    let f0 = pixel.minReflectance;
    let NoV = max(dot(n, v), 0.0);
    let r = pixel.reflection;

    // Specular indirect
    float3 indirectSpecular = evaluateSpecularIBL(r, perceptualRoughness);
    float3 specularColor = envDFGPolynomial(f0, roughness, NoV);

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
