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

use neocogi::ui;
use ui::*;

struct State<'a> {
    label_colors: [LabelColor<'a>; 15],
    bg: [Real; 3],
    logbuf: FixedString<65536>,
    logbuf_updated: bool,
    submit_buf: FixedString<128>,
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

    fn test_window(&mut self, ctx: &mut ui::Context<Pass, system::Renderer>) {
        ctx
        .window_ex(
                "Demo Window",
                Rect::new(40, 40, 300, 450),
                WidgetOption::NONE,
                |ctx|  {
            let mut win = ctx.get_current_container_rect();
            win.width = if win.width > 240 { win.width } else { 240 };
            win.height = if win.height > 300 { win.height } else { 300 };

            ctx.set_current_container_rect(&win);

            let mut buff = FixedString::<128>::new();

            if !ctx.header_ex("Window Info", WidgetOption::NONE).is_none() {
                let win_0 = ctx.get_current_container_rect();
                ctx.row(&[54, -1], 0);
                ctx.label("Position:");

                buff.clear();
                buff.append_int("%d", win_0.x);
                buff.add_str(", ");
                buff.append_int("%d", win_0.y);

                ctx.label(buff.as_str());
                buff.clear();
                ctx.label("Size:");

                buff.append_int("%d", win_0.width);
                buff.add_str(", ");
                buff.append_int("%d", win_0.height);

                ctx.label(buff.as_str());
            }
            if !ctx
                .header_ex("Test Buttons", WidgetOption::EXPANDED)
                .is_none()
            {
                ctx.row(&[86, -110, -1], 0);
                ctx.label("Test buttons 1:");
                if !ctx
                    .button_ex("Button 1", Icon::None, WidgetOption::ALIGN_CENTER)
                    .is_none()
                {
                    self.write_log("Pressed button 1");
                }
                if !ctx
                    .button_ex("Button 2", Icon::None, WidgetOption::ALIGN_CENTER)
                    .is_none()
                {
                    self.write_log("Pressed button 2");
                }
                ctx.label("Test buttons 2:");
                if !ctx
                    .button_ex("Button 3", Icon::None, WidgetOption::ALIGN_CENTER)
                    .is_none()
                {
                    self.write_log("Pressed button 3");
                }
                if !ctx
                    .button_ex("Popup", Icon::None, WidgetOption::ALIGN_CENTER)
                    .is_none()
                {
                    ctx.open_popup("Test Popup");
                }

                ctx.popup("Test Popup", |ctx| {
                    if !ctx
                        .button_ex("Hello", Icon::None, WidgetOption::ALIGN_CENTER)
                        .is_none()
                    {
                        self.write_log("Hello")
                    }
                    if !ctx
                        .button_ex("World", Icon::None, WidgetOption::ALIGN_CENTER)
                        .is_none()
                    {
                        self.write_log("World")
                    }
                });
            }
            if !ctx
                .header_ex("Tree and Text", WidgetOption::EXPANDED)
                .is_none()
            {
                ctx.row(&[140, -1], 0);
                ctx.column(&ctx.style, |ctx| {
                    ctx.treenode_ex("Test 1", WidgetOption::NONE, |ctx| {
                        ctx.treenode_ex("Test 1a", WidgetOption::NONE, |ctx| {
                            ctx.label("Hello");
                            ctx.label("world");
                        });
                        ctx.treenode_ex("Test 1b", WidgetOption::NONE, |ctx| {
                            if !ctx
                                .button_ex("Button 1", Icon::None, WidgetOption::ALIGN_CENTER)
                                .is_none()
                            {
                                self.write_log("Pressed button 1");
                            }
                            if !ctx
                                .button_ex("Button 2", Icon::None, WidgetOption::ALIGN_CENTER)
                                .is_none()
                            {
                                self.write_log("Pressed button 2");
                            }
                        });
                    });
                    ctx.treenode_ex("Test 2", WidgetOption::NONE, |ctx| {
                        ctx.row(&[54, 54], 0);
                        if !ctx
                            .button_ex("Button 3", Icon::None, WidgetOption::ALIGN_CENTER)
                            .is_none()
                        {
                            self.write_log("Pressed button 3");
                        }
                        if !ctx
                            .button_ex("Button 4", Icon::None, WidgetOption::ALIGN_CENTER)
                            .is_none()
                        {
                            self.write_log("Pressed button 4");
                        }
                        if !ctx
                            .button_ex("Button 5", Icon::None, WidgetOption::ALIGN_CENTER)
                            .is_none()
                        {
                            self.write_log("Pressed button 5");
                        }
                        if !ctx
                            .button_ex("Button 6", Icon::None, WidgetOption::ALIGN_CENTER)
                            .is_none()
                        {
                            self.write_log("Pressed button 6");
                        }
                    });
                    ctx.treenode_ex("Test 3", WidgetOption::NONE, |ctx| {
                        ctx.checkbox("Checkbox 1", &mut self.checks[0]);
                        ctx.checkbox("Checkbox 2", &mut self.checks[1]);
                        ctx.checkbox("Checkbox 3", &mut self.checks[2]);
                    });
                });
                ctx.column(&ctx.style, |ctx| {
                    ctx.row(&[-1], 0);
                    ctx.text(
                        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Maecenas lacinia, sem eu lacinia molestie, mi risus faucibus ipsum, eu varius magna felis a nulla."
                        ,
                    );
                });
            }
            if !ctx
                .header_ex("Background Color", WidgetOption::EXPANDED)
                .is_none()
            {
                ctx.row(&[-78, -1], 74);
                ctx.column(&ctx.style, |crx| {
                    ctx.row(&[46, -1], 0);
                    ctx.label("Red:");
                    ctx.slider_ex(
                        &mut self.bg[0],
                        0 as libc::c_int as Real,
                        255 as libc::c_int as Real,
                        0 as libc::c_int as Real,
                        "%.2",
                        WidgetOption::ALIGN_CENTER,
                    );
                    ctx.label("Green:");
                    ctx.slider_ex(
                        &mut self.bg[1],
                        0 as libc::c_int as Real,
                        255 as libc::c_int as Real,
                        0 as libc::c_int as Real,
                        "%.2",
                        WidgetOption::ALIGN_CENTER,
                    );
                    ctx.label("Blue:");
                    ctx.slider_ex(
                        &mut self.bg[2],
                        0 as libc::c_int as Real,
                        255 as libc::c_int as Real,
                        0 as libc::c_int as Real,
                        "%.2",
                        WidgetOption::ALIGN_CENTER,
                    );
                });
                let r = ctx.layout_stack.next(&ctx.style);
                ctx.draw_rect(
                    r,
                    color(self.bg[0] as u8, self.bg[1] as u8, self.bg[2] as u8, 255),
                );
                let mut buff = FixedString::<128>::new();
                buff.add_str("#");
                buff.append_int("%02X", self.bg[0] as _);
                buff.append_int("%02X", self.bg[1] as _);
                buff.append_int("%02X", self.bg[2] as _);
                ctx.draw_control_text(
                    buff.as_str(),
                    r,
                    ControlColor::Text,
                    WidgetOption::ALIGN_CENTER,
                );
            }
        });
    }

    fn log_window(&mut self, ctx: &mut ui::Context<Pass, system::Renderer>) {
        ctx.window_ex(
            "Log Window",
            Rect::new(350, 40, 300, 200),
            WidgetOption::NONE,
            |ctx| {
                ctx.row(&[-1], -25);
                ctx.panel_ex("Log Output", WidgetOption::NONE, |ctx| {
                    let mut scroll = ctx.get_current_container_scroll();
                    let content_size = ctx.get_current_container_content_size();
                    ctx.row(&[-1], -1);
                    ctx.text(self.logbuf.as_str());
                    if self.logbuf_updated {
                        scroll.y = content_size.y;
                        ctx.set_current_container_scroll(&scroll);
                        self.logbuf_updated = false;
                    }
                });

                let mut submitted = false;
                ctx.row(&[-70, -1], 0);
                if ctx
                    .textbox_ex(&mut self.submit_buf, WidgetOption::NONE)
                    .is_submitted()
                {
                    ctx.set_focus(ctx.last_id);
                    submitted = true;
                }
                if !ctx
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
            },
        );
    }
    fn uint8_slider(
        &mut self,
        ctx: &mut ui::Context<Pass, system::Renderer>,
        value: &mut u8,
        low: i32,
        high: i32,
    ) -> ResourceState {
        let mut tmp = *value as f32;
        ctx.push_id_from_ptr(value);
        let res = ctx.slider_ex(
            &mut tmp,
            low as Real,
            high as Real,
            0 as libc::c_int as Real,
            "%.2f",
            WidgetOption::ALIGN_CENTER,
        );
        *value = tmp as libc::c_uchar;
        ctx.pop_id();
        return res;
    }
    fn style_window(&mut self, ctx: &mut ui::Context<Pass, system::Renderer>) {
        ctx.window_ex(
            "Style Editor",
            Rect::new(
                350 as libc::c_int,
                250 as libc::c_int,
                300 as libc::c_int,
                240 as libc::c_int,
            ),
            WidgetOption::NONE,
            |ctx| {
                let sw: libc::c_int = (ctx.get_current_container_body().width as libc::c_double
                    * 0.14f64) as libc::c_int;
                ctx.row(&[80, sw, sw, sw, sw, -1], 0);
                let mut i = 0;
                while self.label_colors[i].label.len() > 0 {
                    ctx.label(self.label_colors[i].label);
                    unsafe {
                        let color = ctx.style.colors.as_mut_ptr().offset(i as isize);
                        self.uint8_slider(
                            ctx,
                            &mut (*color).x,
                            0 as libc::c_int,
                            255 as libc::c_int,
                        );
                        self.uint8_slider(
                            ctx,
                            &mut (*color).y,
                            0 as libc::c_int,
                            255 as libc::c_int,
                        );
                        self.uint8_slider(
                            ctx,
                            &mut (*color).z,
                            0 as libc::c_int,
                            255 as libc::c_int,
                        );
                        self.uint8_slider(
                            ctx,
                            &mut (*color).w,
                            0 as libc::c_int,
                            255 as libc::c_int,
                        );
                    }
                    let next_layout = ctx.layout_stack.next(&ctx.style);
                    ctx.draw_rect(next_layout, ctx.style.colors[i]);
                    i += 1;
                }
            },
        );
    }

    fn process_frame(
        &mut self,
        ctx: &mut ui::Context<Pass, system::Renderer>,
        width: usize,
        height: usize,
    ) -> Pass {
        ctx.frame(width, height, |ctx| {
            self.style_window(ctx);
            self.log_window(ctx);
            self.test_window(ctx);
        })
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

    window.set_all_polling(true);
    window.make_current();
    glfw.set_swap_interval(glfw::SwapInterval::Sync(0));

    let mut driver = renderer::get_driver();

    // initialize UI
    let mut state = State::new();

    let (width, height) = window.get_framebuffer_size();
    let renderer = system::Renderer::new(&mut driver, width as u32, height as u32);
    let mut input = Input::new();
    let mut ctx = ui::Context::new(renderer);

    'running: while !window.should_close() {
        let (width, height) = window.get_framebuffer_size();

        let mut pass = state.process_frame(&mut ctx, width as _, height as _);

        driver.render_pass(&mut pass);
        window.swap_buffers();

        glfw.wait_events_timeout(0.007);
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Close | glfw::WindowEvent::Key(glfw::Key::Escape, ..) => {
                    break 'running
                }

                _ => input.handle_event(event, &mut window, &mut ctx),
            }
        }
    }

    window.close()
}
