//
// Copyright 2021-Present (c) Raja Lehtihet & Wael El Oraiby
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
// this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
// this list of conditions and the following disclaimer in the documentation
// and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
// may be used to endorse or promote products derived from this software without
// specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.
//
use super::*;
use crate::rs_math3d::*;
use crate::*;
use crate::scene::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavigationMode {
    Pan,
    Orbit,
}

pub struct View3D {
    camera: Camera,
    nav_mode: NavigationMode,
    dimension: Dimensioni,
    scroll: f32,
    bounds: Box3f,
    pvm: Mat4f,

    pointer_state: pointer::State,
}

impl View3D {
    pub fn new(camera: Camera, dimension: Dimensioni, bounds: Box3f) -> Self {
        let scroll = camera.distance();
        Self {
            camera,
            nav_mode: NavigationMode::Orbit,
            dimension,
            scroll,
            bounds,
            pvm: camera.projection_matrix() * camera.view_matrix(),

            pointer_state: pointer::State::new(),
        }
    }

    fn update(&mut self) -> UpdateResult {
        // TODO: do proper computation of the far plane
        let far_plane = self.bounds.extent().length() * 100.0;
        self.camera = self.camera.with_far_plane(far_plane);
        let handled = match (&self.nav_mode, self.pointer_state.event()) {
            (NavigationMode::Orbit, pointer::Event::Drag(prev, _, curr, _)) => {
                let p = Vec2f::new(prev.x, prev.y);
                let c = Vec2f::new(curr.x, curr.y);
                self.camera = self.camera.tracball_rotate(self.dimension, &p, &c);
                UpdateResult::Handled
            }

            (NavigationMode::Pan, pointer::Event::Drag(prev, _, curr, _)) => {
                let p = Vec2f::new(prev.x, prev.y);
                let c = Vec2f::new(curr.x, curr.y);
                self.camera = self.camera.pan(self.dimension, &p, &c);
                UpdateResult::Handled
            }

            (_, pointer::Event::Scroll(v)) => {
                self.scroll += v;
                self.scroll = f32::max(0.5, self.scroll);
                let distance = self.scroll;
                let aspect = (self.dimension.width as f32) / (self.dimension.height as f32);
                self.camera = Camera::new(
                    self.camera.target(),
                    distance,
                    self.camera.rotation(),
                    self.camera.fov(),
                    aspect,
                    self.camera.near_plane(),
                    self.camera.far_plane(),
                );
                self.pointer_state.reset_button_state();
                UpdateResult::Handled
            }

            _ => UpdateResult::Unhandled,
        };

        self.pvm = self.camera.projection_matrix() * self.camera.view_matrix();
        handled
    }

    pub fn set_pointer(&mut self, pos: Vec2f, st: pointer::ButtonState) -> UpdateResult {
        self.pointer_state.update(pos, st);
        self.update()
    }

    pub fn set_dimension(&mut self, dimension: Dimensioni) {
        self.dimension = dimension;
        let aspect = (self.dimension.width as f32) / (self.dimension.height as f32);
        self.camera = self.camera.with_aspect(aspect);
        self.update();
    }

    pub fn get_navigation_mode(&self) -> NavigationMode {
        self.nav_mode
    }
    pub fn set_navigation_mode(&mut self, nav_mode: NavigationMode) {
        if nav_mode != self.nav_mode {
            self.nav_mode = nav_mode;
            self.pointer_state = pointer::State::new();
            self.update(); // idem potent in this case
        }
    }

    pub fn pointer_event(&self) -> pointer::Event {
        self.pointer_state.event()
    }

    pub fn pvm(&self) -> Mat4f {
        self.pvm
    }

    pub fn projection_matrix(&self) -> Mat4f {
        self.camera.projection_matrix()
    }

    pub fn view_matrix(&self) -> Mat4f {
        self.camera.view_matrix()
    }
}
