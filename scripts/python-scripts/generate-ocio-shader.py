import PyOpenColorIO as OCIO
import os

def get_ocio_config(config_path=None):
    config = None

    if config_path and os.path.exists(config_path):
        config = OCIO.Config.CreateFromFile(config_path)
        print(f"Loaded OCIO config from: {config_path}")
    else:
        config = OCIO.Config.CreateFromBuiltinConfig("studio-config-latest")
        print("Using latest ACES studio OCIO config.")

    return config
    
def generate_ocio_hlsl_shader(
    config,
    ocio_transform,
    shader_name="ocio_transform",
    function_name="ocio_display_view_transform",
    resource_prefix="ocio"
):

    # Create a processor for the display transform
    processor = config.getProcessor(
        ocio_transform
    )

    # Create a GPU shader description for HLSL
    shader_desc = OCIO.GpuShaderDesc.CreateShaderDesc()
    shader_desc.setLanguage(OCIO.GPU_LANGUAGE_HLSL_DX11)
    shader_desc.setFunctionName(function_name)
    shader_desc.setResourcePrefix(resource_prefix)

    # Generate the HLSL shader code from the processor
    # This will contain the core OCIO transformation logic
    gpu_proc = processor.getOptimizedGPUProcessor(OCIO.OptimizationFlags.OPTIMIZATION_ALL)
    gpu_proc.extractGpuShaderInfo(shader_desc)
    ocio_hlsl_code = shader_desc.getShaderText()

    # HLSL Pixel Shader Template
    # This shader receives a texture and applies the OCIO transform to its color.
    pixel_shader_template = f"""
implementing color;

{ocio_hlsl_code}
"""
    return pixel_shader_template

if __name__ == "__main__":
    config_path = None
    config_path = "../studio-config-all-views-v2.3.0_aces-v2.0_ocio-v2.4.ocio"
    config = get_ocio_config(config_path)

    # Sample Display-View Transform
    input_color_space = "lin_rec709"
    # display = "sRGB - Display"
    display = "Display P3 - Display"
    # view = "ACES 2.0 - SDR 100 nits (Rec.709)"
    view = "ACES 1.0 - SDR Video"

    display_view_transform = OCIO.DisplayViewTransform(
        src=input_color_space,
        display=display,
        view=view
    )

    # Sample Color Space Transform
    # input_color_space = "lin_ap1"
    input_color_space = "lin_rec709"
    output_color_space = "acescct_ap1"

    cst = OCIO.ColorSpaceTransform(
        src=input_color_space,
        dst=output_color_space,
    )
    
    fragment_shader = generate_ocio_hlsl_shader(
        config,        
        cst,
        shader_name="ocio_transform",
        function_name="ocio_acescg_to_acescct",
        resource_prefix="ocio"
    )

    with open('ocio.slang', 'w') as f:
        f.write(fragment_shader)
    
