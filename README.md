# NeoCogi's core libraries
![Build workflow](https://github.com/neocogi/neocogi/actions/workflows/rust.yml/badge.svg) [![Crate](https://img.shields.io/crates/v/neocogi.svg)](https://crates.io/crates/neocogi)

![UI+3D Triangle Example](docs/ui-screenshot.png)
A repo containing open source NeoCogi libraries:

- [x] Renderer
- [x] Rendering Helpers
- [x] Pass as the unit of rendering: a pass record both updates and draw commands
- [x] Arc \<Resource\> & Arc\<Mutex\<Driver\>\>
- [ ] Pixel uniform (example)
- [ ] Pixel readback (example)
- [ ] Partial texture updates: Not sure how wise it is to support it
- [ ] Stall detection on resource update commands (guard with a mutex?)
- [x] Immediate mode UI
- [x] UI Examples
- [x] Direct bypass rendering commands


## License

BSD-3-Clause license
