# NeoCogi's core libraries
![Build workflow](https://github.com/neocogi/neocogi/actions/workflows/rust.yml/badge.svg) [![Crate](https://img.shields.io/crates/v/neocogi.svg)](https://crates.io/crates/neocogi)

![UI+3D Triangle Example](docs/ui-screenshot.png)
A repo containing open source NeoCogi libraries:

## Features
- 3D Renderer (GLES 3)
  - Pass/Render Command queue 
  - Pixel readbacks
  - GLSL Shaders
- 3D Helpers (3D Objects rendering: Debug Meshes)
- Immediate Mode GUI
  - Windows/Panels/Popups
  - Widgets
  - 3D Viewport
- Very lightweight (< 250Kb for the UI example)

## TODO
- [x] Rendering Helpers
- [x] Pass as the unit of rendering: a pass record both updates and draw commands
- [x] `Arc<Resource>` & `Arc<Mutex<Driver>>`
- [x] Immediate mode UI
- [x] UI Examples
- [x] Direct bypass rendering commands
- [ ] Pixel uniform Example
- [ ] Pixel readback Example
- [ ] Partial texture updates: Not sure how wise it is to support it
- [ ] Stall detection on resource update commands (guard with a mutex?)


## License

BSD-3-Clause license
