# Learn WGPU Tutorials
Source: https://sotrh.github.io/learn-wgpu

# Building the project
## For WASM
Run the following command to build the wasm library:
```bash
cd ./crates/renderer
wasm-pack build --target web
```

# Dev Workflow
## For WASM
While developing for web, you need to set up two directory symlinks:
1. One for your dir containing the built and packaged WASM library.
2. One for your dir containing the static files that WASM will fetch at runtime

For example, when serving our project with Svelte, here's how those symlinks should be:
1. `<RUST_PROJECT_ROOT>/crates/renderer/pkg` -> `<SVELTE_PROJECT_ROOT>/src/lib/pkg`
> For Svelte, `/src/lib/` is where libraries directly references in components should go.

2. `<RUST_PROJECT_ROOT>/crates/renderer/res` -> `<SVELTE_PROJECT_ROOT>/static/<PATH_TO_PAGE>/res`
> For example, if we want to serve our project at `http://localhost/my_page`, then we
> need to use the path `/static/my_page/res` under our Svelte project.

Once the symlinks are set up, you can import and init the WASM library:
```typescript
import init from '$lib/pkg/renderer.js';

onMount(async () => {
      init().catch((error) => {
            if (!error.message.startsWith("Using exceptions for control flow,")) {
                  throw error;
            }
      });
});
```
 
# TODO
- [ ] Identify odd tonemapper behavior with greens in color sweep
- [ ] Use Slang to compile to SPIRV or WGSL
- [ ] Use block compression on all ktx2 textures
- [ ] impl of AsBindGroup in Material has too many empty functions. Maybe some of those methods in AsBindGroup should be moved to another derived trait that is only for UniformBuffer bindgroups?
- [ ] Refactor to have a LightManager and do an instance draw call from there to draw all light debug meshes.
