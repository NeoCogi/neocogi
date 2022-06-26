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
use neocogi::glfw::{Action, Context, Key};

use neocogi::*;
use neocogi::rs_math3d::*;

use neocogi::renderer::*;

use neocogi::egui;
use neocogi::egui::widgets::*;

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
        .create_window(1024, 900, "Egui Test", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_char_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.make_current();
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    let mut driver = renderer::get_driver();

    // initialize EGUI
    let mut egui_ctx = egui::Context::default();
    let mut painter = ui::Painter::new(&mut driver, &mut window, 1024, 900);


    let (width, height) = window.get_framebuffer_size();
    let native_pixels_per_point = window.get_content_scale().0;

    println!("pixels per point: {}", native_pixels_per_point);
    let mut egui_input_state = ui::EguiInputState::new(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::new(0f32, 0f32),
            egui::vec2(width as f32, height as f32) / native_pixels_per_point,
        )),
        pixels_per_point: Some(native_pixels_per_point),
        ..Default::default()
    });

    let start_time = std::time::Instant::now();
    let mut quit = false;
    while !window.should_close() {
        let (width, height) = window.get_framebuffer_size();
        painter.set_canvas_size(width as u32, height as u32);
        let native_pixels_per_point = window.get_content_scale().0;
        egui_input_state.egui_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::new(0f32, 0f32),
            egui::vec2(width as f32, height as f32) / native_pixels_per_point,
        ));
        egui_input_state.egui_input.time = Some(start_time.elapsed().as_secs_f64());


        let egui_output = egui_ctx.run(egui_input_state.egui_input.take(), |egui_ctx| {
            egui::SidePanel::left("Test").show(&egui_ctx, |ui| {
                if ui.button("click me!").clicked() {
                    println!("Clicked")
                }
            });

        });

        //Handle cut, copy text from egui
        if !egui_output.platform_output.copied_text.is_empty() {
            ui::copy_to_clipboard(&mut egui_input_state, egui_output.platform_output.copied_text);
        }

        let paint_jobs = egui_ctx.tessellate(egui_output.shapes);
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

        //Note: passing a bg_color to paint_jobs will clear any previously drawn stuff.
        //Use this only if egui is being used for all drawing and you aren't mixing your own Open GL
        //drawing calls with it.
        //Since we are custom drawing an OpenGL Triangle we don't need egui to clear the background.
        painter.paint_jobs(
            &mut pass,
            paint_jobs,
            &egui_output.textures_delta,
            native_pixels_per_point,
        );

        driver.render_pass(&mut pass);
        window.swap_buffers();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, _, _) |
                glfw::WindowEvent::Close  => quit = true,
                _ => neocogi::ui::handle_event(event, &mut egui_input_state),
            }
        }

        if quit {
            window.set_should_close(true)
        }
    }
}