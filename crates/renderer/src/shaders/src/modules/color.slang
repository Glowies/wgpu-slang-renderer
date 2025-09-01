module "color";

__include "ocio";
__include "oklab";

public float3 scene_linear_to_shaper_space(float3 color) {
    // constants from the ocio shaper space
    let log_side_slope = 0.0833308877282;
    let log_side_offset = -0.6;
    let lin_side_slope = 4096.0;
    let lin_side_offset = 100.0;

    return log_side_slope * log2(lin_side_slope * color + lin_side_offset) + log_side_offset;
}

public struct ColorSweepSettings {
    public float ev_min;
    public float ev_max;
    public float ev_step;
    public float hue_min;
    public float hue_max;
    public float hue_step;
}

public float3 hsv2rgb(float3 c) {
    let K = float4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * lerp(K.xxx, saturate(p - K.xxx), c.y);
}

public static float3 REC709_LUMINANCE_COEFS = float3(0.2126, 0.7152, 0.0722);

public float luminance_rec709(float3 color) {
    return dot(color, REC709_LUMINANCE_COEFS);
}

public float3 color_sweep(float2 uv, ColorSweepSettings settings) {
    let ev_min = settings.ev_min;
    let ev_max = settings.ev_max;
    let ev_step = settings.ev_step;
    let ev_step_count = (ev_max - ev_min) / ev_step;

    let hue_min = settings.hue_min;
    let hue_max = settings.hue_max;
    let hue_step = settings.hue_step;
    let hue_step_count = (hue_max - hue_min) / hue_step;

    let ev_idx = floor(uv.x * ev_step_count);
    let ev = ev_min + ev_idx * ev_step;
    let value = exp2(ev);

    let hue_idx = floor(uv.y * hue_step_count);
    let hue = hue_min + hue_idx * hue_step;
    let hue_normalized = hue / 360.0;

    let hsv = float3(hue_normalized, 1.0, 1.0);
    let color = hsv2rgb(hsv) * value;

    return color;
}
