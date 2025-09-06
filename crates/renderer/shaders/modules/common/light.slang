module "light";

import "../filament-brdf.slang";
import "bsdf-properties.slang";

public struct LightUniform {
    public float3 position;
    public float intensity;
    public float3 color;
}

public float getSquareFalloffAttenuation(float3 posToLight, float lightInvRadius) {
    float distanceSquare = dot(posToLight, posToLight);
    float factor = distanceSquare * lightInvRadius * lightInvRadius;
    float smoothFactor = max(1.0 - factor * factor, 0.0);
    return (smoothFactor * smoothFactor) / max(distanceSquare, 1e-4);
}

float getSpotAngleAttenuation(float3 l, float3 lightDir, float innerAngle, float outerAngle) {
    // the scale and offset computations can be done CPU-side
    float cosOuter = cos(outerAngle);
    float spotScale = 1.0 / max(cos(innerAngle) - cosOuter, 1e-4);
    float spotOffset = -cosOuter * spotScale;

    float cd = dot(normalize(-lightDir), l);
    float attenuation = clamp(cd * spotScale + spotOffset, 0.0, 1.0);
    return attenuation * attenuation;
}

public float3 evaluatePunctualLight(PixelProperties pixel, LightProperties light) {
    // destruct required parameters
    let posToLight = light.posToLight;
    let lightDir = light.direction;
    let n = pixel.normal;

    float3 l = normalize(posToLight);
    float NoL = clamp(dot(n, l), 0.0, 1.0);

    float attenuation;
    attenuation  = getSquareFalloffAttenuation(posToLight, light.invRadius);
    attenuation *= getSpotAngleAttenuation(l, lightDir, light.innerAngle, light.outerAngle);

    float3 luminance = (BRDF(pixel, light) * light.intensity * attenuation * NoL) * light.color;
    return luminance;
}
