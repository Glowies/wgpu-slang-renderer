# Talking Points
Here are some interesting talking points about the choices made in this project and the reasons behind them.

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
