module "filament-brdf";

import "math.slang";
import "common/bsdf-properties.slang";

// From the Filament design doc
// https://google.github.io/filament/Filament.html#table_symbols
// Symbol Definition
// v    View unit vector
// l    Incident light unit vector
// n    Surface normal unit vector
// h    Half unit vector between l and v
// f    BRDF
// f_d    Diffuse component of a BRDF
// f_r    Specular component of a BRDF
// α    Roughness, remapped from using input perceptualRoughness
// σ    Diffuse reflectance
// Ω    Spherical domain
// f0    Reflectance at normal incidence
// f90    Reflectance at grazing angle
// χ+(a)    Heaviside function (1 if a>0 and 0 otherwise)
// nior    Index of refraction (IOR) of an interface
// ⟨n⋅l⟩    Dot product clamped to [0..1]
// ⟨a⟩    Saturated value (clamped to [0..1])


float D_GGX(float NoH, float a) {
    let a2 = a * a;
    let f = (NoH * a2 - NoH) * NoH + 1.0;
    return a2 / (PI * f * f);
}

float3 F_Schlick(float u, float3 f0) {
    return f0 + (float3(1.0) - f0) * pow(1.0 - u, 5.0);
}

float V_SmithGGXCorrelated(float NoV, float NoL, float roughness) {
    float a = roughness;
    float GGXV = NoL * (NoV * (1.0 - a) + a);
    float GGXL = NoV * (NoL * (1.0 - a) + a);
    return 0.5 / (GGXV + GGXL);
}

public float Fd_Lambert() {
    return 1.0 / PI;
}

public float3 BRDF(PixelProperties pixel, LightProperties light) {
    // destruct pixel struct
    let diffuseColor = pixel.diffuseColor;
    let roughness = pixel.roughness;
    let f0 = pixel.min_reflectance;
    let n = pixel.normal;
    let v = pixel.view;

    // destruct light struct
    let l = light.direction;
    
    let h = normalize(v + l);

    let NoV = abs(dot(n, v)) + 1e-5;
    let NoL = clamp(dot(n, l), 0.0, 1.0);
    let NoH = clamp(dot(n, h), 0.0, 1.0);
    let LoH = clamp(dot(l, h), 0.0, 1.0);

    let D = D_GGX(NoH, roughness);
    let F = F_Schlick(LoH, f0);
    let V = V_SmithGGXCorrelated(NoV, NoL, roughness);

    // specular BRDF
    let Fr = (D * V) * F;

    // diffuse BRDF
    let Fd = diffuseColor * Fd_Lambert();

    return Fr + Fd;
}

