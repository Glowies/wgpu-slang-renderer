# Talking Points
Here are some interesting talking points about the choices made in this project and the reasons behind them.

## Row-Major vs Column-Major
It was tough to decide whether to use row-major or column-major matrices in the slang shaders. I had initially written
all my old wgsl shaders assuming column-major since that was the standard for wgsl - at least as far as I understand -
so my initial approach was to keep my slang shaders column-major as well. However, even though slang supports both matrix
representations, [their docs](https://docs.shader-slang.org/en/stable/external/slang/docs/user-guide/a1-01-matrix-layout.html) 
strongly recommended row-major for best portability across different compile targets. In this case, my only target is
compiling to wgsl, so I didn't think that was important. Unfortunately, I started running into some issues with column-major
layout that I could not figure out.

### Issues with Column-Major slang to wgsl 
The main issue was that I would get incorrect behavior whenever I constructed a matrix from individual elements using
the the float4x4 or float3x3 constructor in slang. This seemed to be caused by how slang swaps the rows and columns
of the matrix when it compiles to wgsl, and it also [swaps the order of matrix-vector multiplications](https://docs.shader-slang.org/en/stable/external/slang/docs/user-guide/a2-03-wgsl-target-specific.html#matrix-type-translation) 
to account for that. (The same is true for GLSL and SPIR-V but I haven't tested that.) Oddly enough, this doesn't
seem to apply to matrices constructed from individual elements using the slang constructors. The constructor treats
the elements as column-major layout without transposing them, but it keeps the swapped ordering of matrix-vector
multiplications, which leads to incorrect results. Here is an example:

When constructing the TBN matrix from the tangent, bitangent, and normal vectors, the individual vectors are treated
as columns of the matrix when put into the constructor. Slang also assumes that the vector being multiplied is a
column vector whenever we put the matrix *before* the vector inside the `mul` function. (`mul(mat, vec)`). Under these
assumptions, we should expect the generated wgsl to be either `mat * vec` or `vec * transpose(mat)` but instead we get
`vec * mat` which gives the incorrect result. So even though the TBN matrix should convert from tangent space to
world space, it does not when multiplied on the RHS of the vector, which means that I have to apply an additional
tranpose on the TBN matrix before using it as a 'tangent-to-world' matrix.

### Decision
I decided to keep using column-major matrices and column vectors on the host side (this is what the cgmath crate uses
anyway) but I swapped to using row-major matrices and row vectors on the slang shaders. This is one of the valid
combinations listed in the documentation for matrix layout differences because it has an even number of 'flips'.
This combination lets me use the host-side cgmath library as-is, and transfer its matrix contents as-is to the GPU,
while also letting the wgsl shaders exported from slang to perform correct the order of matrix-vector operations.

The biggest implication of this decision: When I'm applying a matrix transformation to a vector, I need to make sure
that I do `mul(vec, mat)` instead of `mul(mat, vec)`. The former implies to slang that the vector is a row-vector
while the latter implies that the vector is a column-vector.

## Spherical Harmonics for Cube Maps
I decided to compute the Spherical Harmonics (SH) representation of my cubemaps in real-time and use them that way,
instead of computing the irradiance cache ahead-of-time and storing it as an additional cubemap. Main reason for this is that I'm targetting web as my main platform and I want
to avoid big texture transfers as much as possible. Of course, there is the concern of sending such large textures
to the GPU, but more that that, I worry that having to fetch and download these textures from a web-server will hurt
the loading times.

## Why Rust?
More than anything else, the main purpose of this project was for me to get more confident with Rust. However, besides
that, I wanted to see what the benefits of Rust really are when it is put to use. Thankfully, now that I have had a few
months of experience with the repo, I can say that the benefits turned out to be very much worth it. Here are a few:
1- Refactoring Confidence
  There were many occasions where I had to refactor the 'teaching oriented' code from the Learn WGPU tutorials into
  something more production oriented. These were often fairly extensive refactors that touched many parts of the code
  and I was consistently amazed to see how smoothly Rust handled that. I would change my API the way I wanted it to be,
  and start fixing the compiler errors caused by that change. Every single time I did this, my code would *immediately*
  work, as soon as I resolved all compiler errors.

2- Clean Code Structure
  It feels like Rust almost guides me into writing clean code. Whenever I try things that feel hacky, it usually turns
  out not to be very 'compatible' with Rust and either the compiler or Clippy will warn me against them. Simply following
  those two tools, lead me to write much cleaner and easier-to-understand code.

### Any Downsides?
The only downside was that the graphics community in Rust doesn't seem to be too big at the moment. This had a few implications:
1- Lack of reliable references.
  For example, most of this project was based on the tutorials *Learn WGPU* website. The tutorials do a great job teaching
  wgpu; however, they don't pay too much attention to code structure. This often led to situations where I felt like
  I was writing code that wasn't idiomatic of Rust. I tried looking around for other projects that use wgpu but
  because the graphics community is still fairly fresh, there weren't many reliable sources to be found. The only good
  help was the Bevy engine repo.

2- Missing/incomplete libraries.
  The one that hurt the most was not having Rust bindings for the OpenColorIO (OCIO) library. I was really hoping to have
  a clean color pipeline implemented with OpenColorIO, but I had to do most of that manually. (Mostly by creating
  LUTs using OCIO's CLI tools and using them in the correct places.) WebGPU being very new didn't help here either.
  OCIO normally has support for exporting color transforms as shader code; however, since the WebGPU Shading Language
  is also very new, OCIO did not have support for it.
