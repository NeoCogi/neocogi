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

use glfw::ffi::glfwGetMonitorWorkarea;
use neocogi::glfw;
use neocogi::glfw::Context;

use neocogi::rs_math3d::*;
use neocogi::*;

use neocogi::renderer::*;

use neocogi::ui;
use ui::*;

pub fn r_get_char_width(_font: FontId, c: char) -> usize {
    ATLAS[ATLAS_FONT as usize + c as usize].width as usize
}

pub fn r_get_font_height(_font: FontId) -> usize {
    18
}

struct State<'a> {
    label_colors: [LabelColor<'a>; 15],
    bg: [Real; 3],
    logbuf: FixedString<65536>,
    logbuf_updated: bool,
    submit_buf: FixedString<128>,
    ctx: ui::Context,
    checks: [bool; 3],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LabelColor<'a> {
    pub label: &'a str,
    pub idx: ControlColor,
}

impl<'a> State<'a> {
    pub fn new() -> Self {
        let mut ctx = ui::Context::new();

        ctx.char_width = Some(r_get_char_width);
        ctx.font_height = Some(r_get_font_height);

        Self {
            label_colors: [
                LabelColor {
                    label: "text",
                    idx: ControlColor::Text,
                },
                LabelColor {
                    label: "border:",
                    idx: ControlColor::Border,
                },
                LabelColor {
                    label: "windowbg:",
                    idx: ControlColor::WindowBG,
                },
                LabelColor {
                    label: "titlebg:",
                    idx: ControlColor::TitleBG,
                },
                LabelColor {
                    label: "titletext:",
                    idx: ControlColor::TitleText,
                },
                LabelColor {
                    label: "panelbg:",
                    idx: ControlColor::PanelBG,
                },
                LabelColor {
                    label: "button:",
                    idx: ControlColor::Button,
                },
                LabelColor {
                    label: "buttonhover:",
                    idx: ControlColor::ButtonHover,
                },
                LabelColor {
                    label: "buttonfocus:",
                    idx: ControlColor::ButtonFocus,
                },
                LabelColor {
                    label: "base:",
                    idx: ControlColor::Base,
                },
                LabelColor {
                    label: "basehover:",
                    idx: ControlColor::BaseHover,
                },
                LabelColor {
                    label: "basefocus:",
                    idx: ControlColor::BaseFocus,
                },
                LabelColor {
                    label: "scrollbase:",
                    idx: ControlColor::ScrollBase,
                },
                LabelColor {
                    label: "scrollthumb:",
                    idx: ControlColor::ScrollThumb,
                },
                LabelColor {
                    label: "",
                    idx: ControlColor::Text,
                },
            ],
            bg: [90.0, 95.0, 100.0],
            logbuf: FixedString::new(),
            logbuf_updated: false,
            submit_buf: FixedString::new(),
            ctx,
            checks: [false, true, false],
        }
    }

    fn write_log(&mut self, text: &str) {
        if self.logbuf.len() != 0 {
            self.logbuf.push('\n');
        }
        for c in text.chars() {
            self.logbuf.push(c);
        }
        self.logbuf_updated = true;
    }

    fn test_window(&mut self) {
        if !self
            .ctx
            .begin_window_ex(
                "Demo Window",
                Rect::new(40, 40, 300, 450),
                WidgetOption::NONE,
            )
            .is_none()
        {
            let mut win = self.ctx.get_current_container_rect();
            win.width = if win.width > 240 { win.width } else { 240 };
            win.height = if win.height > 300 { win.height } else { 300 };

            self.ctx.set_current_container_rect(&win);

            let mut buff = FixedString::<128>::new();

            if !self
                .ctx
                .header_ex("Window Info", WidgetOption::NONE)
                .is_none()
            {
                let win_0 = self.ctx.get_current_container_rect();
                self.ctx.layout_row(&[54, -1], 0);
                self.ctx.label("Position:");

                buff.clear();
                buff.append_int("%d", win_0.x);
                buff.add_str(", ");
                buff.append_int("%d", win_0.y);

                self.ctx.label(buff.as_str());
                buff.clear();
                self.ctx.label("Size:");

                buff.append_int("%d", win_0.width);
                buff.add_str(", ");
                buff.append_int("%d", win_0.height);

                self.ctx.label(buff.as_str());
            }
            if !self
                .ctx
                .header_ex("Test Buttons", WidgetOption::EXPANDED)
                .is_none()
            {
                self.ctx.layout_row(&[86, -110, -1], 0);
                self.ctx.label("Test buttons 1:");
                if !self
                    .ctx
                    .button_ex("Button 1", Icon::None, WidgetOption::ALIGN_CENTER)
                    .is_none()
                {
                    self.write_log("Pressed button 1");
                }
                if !self
                    .ctx
                    .button_ex("Button 2", Icon::None, WidgetOption::ALIGN_CENTER)
                    .is_none()
                {
                    self.write_log("Pressed button 2");
                }
                self.ctx.label("Test buttons 2:");
                if !self
                    .ctx
                    .button_ex("Button 3", Icon::None, WidgetOption::ALIGN_CENTER)
                    .is_none()
                {
                    self.write_log("Pressed button 3");
                }
                if !self
                    .ctx
                    .button_ex("Popup", Icon::None, WidgetOption::ALIGN_CENTER)
                    .is_none()
                {
                    self.ctx.open_popup("Test Popup");
                }
                if !self.ctx.begin_popup("Test Popup").is_none() {
                    if !self
                        .ctx
                        .button_ex("Hello", Icon::None, WidgetOption::ALIGN_CENTER)
                        .is_none()
                    {
                        self.write_log("Hello")
                    }
                    if !self
                        .ctx
                        .button_ex("World", Icon::None, WidgetOption::ALIGN_CENTER)
                        .is_none()
                    {
                        self.write_log("World")
                    }
                    self.ctx.end_popup();
                }
            }
            if !self
                .ctx
                .header_ex("Tree and Text", WidgetOption::EXPANDED)
                .is_none()
            {
                self.ctx.layout_row(&[140, -1], 0);
                self.ctx.layout_begin_column();
                if !self
                    .ctx
                    .begin_treenode_ex("Test 1", WidgetOption::NONE)
                    .is_none()
                {
                    if !self
                        .ctx
                        .begin_treenode_ex("Test 1a", WidgetOption::NONE)
                        .is_none()
                    {
                        self.ctx.label("Hello");
                        self.ctx.label("world");
                        self.ctx.end_treenode();
                    }
                    if !self
                        .ctx
                        .begin_treenode_ex("Test 1b", WidgetOption::NONE)
                        .is_none()
                    {
                        if !self
                            .ctx
                            .button_ex("Button 1", Icon::None, WidgetOption::ALIGN_CENTER)
                            .is_none()
                        {
                            self.write_log("Pressed button 1");
                        }
                        if !self
                            .ctx
                            .button_ex("Button 2", Icon::None, WidgetOption::ALIGN_CENTER)
                            .is_none()
                        {
                            self.write_log("Pressed button 2");
                        }
                        self.ctx.end_treenode();
                    }
                    self.ctx.end_treenode();
                }
                if !self
                    .ctx
                    .begin_treenode_ex("Test 2", WidgetOption::NONE)
                    .is_none()
                {
                    self.ctx.layout_row(&[54, 54], 0);
                    if !self
                        .ctx
                        .button_ex("Button 3", Icon::None, WidgetOption::ALIGN_CENTER)
                        .is_none()
                    {
                        self.write_log("Pressed button 3");
                    }
                    if !self
                        .ctx
                        .button_ex("Button 4", Icon::None, WidgetOption::ALIGN_CENTER)
                        .is_none()
                    {
                        self.write_log("Pressed button 4");
                    }
                    if !self
                        .ctx
                        .button_ex("Button 5", Icon::None, WidgetOption::ALIGN_CENTER)
                        .is_none()
                    {
                        self.write_log("Pressed button 5");
                    }
                    if !self
                        .ctx
                        .button_ex("Button 6", Icon::None, WidgetOption::ALIGN_CENTER)
                        .is_none()
                    {
                        self.write_log("Pressed button 6");
                    }
                    self.ctx.end_treenode();
                }
                if !self
                    .ctx
                    .begin_treenode_ex("Test 3", WidgetOption::NONE)
                    .is_none()
                {
                    self.ctx.checkbox("Checkbox 1", &mut self.checks[0]);
                    self.ctx.checkbox("Checkbox 2", &mut self.checks[1]);
                    self.ctx.checkbox("Checkbox 3", &mut self.checks[2]);
                    self.ctx.end_treenode();
                }
                self.ctx.layout_end_column();
                self.ctx.layout_begin_column();
                self.ctx.layout_row(&[-1], 0);
                self.ctx.text(
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Maecenas lacinia, sem eu lacinia molestie, mi risus faucibus ipsum, eu varius magna felis a nulla."
                    ,
                );
                self.ctx.layout_end_column();
            }
            if !self
                .ctx
                .header_ex("Background Color", WidgetOption::EXPANDED)
                .is_none()
            {
                self.ctx.layout_row(&[-78, -1], 74);
                self.ctx.layout_begin_column();
                self.ctx.layout_row(&[46, -1], 0);
                self.ctx.label("Red:");
                self.ctx.slider_ex(
                    &mut self.bg[0],
                    0 as libc::c_int as Real,
                    255 as libc::c_int as Real,
                    0 as libc::c_int as Real,
                    "%.2",
                    WidgetOption::ALIGN_CENTER,
                );
                self.ctx.label("Green:");
                self.ctx.slider_ex(
                    &mut self.bg[1],
                    0 as libc::c_int as Real,
                    255 as libc::c_int as Real,
                    0 as libc::c_int as Real,
                    "%.2",
                    WidgetOption::ALIGN_CENTER,
                );
                self.ctx.label("Blue:");
                self.ctx.slider_ex(
                    &mut self.bg[2],
                    0 as libc::c_int as Real,
                    255 as libc::c_int as Real,
                    0 as libc::c_int as Real,
                    "%.2",
                    WidgetOption::ALIGN_CENTER,
                );
                self.ctx.layout_end_column();
                let r = self.ctx.layout_next();
                self.ctx.draw_rect(
                    r,
                    color(self.bg[0] as u8, self.bg[1] as u8, self.bg[2] as u8, 255),
                );
                let mut buff = FixedString::<128>::new();
                buff.add_str("#");
                buff.append_int("%02X", self.bg[0] as _);
                buff.append_int("%02X", self.bg[1] as _);
                buff.append_int("%02X", self.bg[2] as _);
                self.ctx.draw_control_text(
                    buff.as_str(),
                    r,
                    ControlColor::Text,
                    WidgetOption::ALIGN_CENTER,
                );
            }
            self.ctx.end_window();
        }
    }

    fn log_window(&mut self) {
        if !self
            .ctx
            .begin_window_ex(
                "Log Window",
                Rect::new(350, 40, 300, 200),
                WidgetOption::NONE,
            )
            .is_none()
        {
            self.ctx.layout_row(&[-1], -25);
            self.ctx.begin_panel_ex("Log Output", WidgetOption::NONE);
            let mut scroll = self.ctx.get_current_container_scroll();
            let content_size = self.ctx.get_current_container_content_size();
            self.ctx.layout_row(&[-1], -1);
            self.ctx.text(self.logbuf.as_str());
            if self.logbuf_updated {
                scroll.y = content_size.y;
                self.ctx.set_current_container_scroll(&scroll);
                self.logbuf_updated = false;
            }
            self.ctx.end_panel();

            let mut submitted = false;
            self.ctx.layout_row(&[-70, -1], 0);
            if self
                .ctx
                .textbox_ex(&mut self.submit_buf, WidgetOption::NONE)
                .is_submitted()
            {
                self.ctx.set_focus(self.ctx.last_id);
                submitted = true;
            }
            if !self
                .ctx
                .button_ex("Submit", Icon::None, WidgetOption::ALIGN_CENTER)
                .is_none()
            {
                submitted = true;
            }
            if submitted {
                let mut buf = FixedString::<128>::new();
                buf.add_str(self.submit_buf.as_str());
                self.write_log(buf.as_str());
                self.submit_buf.clear();
            }
            self.ctx.end_window();
        }
    }
    fn uint8_slider(&mut self, value: &mut u8, low: i32, high: i32) -> ResourceState {
        let mut tmp = *value as f32;
        self.ctx.push_id_from_ptr(value);
        let res = self.ctx.slider_ex(
            &mut tmp,
            low as Real,
            high as Real,
            0 as libc::c_int as Real,
            "%.2f",
            WidgetOption::ALIGN_CENTER,
        );
        *value = tmp as libc::c_uchar;
        self.ctx.pop_id();
        return res;
    }
    fn style_window(&mut self) {
        if !self
            .ctx
            .begin_window_ex(
                "Style Editor",
                Rect::new(
                    350 as libc::c_int,
                    250 as libc::c_int,
                    300 as libc::c_int,
                    240 as libc::c_int,
                ),
                WidgetOption::NONE,
            )
            .is_none()
        {
            let sw: libc::c_int = (self.ctx.get_current_container_body().width as libc::c_double
                * 0.14f64) as libc::c_int;
            self.ctx.layout_row(&[80, sw, sw, sw, sw, -1], 0);
            let mut i = 0;
            while self.label_colors[i].label.len() > 0 {
                self.ctx.label(self.label_colors[i].label);
                unsafe {
                    let color = self.ctx.style.colors.as_mut_ptr().offset(i as isize);
                    self.uint8_slider(&mut (*color).x, 0 as libc::c_int, 255 as libc::c_int);
                    self.uint8_slider(&mut (*color).y, 0 as libc::c_int, 255 as libc::c_int);
                    self.uint8_slider(&mut (*color).z, 0 as libc::c_int, 255 as libc::c_int);
                    self.uint8_slider(&mut (*color).w, 0 as libc::c_int, 255 as libc::c_int);
                }
                let next_layout = self.ctx.layout_next();
                self.ctx.draw_rect(next_layout, self.ctx.style.colors[i]);
                i += 1;
            }
            self.ctx.end_window();
        }
    }

    fn process_frame(&mut self) {
        self.ctx.begin();
        self.style_window();
        self.log_window();
        self.test_window();
        self.ctx.end();
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
        .create_window(1024, 900, "ui Test", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_char_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.make_current();
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    let mut driver = renderer::get_driver();

    // initialize UI
    let mut state = State::new();
    let (width, height) = window.get_framebuffer_size();
    let mut painter = Painter::new(&mut driver, width as u32, height as u32);

    let start_time = std::time::Instant::now();
    'running: while !window.should_close() {
        let (width, height) = window.get_framebuffer_size();
        painter.set_canvas_size(width as u32, height as u32);

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

        state.process_frame();
        //Note: passing a bg_color to paint_jobs will clear any previously drawn stuff.
        //Use this only if egui is being used for all drawing and you aren't mixing your own Open GL
        //drawing calls with it.
        //Since we are custom drawing an OpenGL Triangle we don't need egui to clear the background.
        painter.paint(&mut pass, &mut state.ctx);

        driver.render_pass(&mut pass);
        window.swap_buffers();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Close | glfw::WindowEvent::Key (glfw::Key::Escape, ..) => break 'running,
                glfw::WindowEvent::CursorPos(x, y) => state.ctx.input_mousemove(x as i32, y as i32),
                glfw::WindowEvent::Char(ch) => state.ctx.input_text(String::from(ch).as_str()),
                glfw::WindowEvent::MouseButton(mb, ac, _) => {
                    let (x, y) = window.get_cursor_pos();
                    let b = match mb {
                        glfw::MouseButtonLeft => ui::MouseButton::LEFT,
                        glfw::MouseButtonLeft => ui::MouseButton::RIGHT,
                        _ => ui::MouseButton::NONE
                    };

                    match ac {
                        glfw::Action::Press => state.ctx.input_mousedown(x as i32, y as i32, b),
                        glfw::Action::Release => state.ctx.input_mouseup(x as i32, y as i32, b),
                        _ => ()
                    }
                }
                // glfw::Event::MouseMotion { x, y, .. } => state.ctx.input_mousemove(x, y),
                // glfw::Event::MouseWheel { y, .. } => state.ctx.input_scroll(0, y * -30),
                // glfw::Event::MouseButtonDown { x, y, mouse_btn, .. } => {
                //     let mb = map_mouse_button(mouse_btn);
                //     state.ctx.input_mousedown(x, y, mb);
                // }
                // glfw::Event::MouseButtonUp { x, y, mouse_btn, .. } => {
                //     let mb = map_mouse_button(mouse_btn);
                //     state.ctx.input_mouseup(x, y, mb);
                // }
                // glfw::Event::KeyDown { keymod, keycode, .. } => {
                //     let km = map_keymode(keymod, keycode);
                //     state.ctx.input_keydown(km);
                // }
                // glfw::Event::KeyUp { keymod, keycode, .. } => {
                //     let km = map_keymode(keymod, keycode);
                //     state.ctx.input_keyup(km);
                // }
                // glfw::Event::TextInput { text, .. } => {
                //     state.ctx.input_text(text.as_str());
                // }

                _ => (),
            }
        }
    }

    window.close()
}
