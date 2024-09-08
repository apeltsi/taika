![Taika Logo](https://github.com/apeltsi/taika/assets/49206921/a6ab4209-4235-4b88-b242-d1931423248f)

A low-cost abstraction layer on top of [wgpu](https://crates.io/crates/wgpu) and [winit](https://crates.io/crates/winit) to make their APIs more ergonomic.

This is very much a work-in-progress. This was originally meant to replace LoiRen, the renderer in [loitsu](https://github.com/apeltsi/loitsu), but the projects kinda got split off. Might still happen one day though.
Currently most settings are hard-coded this will change.

# State
Taika is early in development, meaning large API changes are bound to happen. However it is currently being used for a production ready game
which serves as a good testbed for the library.

# Goals
1. Simplify window creation
2. Introduce "RenderPasses" and "RenderPipelines", which are common tropes in game
   engines.
3. Make API changes in WGPU and Winit less frustrating by providing a semi-stable API. API
   changes will still happen though.
4. Give full access to WGPU

In addition to these goals taika also includes some common utilities mainly targeted towards
game-development. Taika also includes a super basic form of asset management. It is designed to
be built upon, not to be a full-fledged asset management system.

## What taika doesn't do:
- Input-handling, you can do this yourself by listening to the winit events that are passed
  through to your event-handler
- Audio, use other libraries
- Make rendering easy. You still have to write shaders, and implement the drawable trait, to
  actually issue the drawcalls to the GPU. Taika doesn't make any drawcalls by itself

# Notes
- The naming of `rendering::RenderPass` and `rendering::RenderPipeline` is a bit confusing at they are also used in
  wgpu.
- No examples currently!

## Platforms
Actively tested on Windows, Linux (wayland and x11) and macOS.

Web might happen one day
