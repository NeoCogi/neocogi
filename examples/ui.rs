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

use neocogi::rs_math3d::*;
use neocogi::*;
use std::sync::Arc;

use neocogi::renderer::*;

use neocogi::ui;
use ui::*;

static VERTEX_SHADER: &'static str = "
#version 300 es
in  highp   vec4        position;
in  highp   vec4        color;
out highp   vec4        v_color;
void main() {
    gl_Position = position;
    v_color     = color;
}";

static PIXEL_SHADER: &'static str = "
#version 300 es
precision mediump float;
        in highp    vec4        v_color;
layout(location = 0) out lowp  vec4     color_buffer;
void main() {
    highp vec4 col  = v_color;
    color_buffer    = col;
}";

render_data! {
    vertex Vertex {
        position: Vec4f,
        color   : Vec4f,
    }
}

struct State<'a> {
    label_colors: [LabelColor<'a>; 14],
    bg: [Real; 3],
    logbuf: String,
    logbuf_updated: bool,
    submit_buf: String,
    checks: [bool; 3],
    colors: [Color4b; 14],
    tri_pipeline: Option<PipelinePtr>,
    vb: Option<DeviceBufferPtr>,
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
            ],
            bg: [90.0, 95.0, 100.0],
            logbuf: String::new(),
            logbuf_updated: false,
            submit_buf: String::new(),
            checks: [false, true, false],
            colors: [color4b(0, 0, 0, 0); 14],
            tri_pipeline: None,
            vb: None,
        }
    }

    fn init_render_objects(driver: &mut DriverPtr) -> PipelinePtr {
        let mut model_attribs = Vec::new();
        model_attribs.push(Vertex::get_attribute_names());

        let model_shader_desc = ShaderDesc {
            vertex_shader: String::from(VERTEX_SHADER),
            pixel_shader: String::from(PIXEL_SHADER),

            vertex_attributes: model_attribs,
            vertex_uniforms: Vec::new(),
            vertex_surfaces: Vec::new(),

            pixel_uniforms: Vec::new(),
            pixel_surfaces: Vec::new(),
        };

        let model_program = driver.create_shader(model_shader_desc).unwrap();

        let vertex_layout = VertexBufferLayout {
            buffer_id: 0,
            vertex_attributes: Vertex::get_attribute_descriptors(),
            stride: Vertex::stride(),
            divisor: 0,
        };

        let tri_pipeline_desc = PipelineDesc {
            primitive_type: PrimitiveType::Triangles,
            shader: model_program.clone(),
            buffer_layouts: vec![vertex_layout.clone()],
            uniform_descs: Vec::new(),
            index_type: IndexType::None,
            face_winding: FaceWinding::CCW,
            cull_mode: CullMode::None,
            depth_write: true,
            depth_test: true,
            blend: BlendOp::None,
            polygon_offset: PolygonOffset::None,
        };

        driver.create_pipeline(tri_pipeline_desc).unwrap()
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

    fn tri_window(
        &mut self,
        driver: &mut DriverPtr,
        ctx: &mut ui::Context<PassCommandQueue, system::Renderer>,
    ) {
        match self.tri_pipeline {
            Some(_) => (),
            None => {
                let pipleine = Self::init_render_objects(driver);
                self.tri_pipeline = Some(pipleine);
            }
        }

        match self.vb {
            Some(_) => (),
            None => {
                let vertex_buffer = driver
                    .create_device_buffer(DeviceBufferDesc::Vertex(Usage::Dynamic(
                        3 * std::mem::size_of::<Vertex>(),
                    )))
                    .unwrap();
                self.vb = Some(vertex_buffer);
            }
        }

        ctx.window(
            "Triangle Window",
            Rect::new(40, 500, 300, 300),
            WidgetOption::NONE,
            |ctx| {
                ctx.column(|ctx| {
                    let mut win = ctx.get_current_container_rect();
                    let bindings = Bindings {
                        vertex_buffers: vec![self.vb.as_ref().unwrap().clone()],
                        index_buffer: None,

                        vertex_images: Vec::new(),
                        pixel_images: Vec::new(),
                    };

                    let vertices = vec![
                        Vertex {
                            position: Vec4f::new(-0.5, -0.5, 0.0, 1.0),
                            color: Vec4f::new(1.0, 0.0, 0.0, 1.0),
                        },
                        Vertex {
                            position: Vec4f::new(0.5, -0.5, 0.0, 1.0),
                            color: Vec4f::new(0.0, 0.0, 1.0, 1.0),
                        },
                        Vertex {
                            position: Vec4f::new(0.0, 0.5, 0.0, 1.0),
                            color: Vec4f::new(0.0, 1.0, 0.0, 1.0),
                        },
                    ];

                    let size = ctx.frame_size();
                    ctx.render_custom(|pass, clip| {
                        pass.set_viewport(
                            win.x,
                            size.1 as i32 - win.height - win.y,
                            clip.width as _,
                            win.height as u32,
                        );
                        pass.update_device_buffer(
                            self.vb.as_mut().unwrap(),
                            0,
                            Arc::new(vertices.to_vec()),
                        );
                        pass.draw(
                            self.tri_pipeline.as_ref().unwrap(),
                            &bindings,
                            Arc::new(Vec::<Vec3f>::new()),
                            1,
                            1,
                        );
                        pass.set_viewport(0, 0, size.0 as _, size.1 as _);
                    });
                });
            },
        );
    }

    fn test_window(&mut self, ctx: &mut ui::Context<PassCommandQueue, system::Renderer>) {
        ctx
        .window(
                "Demo Window",
                Rect::new(40, 40, 300, 450),
                WidgetOption::NONE,
                |ctx|  {
            let mut win = ctx.get_current_container_rect();
            win.width = if win.width > 240 { win.width } else { 240 };
            win.height = if win.height > 300 { win.height } else { 300 };

            ctx.set_current_container_rect(&win);

            let mut buff = String::new();
            let style = ctx.style;

            ctx.header("Window Info", WidgetOption::NONE, |ctx| {
                let win_0 = ctx.get_current_container_rect();
                ctx.rows_with_line_config(&[96, -1], 0, |ctx| {
                    ctx.label("Position:");

                    buff.clear();
                    buff.append_int(10, 0, win_0.x);
                    buff.push_str(", ");
                    buff.append_int(10, 0, win_0.y);

                    ctx.label(buff.as_str());
                    buff.clear();
                    ctx.label("Size:");

                    buff.append_int(10, 0, win_0.width);
                    buff.push_str(", ");
                    buff.append_int(10, 0, win_0.height);

                    ctx.label(buff.as_str());
                });
            });
            ctx
            .header( "Test Buttons", WidgetOption::EXPANDED, |ctx| {
                ctx.rows_with_line_config(&[120, -110, -1], 0, |ctx| {
                    ctx.label("Test buttons 1:");
                    if !ctx
                        .button("Button 1", None, WidgetOption::ALIGN_CENTER)
                        .is_none()
                    {
                        self.write_log("Pressed button 1");
                    }
                    if !ctx
                        .button("Button 2", None, WidgetOption::ALIGN_CENTER)
                        .is_none()
                    {
                        self.write_log("Pressed button 2");
                    }
                    ctx.label("Test buttons 2:");
                    if !ctx
                        .button("Button 3", None, WidgetOption::ALIGN_CENTER)
                        .is_none()
                    {
                        self.write_log("Pressed button 3");
                    }
                    if !ctx
                        .button("Popup", None, WidgetOption::ALIGN_CENTER)
                        .is_none()
                    {
                        ctx.open_popup("Test Popup");
                    }

                    ctx.popup("Test Popup", Recti::new(0, 0, 90, 90), |ctx| {
                        if !ctx
                            .button("Hello", None, WidgetOption::ALIGN_CENTER)
                            .is_none()
                        {
                            self.write_log("Hello")
                        }
                        if !ctx
                            .button("World", None, WidgetOption::ALIGN_CENTER)
                            .is_none()
                        {
                            self.write_log("World")
                        }
                    });
                });
            });
            ctx.header( "Tree and Text", WidgetOption::EXPANDED, |ctx| {
                ctx.rows_with_line_config(&[140, -1], 0, |ctx| {
                    ctx.column(|ctx| {
                        ctx.treenode("Test 1", WidgetOption::NONE, |ctx| {
                            ctx.treenode("Test 1a", WidgetOption::NONE, |ctx| {
                                ctx.label("Hello");
                                ctx.label("world");
                            });
                            ctx.treenode("Test 1b", WidgetOption::NONE, |ctx| {
                                if !ctx
                                    .button("Button 1", None, WidgetOption::ALIGN_CENTER)
                                    .is_none()
                                {
                                    self.write_log("Pressed button 1");
                                }
                                if !ctx
                                    .button("Button 2", None, WidgetOption::ALIGN_CENTER)
                                    .is_none()
                                {
                                    self.write_log("Pressed button 2");
                                }
                            });
                        });
                        ctx.treenode("Test 2", WidgetOption::NONE, |ctx| {
                            ctx.rows_with_line_config(&[54, 54], 0, |ctx| {
                                if !ctx
                                    .button("Button 3", None, WidgetOption::ALIGN_CENTER)
                                    .is_none()
                                {
                                    self.write_log("Pressed button 3");
                                }
                                if !ctx
                                    .button("Button 4", None, WidgetOption::ALIGN_CENTER)
                                    .is_none()
                                {
                                    self.write_log("Pressed button 4");
                                }
                                if !ctx
                                    .button("Button 5", None, WidgetOption::ALIGN_CENTER)
                                    .is_none()
                                {
                                    self.write_log("Pressed button 5");
                                }
                                if !ctx
                                    .button("Button 6", None, WidgetOption::ALIGN_CENTER)
                                    .is_none()
                                {
                                    self.write_log("Pressed button 6");
                                }
                            });
                        });
                        ctx.treenode("Test 3", WidgetOption::NONE, |ctx| {
                            ctx.checkbox("Checkbox 1", &mut self.checks[0]);
                            ctx.checkbox("Checkbox 2", &mut self.checks[1]);
                            ctx.checkbox("Checkbox 3", &mut self.checks[2]);
                        });
                    });
                    ctx.column(|ctx| {
                        ctx.rows_with_line_config(&[-1], 0, |ctx| {
                            ctx.text(
                                "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Maecenas lacinia, sem eu lacinia molestie, mi risus faucibus ipsum, eu varius magna felis a nulla."
                                ,
                            );
                        });
                    });
                });
            });
            ctx.header("Background Color", WidgetOption::EXPANDED, |ctx| {
                ctx.rows_with_line_config(&[-78, -1], 74, |ctx| {
                    ctx.column(|ctx| {
                        ctx.rows_with_line_config(&[46, -1], 0, |ctx| {
                            ctx.label("Red:");
                            ctx.slider_ex(
                                &mut self.bg[0],
                                0.0,
                                255.0,
                                0.0,
                                0,
                                WidgetOption::ALIGN_CENTER,
                            );
                            ctx.label("Green:");
                            ctx.slider_ex(
                                &mut self.bg[1],
                                0.0,
                                255.0,
                                0.0,
                                0,
                                WidgetOption::ALIGN_CENTER,
                            );
                            ctx.label("Blue:");
                            ctx.slider_ex(
                                &mut self.bg[2],
                                0.0,
                                255.0,
                                0.0,
                                0,
                                WidgetOption::ALIGN_CENTER,
                            );
                        });
                    });
                    let r = ctx.next_cell();
                    ctx.draw_rect(
                        r,
                        color(self.bg[0] as u8, self.bg[1] as u8, self.bg[2] as u8, 255),
                    );
                    let mut buff = String::new();
                    buff.push_str("#");
                    buff.append_int(16, 2, self.bg[0] as _);
                    buff.append_int(16, 2, self.bg[1] as _);
                    buff.append_int(16, 2, self.bg[2] as _);
                    let font = FontId(0);
                    ctx.draw_control_text(
                        font,
                        buff.as_str(),
                        r,
                        ControlColor::Text,
                        WidgetOption::ALIGN_CENTER,
                    );
                });
            });
        });
    }

    fn log_window(&mut self, ctx: &mut ui::Context<PassCommandQueue, system::Renderer>) {
        ctx.window(
            "Log Window",
            Rect::new(350, 40, 300, 200),
            WidgetOption::NONE,
            |ctx| {
                ctx.rows_with_line_config(&[-1], -25, |ctx| {
                    ctx.panel("Log Output", WidgetOption::NONE, |ctx| {
                        let mut scroll = ctx.get_current_container_scroll();
                        let content_size = ctx.get_current_container_content_size();
                        ctx.rows_with_line_config(&[-1], -1, |ctx| {
                            ctx.text(self.logbuf.as_str());
                            if self.logbuf_updated {
                                scroll.y = content_size.y;
                                ctx.set_current_container_scroll(&scroll);
                                self.logbuf_updated = false;
                            }
                        });
                    });

                    let mut submitted = false;
                    ctx.rows_with_line_config(&[-70, -1], 0, |ctx| {
                        if ctx
                            .textbox_ex(&mut self.submit_buf, WidgetOption::NONE)
                            .is_submitted()
                        {
                            ctx.set_focus(ctx.last_id);
                            submitted = true;
                        }
                        if !ctx
                            .button("Submit", None, WidgetOption::ALIGN_CENTER)
                            .is_none()
                        {
                            submitted = true;
                        }
                        if submitted {
                            let mut buf = String::new();
                            buf.push_str(self.submit_buf.as_str());
                            self.write_log(buf.as_str());
                            self.submit_buf.clear();
                        }
                    });
                });
            },
        );
    }
    fn uint8_slider(
        ctx: &mut ui::Context<PassCommandQueue, system::Renderer>,
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
            0.0,
            0,
            WidgetOption::ALIGN_CENTER,
        );
        *value = tmp as u8;
        ctx.pop_id();
        return res;
    }
    fn style_window(&mut self, ctx: &mut ui::Context<PassCommandQueue, system::Renderer>) {
        ctx.window(
            "Style Editor",
            Rect::new(350, 250, 300, 240),
            WidgetOption::NONE,
            |ctx| {
                let style = ctx.style;
                let sw = (ctx.get_current_container_body().width as f64 * 0.14f64) as i32;
                ctx.rows_with_line_config(&[80, sw, sw, sw, sw, -1], 0, |ctx| {
                    for i in 0..self.label_colors.len() {
                        ctx.label(self.label_colors[i].label);
                        let color = &mut self.colors[i];
                        Self::uint8_slider(ctx, &mut color.x, 0, 255);
                        Self::uint8_slider(ctx, &mut color.y, 0, 255);
                        Self::uint8_slider(ctx, &mut color.z, 0, 255);
                        Self::uint8_slider(ctx, &mut color.w, 0, 255);
                        ctx.style.colors[i] = *color;
                        let r = ctx.next_cell();
                        ctx.draw_rect(r, ctx.style.colors[i]);
                    }
                });
            },
        );
    }

    fn process_frame(
        &mut self,
        drv: &mut DriverPtr,
        ctx: &mut ui::Context<PassCommandQueue, system::Renderer>,
    ) {
        self.style_window(ctx);
        self.log_window(ctx);
        self.tri_window(drv, ctx);
        self.test_window(ctx);
    }
}

fn main() {
    // initialize GLFW3 with OpenGL ES 3.0
    let mut app = system::App::new();
    // initialize UI
    let mut state = State::new();

    let style = Style::default();
    for i in 0..state.colors.len() {
        state.colors[i] = style.colors[i];
    }

    app.run(|drv, ctx| {
        state.process_frame(drv, ctx);
    });
}
