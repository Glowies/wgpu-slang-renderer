# Learn WGPU Tutorials
Source: https://sotrh.github.io/learn-wgpu

# Building the project
## For WASM
Run the following command to build the wasm library:
```bash
wasm-pack build --target web
```

# Dev Workflow
## For WASM
While developing for web, you need to set up two directory symlinks:
1. One for your dir containing the built and packaged WASM library.
2. One for your dir containing the static files that WASM will fetch at runtime

For example, when serving our project with Svelte, here's how those symlinks should be:
1. `<RUST_PROJECT_ROOT>/pkg` -> `<SVELTE_PROJECT_ROOT>/src/lib/pkg`
> For Svelte, `/src/lib/` is where libraries directly references in components should go.

2. `<RUST_PROJECT_ROOT>/res` -> `<SVELTE_PROJECT_ROOT>/static/<PATH_TO_PAGE>/res`
> For example, if we want to serve our project at `http://localhost/my_page`, then we
> need to use the path `/static/my_page/res` under our Svelte project.
 
# TODO
- [ ] Open PR for missing `cache: None` in Seeing the light section
- [ ] Refactor to have a LightManager and do an instance draw call from there to draw all light debug meshes.
