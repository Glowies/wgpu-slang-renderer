## Generating an OCIO Display-View WGSL Shader
Run the following command to generate the hlsl shader using OpenColorIO's python bindings:
(You'll need uv-python installed to be able to run `uv`)
```
uv run generate-display-view-shader.py  
```

Then, run this to convert the hlsl shader to a wgsl shader.
(You will need shader-slang binaries installed to be able to run `slangc`)
```
slangc -lang hlsl -stage fragment ocio_frag.hlsl -target wgsl -o ocio_frag.wgsl
```
