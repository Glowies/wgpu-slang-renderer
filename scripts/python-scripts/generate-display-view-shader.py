import PyOpenColorIO as OCIO
import os

def generate_ocio_hlsl_shader(
    config_path=None,
    input_color_space="sRGB",
    display="sRGB",
    view="Film",
    shader_name="ocio_transform",
    function_name="ocio_display_view_transform",
    resource_prefix="ocio"
):
    # Load the OCIO configuration
    if config_path and os.path.exists(config_path):
        config = OCIO.Config.CreateFromFile(config_path)
        print(f"Loaded OCIO config from: {config_path}")
    else:
        config = OCIO.Config.CreateFromEnv()
        if config is None:
            config = OCIO.Config.Create() # Fallback to a default empty config
        print("Using default OCIO config.")

    # Create a processor for the display transform
    processor = config.getProcessor(
        OCIO.DisplayViewTransform(
            src=input_color_space, # Apply display transform from scene linear
            display=display,
            view=view
        )
    )

    # Create a GPU shader description for HLSL
    shader_desc = OCIO.GpuShaderDesc.CreateShaderDesc()
    shader_desc.setLanguage(OCIO.GPU_LANGUAGE_HLSL_DX11)
    shader_desc.setFunctionName(function_name)
    shader_desc.setResourcePrefix(resource_prefix)

    # Generate the HLSL shader code from the processor
    # This will contain the core OCIO transformation logic
    gpu_proc = processor.getDefaultGPUProcessor()
    gpu_proc.extractGpuShaderInfo(shader_desc)
    ocio_hlsl_code = shader_desc.getShaderText()

    # HLSL Pixel Shader Template
    # This shader receives a texture and applies the OCIO transform to its color.
    pixel_shader_template = f"""
// HLSL Pixel Shader
// Shader Model 5.0 (for DirectX 11)
struct PS_INPUT
{{
    float4 Pos : SV_POSITION;
    float2 Tex : TEXCOORD0;
}};

Texture2D uTexture;
SamplerState uSampler;

// --- OCIO Generated Code Start ---
{ocio_hlsl_code}
// --- OCIO Generated Code End ---

float4 main(PS_INPUT input) : SV_Target
{{
    // Sample the texture color
    float4 input_color = uTexture.Sample(uSampler, input.Tex);

    // Apply the OCIO transform to the color (assuming it's in the input_color_space)
    // The OCIO function usually expects a float4 and returns a float4.
    // The alpha channel is typically passed through untouched by OCIO display transforms.
    float4 transformed_color = {function_name}(input_color);

    return transformed_color;
}}
"""
    return pixel_shader_template

if __name__ == "__main__":
    config_path = "../studio-config-all-views-v2.3.0_aces-v2.0_ocio-v2.4.ocio"
    input_color_space = "lin_rec709"
    display = "sRGB - Display"
    view = "ACES 2.0 - SDR 100 nits (Rec.709)"
    fragment_shader = generate_ocio_hlsl_shader(
        config_path,
        input_color_space,
        display,
        view,
        shader_name="ocio_transform",
        function_name="ocio_display_view_transform",
        resource_prefix="ocio"
    )

    with open('ocio_frag.hlsl', 'w') as f:
        f.write(fragment_shader)
    
