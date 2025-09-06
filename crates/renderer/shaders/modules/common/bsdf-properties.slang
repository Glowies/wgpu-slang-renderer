module "bsdf-properties";

public struct LightProperties {
    public float3 direction; 
    public float3 color;
    public float3 posToLight;
    public float intensity;
    public float invRadius;
    public float innerAngle;
    public float outerAngle;
}

public struct PixelProperties {
    public float3 view;
    public float3 normal;
    public float roughness; // roughness = perceptualRoughness * perceptualRoughness
    public float3 diffuseColor; // diffuseColor = (1.0 - metallic) * baseColor.rgb
    // min_reflectance(f0) for dielectrics is a function of reflectance while for metal it comes directly from baseColor
    public float3 min_reflectance; // f0 = 0.16 * reflectance * reflectance * (1.0 - metallic) + baseColor * metallic;
}
