## Generating an OCIO Display-View Shader
Run the following command to generate the hlsl shader using OpenColorIO's python bindings:
(You'll need uv-python installed to be able to run `uv`)
```
uv run generate-display-view-shader.py  
```

OCIO doesn't support extracting slang shaders currently; however, slang is so similar to hlsl that
most hlsl shaders from OCIO can directly be interpreted as if they as slang. There's just a few things
that need to be done if we want to use the shader as a *module* in slang:

- Add `implementing color` to make it part of the color module
- In the color module, add `__include SLANG_SHADER_FILE` to include your new shader
- Mark the function you want to expose as public

### Converting the generated shader to wgsl

If you want to convert the generated shader to wgsl directly, you need to generate an hlsl fragment
shader with a valid entry point, and you need to use the generated OCIO function in that entry point.
Once you've done that, you can run the following command to directly convert it to wgsl.
(You will need shader-slang binaries installed to be able to run `slangc`)
```
slangc -lang hlsl -stage fragment ocio_frag.hlsl -target wgsl -o ocio_frag.wgsl
```
