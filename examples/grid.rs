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
}

impl State {
    pub fn new(width: usize, height: usize) -> Self {
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

        Self { view }
    }
}

fn main() {
    // initialize GLFW3 with OpenGL ES 3.0
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextCreationApi(
        glfw::ContextCreationApi::Egl,
    ));
    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::OpenGlEs));
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 0));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::DoubleBuffer(true));
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    glfw.window_hint(glfw::WindowHint::Floating(true));

    let (mut window, events) = glfw
        .create_window(1024, 900, "Grid", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_char_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.make_current();
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    let mut driver = renderer::get_driver();
    // utility mesh renderer
    let mut um_renderer = UMRenderer::new(&mut driver, 65536);

    // initialize UI
    let (width, height) = window.get_framebuffer_size();

    let mut state = State::new(width as usize, height as usize);
    let renderer = system::Renderer::new(&mut driver, width as u32, height as u32);
    let mut input = Input::new();
    let mut ctx = ui::Context::new(renderer);
    let style = Style::default();
    'running: while !window.should_close() {
        let (width, height) = window.get_framebuffer_size();

        let (mut pass_queue, _) = ctx.frame(width as _, height as _, |ctx| {
            ctx.window(
                &style,
                "Navigation",
                Rect::new(100, 100, 100, 100),
                ui::WidgetOption::NONE,
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
        });

        let mut pass = Pass::new(
            width as usize,
            height as usize,
            None,
            [
                ColorPassAction::Clear(color4b(0x7F, 0x7F, 0x7F, 0xFF)),
                ColorPassAction::Previous,
                ColorPassAction::Previous,
                ColorPassAction::Previous,
            ],
            DepthPassAction::Clear(1.0),
        );

        let mut pass_3d = Pass::new(
            width as usize,
            height as usize,
            None,
            [
                ColorPassAction::Clear(color4b(0x7F, 0x7F, 0x7F, 0xFF)),
                ColorPassAction::Previous,
                ColorPassAction::Previous,
                ColorPassAction::Previous,
            ],
            DepthPassAction::Clear(1.0),
        );

        let grid = UMNode::grid_xz(&Vec3f::new(0.0, 0.0, 0.0), 10.0, 20);
        let axis = UMNode::basis_cone(
            &Vec3f::zero(),
            &Vec3f::new(2.0, 0.0, 0.0),
            &Vec3f::new(0.0, 2.0, 0.0),
            &Vec3f::new(0.0, 0.0, 2.0),
        );

        let nodes = UMNode::Assembly(vec![grid, axis]);
        um_renderer.draw_node(&mut pass_3d, &state.view.pvm(), &nodes);

        driver.render_pass(&mut pass_3d);

        pass.color_actions[0] = ColorPassAction::Previous;
        pass.queue.append(pass_queue);
        driver.render_pass(&mut pass);
        window.swap_buffers();

        let (xpos, ypos) = window.get_cursor_pos();

        let rel_x = ((xpos as f32) / (width as f32)) * 2.0 - 1.0;
        let rel_y = -(((ypos as f32) / (height as f32)) * 2.0 - 1.0);

        let pointer_button =
            if window.get_mouse_button(glfw::MouseButtonLeft) == glfw::Action::Press {
                ButtonState::Pressed(1.0)
            } else {
                ButtonState::Released
            };
        let pointer_pos = Vec2f::new(rel_x, rel_y);

        state
            .view
            .set_dimension(Dimensioni::new(width as i32, height as i32));
        state.view.set_pointer(pointer_pos, pointer_button);

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, _, _) | glfw::WindowEvent::Close => {
                    break 'running
                }
                _ => input.handle_event(event, &mut window, &mut ctx),
            }
        }
    }

    window.close();
}
