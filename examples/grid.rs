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
extern crate neocogi;

use glfw::WindowEvent::MouseButton;
use neocogi::glfw;
use neocogi::glfw::Context;

use neocogi::rs_math3d::*;
use neocogi::*;

use neocogi::renderer::*;

use neocogi::editor::*;
use neocogi::scene::*;
use neocogi::ui::*;

use neocogi::scene::utility_mesh::*;

pub struct State {
    view: View3D,
    um_renderer: UMRenderer,
}

impl State {
    pub fn new(driver: &mut DriverPtr, width: usize, height: usize) -> Self {
        let um_renderer = UMRenderer::new(driver, 65536);
        let camera = Camera::new(
            Vec3f::zero(),
            20.0,
            Quatf::of_axis_angle(&Vec3f::new(-1.0, 0.0, 0.0), std::f32::consts::PI / 4.0),
            std::f32::consts::PI / 4.0,
            1.0,
            0.1,
            100.0,
        );
        let view = View3D::new(
            camera,
            Dimensioni::new(1024, 900),
            Box3f::new(
                &Vec3f::new(-20.0, -20.0, -20.0),
                &Vec3f::new(20.0, 20.0, 20.0),
            ),
        );

        Self { view, um_renderer }
    }
}

fn main() {
    let mut app = ui::App::new("grid example");
    let style = Style::default();
    app.run(None, |drv, ctx, state| {
        let style = Style::default();
        let (width, height) = ctx.frame_size();
        let mut state = match state {
            Some(s) => s,
            None => State::new(&mut drv.clone(), width, height),
        };
        ctx.window(
            &style,
            "Tools",
            Recti::new(0, 0, 200, height as _),
            ui::WidgetOption::SET_SIZE | ui::WidgetOption::NO_INTERACT | ui::WidgetOption::NO_CLOSE,
            |ctx, style| {
                ctx.column(style, |ctx, style| {
                    if ctx
                        .button(style, "Orbit", None, WidgetOption::NONE)
                        .is_submitted()
                    {
                        state.view.set_navigation_mode(NavigationMode::Orbit)
                    }

                    if ctx
                        .button(style, "Pan", None, WidgetOption::NONE)
                        .is_submitted()
                    {
                        state.view.set_navigation_mode(NavigationMode::Pan)
                    }
                });
            },
        );

        let pos = ctx.mouse_pos;
        let button = ctx.mouse_down;
        ctx.window(
            &style,
            "Viewport",
            Recti::new(201, 0, width as i32 - 201, height as _),
            WidgetOption::NO_TITLE
                | ui::WidgetOption::SET_SIZE
                | ui::WidgetOption::NO_INTERACT
                | ui::WidgetOption::NO_CLOSE,
            |ctx, style| {
                ctx.column(style, |ctx, style| {
                    let mut win = ctx.get_current_container_rect();
                    let size = ctx.frame_size();

                    ctx.render_custom(|queue, clip| {
                        let width = clip.width;
                        let height = clip.height;

                        queue.set_viewport(
                            win.x,
                            size.1 as i32 - win.height - win.y,
                            clip.width as _,
                            win.height as u32,
                        );
                        let mut pass_3d = PassCommandQueue::new();

                        let grid = UMNode::grid_xz(&Vec3f::new(0.0, 0.0, 0.0), 10.0, 20);
                        let axis = UMNode::basis_cone(
                            &Vec3f::zero(),
                            &Vec3f::new(2.0, 0.0, 0.0),
                            &Vec3f::new(0.0, 2.0, 0.0),
                            &Vec3f::new(0.0, 0.0, 2.0),
                        );

                        state
                            .view
                            .set_dimension(Dimensioni::new(clip.width, clip.height));
                        let nodes = UMNode::Assembly(vec![grid, axis]);
                        state
                            .um_renderer
                            .draw_node(&mut pass_3d, &state.view.pvm(), &nodes);
                        queue.append(pass_3d);
                        let (xpos, ypos) = (pos.x, pos.y);

                        let rel_x = ((xpos as f32 - clip.x as f32) / (width as f32)) * 2.0 - 1.0;
                        let rel_y =
                            -(((ypos as f32 - clip.y as f32) / (height as f32)) * 2.0 - 1.0);

                        let pointer_button = if button.is_left() {
                            ButtonState::Pressed(1.0)
                        } else {
                            ButtonState::Released
                        };

                        let pointer_pos = Vec2f::new(rel_x, rel_y);

                        state
                            .view
                            .set_dimension(Dimensioni::new(width as i32, height as i32));
                        state.view.set_pointer(pointer_pos, pointer_button);

                        queue.set_viewport(0, 0, size.0 as _, size.1 as _);
                    });
                });
            },
        );
        Some(state)
    });
}
