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
use crate::renderer::*;
use crate::rs_math3d::*;
use crate::*;

use super::*;
use std::collections::HashMap;
use std::sync::*;

render_data! {
    vertex Vertex {
        a_pos   : Vec2f,
        a_tc    : Vec2f,
        s_rgba  : Color4b,

    }

    uniforms Uniforms {
        u_screen_size   : Vec2f,
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            a_pos: Vec2f::new(0.0, 0.0),
            a_tc: Vec2f::new(0.0, 0.0),
            s_rgba: color4b(0, 0, 0, 0),
        }
    }
}

#[derive(Default, Clone)]
struct PaintTexture {
    size: (usize, usize),

    /// Pending upload (will be emptied later).
    pixels: Vec<Color4b>,

    /// Lazily uploaded
    texture: Option<TexturePtr>,

    /// User textures can be modified and this flag
    /// is used to indicate if pixel data for the
    /// texture has been updated.
    dirty: bool,
}

const VS_SRC: &str = r#"
    #version 300 es
    uniform vec2 u_screen_size;
    in highp vec2 a_pos;
    in highp vec4 s_rgba;
    in highp vec2 a_tc;
    in highp vec3 b;
    out highp vec4 v_rgba;
    out vec2 v_tc;

    void main() {
        gl_Position = vec4(
            2.0 * a_pos.x / u_screen_size.x - 1.0,
            1.0 - 2.0 * a_pos.y / u_screen_size.y,
            0.0,
            1.0);
        v_rgba = s_rgba;
        v_tc = a_tc;
    }
"#;

const FS_SRC: &str = r#"
    #version 300 es
    uniform lowp sampler2D u_sampler;
    in highp vec4 v_rgba;
    in highp vec2 v_tc;
    in highp vec3 v_b;
    layout(location = 0) out lowp vec4 f_color;

    void main() {
        highp vec4 tcol = texture(u_sampler, v_tc).rrrr;
        f_color = tcol * v_rgba;
    }
"#;

const MAX_VERTEX_COUNT: usize = 65536;
const MAX_INDEX_COUNT: usize = 65536;

pub struct Painter {
    driver: DriverPtr,
    pipeline: PipelinePtr,
    vertex_buffer: DeviceBufferPtr,
    index_buffer: DeviceBufferPtr,

    canvas_width: u32,
    canvas_height: u32,
    ui_texture: TexturePtr,

    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl Painter {
    pub fn new(drv: &mut DriverPtr, canvas_width: u32, canvas_height: u32) -> Painter {
        let program = drv
            .create_shader(ShaderDesc {
                vertex_shader: String::from(VS_SRC),
                pixel_shader: String::from(FS_SRC),

                vertex_attributes: vec![Vertex::get_attribute_names()],
                vertex_uniforms: Uniforms::get_uniform_names(),
                vertex_surfaces: vec![],

                pixel_uniforms: vec![],
                pixel_surfaces: vec![String::from("u_sampler")],
            })
            .unwrap();

        let vertex_layout = VertexBufferLayout {
            buffer_id: 0,
            vertex_attributes: Vertex::get_attribute_descriptors(),
            stride: Vertex::stride(),
            divisor: 0,
        };

        let pipeline_desc = PipelineDesc {
            primitive_type: PrimitiveType::Triangles,
            shader: program,
            buffer_layouts: vec![vertex_layout.clone()],
            uniform_descs: Uniforms::get_uniform_descriptors(),
            index_type: IndexType::UInt16,
            face_winding: FaceWinding::CCW,
            cull_mode: CullMode::None,
            depth_write: true,
            depth_test: false,
            blend: BlendOp::Add(Blend::default()),
            polygon_offset: PolygonOffset::None,
        };

        let pipeline = drv.create_pipeline(pipeline_desc).unwrap();

        let vertex_buffer = drv
            .create_device_buffer(DeviceBufferDesc::Vertex(Usage::Dynamic(
                MAX_VERTEX_COUNT * std::mem::size_of::<Vertex>(),
            )))
            .unwrap();

        let index_buffer = drv
            .create_device_buffer(DeviceBufferDesc::Vertex(Usage::Dynamic(
                MAX_INDEX_COUNT * std::mem::size_of::<u16>(),
            )))
            .unwrap();

        let tex_desc = TextureDesc {
            sampler_desc: SamplerDesc::default(ATLAS_WIDTH as usize, ATLAS_HEIGHT as usize)
                .with_pixel_format(PixelFormat::R8(
                    MinMagFilter::default()
                        .with_mag_filter(Filter::Nearest)
                        .with_min_filter(Filter::Nearest),
                ))
                .with_wrap_mode(WrapMode::ClampToEdge),
            payload: Some(Arc::new(ATLAS_TEXTURE.to_vec())),
        };

        let ui_texture = drv.create_texture(tex_desc).unwrap();
        Painter {
            driver: drv.clone(),
            pipeline,
            canvas_width,
            canvas_height,
            vertex_buffer,
            index_buffer,
            ui_texture,
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn set_canvas_size(&mut self, width: u32, height: u32) {
        self.canvas_width = width;
        self.canvas_height = height;
    }

    fn push_quad_vertices(
        &mut self,
        pass: &mut Pass,
        v0: &Vertex,
        v1: &Vertex,
        v2: &Vertex,
        v3: &Vertex,
    ) {
        if self.vertices.len() + 4 >= MAX_VERTEX_COUNT || self.indices.len() + 6 >= MAX_INDEX_COUNT
        {
            self.flush(pass);
        }

        let is = self.vertices.len() as u16;
        self.indices.push(is + 0);
        self.indices.push(is + 1);
        self.indices.push(is + 2);
        self.indices.push(is + 2);
        self.indices.push(is + 3);
        self.indices.push(is + 0);

        self.vertices.push(v0.clone());
        self.vertices.push(v1.clone());
        self.vertices.push(v2.clone());
        self.vertices.push(v3.clone());
    }

    pub fn push_rect(&mut self, pass: &mut Pass, dst: Recti, src: Recti, color: Color4b) {
        let x = src.x as f32 / ATLAS_WIDTH as f32;
        let y = src.y as f32 / ATLAS_HEIGHT as f32;
        let w = src.width as f32 / ATLAS_WIDTH as f32;
        let h = src.height as f32 / ATLAS_HEIGHT as f32;

        let mut v0 = Vertex::default();
        let mut v1 = Vertex::default();
        let mut v2 = Vertex::default();
        let mut v3 = Vertex::default();

        // tex coordinates
        v0.a_tc.x = x;
        v0.a_tc.y = y;
        v1.a_tc.x = x + w;
        v1.a_tc.y = y;
        v2.a_tc.x = x + w;
        v2.a_tc.y = y + h;
        v3.a_tc.x = x;
        v3.a_tc.y = y + h;

        // position
        v0.a_pos.x = dst.x as f32;
        v0.a_pos.y = dst.y as f32;
        v1.a_pos.x = dst.x as f32 + dst.width as f32;
        v1.a_pos.y = dst.y as f32;
        v2.a_pos.x = dst.x as f32 + dst.width as f32;
        v2.a_pos.y = dst.y as f32 + dst.height as f32;
        v3.a_pos.x = dst.x as f32;
        v3.a_pos.y = dst.y as f32 + dst.height as f32;

        // color
        v0.s_rgba = color4b(color.x, color.y, color.z, color.w);
        v1.s_rgba = v0.s_rgba;
        v2.s_rgba = v0.s_rgba;
        v3.s_rgba = v0.s_rgba;

        self.push_quad_vertices(pass, &v0, &v1, &v2, &v3);
    }

    pub fn draw_rect(&mut self, pass: &mut Pass, rect: Recti, color: Color4b) {
        self.push_rect(pass, rect, ATLAS[ATLAS_WHITE as usize], color);
    }

    pub fn draw_text(&mut self, pass: &mut Pass, text: &str, pos: Vec2i, color: Color4b) {
        let mut dst = Rect::new(pos.x, pos.y, 0, 0);
        for p in text.chars() {
            if (p as usize) < 127 {
                let chr = usize::min(p as usize, 127);
                let src = ATLAS[ATLAS_FONT as usize + chr];
                dst.width = src.width;
                dst.height = src.height;
                self.push_rect(pass, dst, src, color);
                dst.x += dst.width;
            }
        }
    }

    pub fn draw_icon(&mut self, pass: &mut Pass, id: Icon, r: Recti, color: Color4b) {
        let src = ATLAS[id as usize];
        let x = r.x + (r.width - src.width) / 2;
        let y = r.y + (r.height - src.height) / 2;
        self.push_rect(pass, Rect::new(x, y, src.width, src.height), src, color);
    }

    pub fn set_clip_rect(&mut self, pass: &mut Pass, width: u32, height: u32, rect: Recti) {
        self.canvas_width = width as u32;
        self.canvas_height = height as u32;
        self.flush(pass);
        pass.set_scissor(
            rect.x as u32,
            (height as i32 - (rect.y + rect.height)) as u32,
            rect.width as u32,
            rect.height as u32,
        );
    }

    pub fn get_char_width(&self, _font: FontId, c: char) -> usize {
        ATLAS[ATLAS_FONT as usize + c as usize].width as usize
    }

    pub fn get_font_height(&self, _font: FontId) -> usize {
        18
    }

    pub fn paint(&mut self, pass: &mut Pass, ctx: &mut super::Context) {
        pass.set_viewport(0, 0, self.canvas_width, self.canvas_height);

        let mut cmd_id = 0;
        loop {
            match ctx.mu_next_command(cmd_id) {
                Some((command, id)) => {
                    match command {
                        Command::Text {
                            str_start,
                            str_len,
                            pos,
                            color,
                            ..
                        } => {
                            let str = &ctx.text_stack[str_start..str_start + str_len];
                            self.draw_text(pass, str, pos, color);
                        }
                        Command::Rect { rect, color } => {
                            self.draw_rect(pass, rect, color);
                        }
                        Command::Icon { id, rect, color } => {
                            self.draw_icon(pass, id, rect, color);
                        }
                        Command::Clip { rect } => {
                            self.set_clip_rect(pass, self.canvas_width, self.canvas_height, rect);
                        }
                        _ => {}
                    }
                    cmd_id = id;
                }
                None => break,
            }
        }

        self.flush(pass);
    }

    fn flush(&mut self, pass: &mut Pass) {
        if self.vertices.len() != 0 && self.indices.len() != 0 {
            pass.update_device_buffer(&mut self.vertex_buffer, 0, Arc::new(self.vertices.clone()));
            pass.update_device_buffer(&mut self.index_buffer, 0, Arc::new(self.indices.clone()));

            let bindings = Bindings {
                vertex_buffers: vec![self.vertex_buffer.clone()],
                index_buffer: Some(self.index_buffer.clone()),

                vertex_images: Vec::new(),
                pixel_images: Vec::from([self.ui_texture.clone()]),
            };

            let u = Uniforms {
                u_screen_size: Vec2f::new(self.canvas_width as f32, self.canvas_height as f32),
            };
            pass.draw(
                &self.pipeline,
                &bindings,
                Arc::new(GenPayload::from(u)),
                (self.indices.len() / 3) as u32,
                1,
            );
        }
        self.vertices.clear();
        self.indices.clear();
    }
}
