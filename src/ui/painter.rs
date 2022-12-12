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
use crate::*;
use crate::rs_math3d::*;
use crate::renderer::*;

use std::collections::HashMap;
use std::sync::*;
use super::*;


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
        highp vec4 tcol = texture(u_sampler, v_tc);
        f_color = tcol * v_rgba;
    }
"#;

const MAX_ELEM_COUNT: usize = 65536 * 4;

pub struct Painter {
    driver: DriverPtr,
    pipeline: PipelinePtr,
    vertex_buffer: DeviceBufferPtr,
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
            index_type: IndexType::None,
            face_winding: FaceWinding::CCW,
            cull_mode: CullMode::None,
            depth_write: true,
            depth_test: false,
            blend: BlendOp::Add(Blend::default()),
            polygon_offset: PolygonOffset::None,
        };

        let pipeline = drv.create_pipeline(pipeline_desc).unwrap();

        let vertex_buffer = drv
            .create_device_buffer(DeviceBufferDesc::Vertex(Usage::Dynamic(MAX_ELEM_COUNT * std::mem::size_of::<Vertex>())))
            .unwrap();
        Painter {
            driver: drv.clone(),
            pipeline,
            canvas_width,
            canvas_height,
            vertex_buffer,
            ui_texture: Default::default(),
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn set_canvas_size(&mut self, width: u32, height: u32) {
        self.canvas_width = width;
        self.canvas_height = height;
    }

    fn push_quad_vertices(&mut self, v0: &Vertex, v1: &Vertex, v2: &Vertex, v3: &Vertex) {
        if self.verts.len() + 4 >= 65536 || self.indices.len() + 6 >= 65536 {
            self.flush(gl);
        }

        let is = self.verts.len() as u16;
        self.indices.push(is + 0);
        self.indices.push(is + 1);
        self.indices.push(is + 2);
        self.indices.push(is + 2);
        self.indices.push(is + 3);
        self.indices.push(is + 0);

        self.verts.push(v0.clone());
        self.verts.push(v1.clone());
        self.verts.push(v2.clone());
        self.verts.push(v3.clone());
    }

    pub fn push_rect(&mut self, dst: Recti, src: Recti, color: Color4b) {
        let x = src.x as f32 / ATLAS_WIDTH as f32;
        let y = src.y as f32 / ATLAS_HEIGHT as f32;
        let w = src.w as f32 / ATLAS_WIDTH as f32;
        let h = src.h as f32 / ATLAS_HEIGHT as f32;

        let mut v0 = Vertex::default();
        let mut v1 = Vertex::default();
        let mut v2 = Vertex::default();
        let mut v3 = Vertex::default();

        // tex coordinates
        v0.tex.x = x;
        v0.tex.y = y;
        v1.tex.x = x + w;
        v1.tex.y = y;
        v2.tex.x = x + w;
        v2.tex.y = y + h;
        v3.tex.x = x;
        v3.tex.y = y + h;

        // position
        v0.pos.x = dst.x as f32;
        v0.pos.y = dst.y as f32;
        v1.pos.x = dst.x as f32 + dst.w as f32;
        v1.pos.y = dst.y as f32;
        v2.pos.x = dst.x as f32 + dst.w as f32;
        v2.pos.y = dst.y as f32 + dst.h as f32;
        v3.pos.x = dst.x as f32;
        v3.pos.y = dst.y as f32 + dst.h as f32;

        // color
        v0.color = color4b(color.r, color.g, color.b, color.a);
        v1.color = v0.color;
        v2.color = v0.color;
        v3.color = v0.color;

        self.push_quad_vertices(&v0, &v1, &v2, &v3);
    }

    pub fn draw_rect(&mut self, rect: Recti, color: Color4b) {
        self.push_rect(rect, ATLAS[ATLAS_WHITE as usize], color);
    }

    pub fn draw_text(&mut self, text: &str, pos: Vec2i, color: Color4b) {
        let mut dst = Rect { x: pos.x, y: pos.y, w: 0, h: 0 };
        for p in text.chars() {
            if (p as usize) < 127 {
                let chr = usize::min(p as usize, 127);
                let src = ATLAS[ATLAS_FONT as usize + chr];
                dst.w = src.w;
                dst.h = src.h;
                self.push_rect(dst, src, color);
                dst.x += dst.w;
            }
        }
    }

    pub fn draw_icon(&mut self, id: Icon, r: Recti, color: Color4b) {
        let src = ATLAS[id as usize];
        let x = r.x + (r.w - src.w) / 2;
        let y = r.y + (r.h - src.h) / 2;
        self.push_rect(rect(x, y, src.w, src.h), src, color);
    }

    pub fn get_char_width(&self, _font: FontId, c: char) -> usize {
        ATLAS[ATLAS_FONT as usize + c as usize].w as usize
    }

    pub fn get_font_height(&self, _font: FontId) -> usize {
        18
    }

    pub fn paint(&mut self, pass: &mut Pass, ctx: &mut super::Context) {
        let screen_size_pixels = vec2(self.canvas_width as f32, self.canvas_height as f32);
        let screen_size_points = screen_size_pixels / pixels_per_point;

        for ClippedPrimitive { clip_rect, primitive } in meshes {
            let clip_min_x = pixels_per_point * clip_rect.min.x;
            let clip_min_y = pixels_per_point * clip_rect.min.y;
            let clip_max_x = pixels_per_point * clip_rect.max.x;
            let clip_max_y = pixels_per_point * clip_rect.max.y;
            let clip_min_x = clip_min_x.clamp(0.0, screen_size_pixels.x);
            let clip_min_y = clip_min_y.clamp(0.0, screen_size_pixels.y);
            let clip_max_x = clip_max_x.clamp(clip_min_x, screen_size_pixels.x);
            let clip_max_y = clip_max_y.clamp(clip_min_y, screen_size_pixels.y);
            let clip_min_x = clip_min_x.round() as i32;
            let clip_min_y = clip_min_y.round() as i32;
            let clip_max_x = clip_max_x.round() as i32;
            let clip_max_y = clip_max_y.round() as i32;

            let r = Recti::new(
                clip_min_x,
                self.canvas_height as i32 - clip_max_y,
                clip_max_x - clip_min_x,
                clip_max_y - clip_min_y,
            );

            //scissor Y coordinate is from the bottom
            pass.set_scissor(r.x as u32, r.y as u32, r.width as u32, r.height as u32);

            match primitive {
                Primitive::Mesh(mesh) => {
                    if mesh.vertices.len() > 0 {
                        self.paint_mesh(pass, &mesh, Vec2f::new(screen_size_points.x, screen_size_points.y));
                    }
                }
                Primitive::Callback(cb) => {
                    // Transform callback rect to physical pixels:
                    let rect_min_x = pixels_per_point * cb.rect.min.x;
                    let rect_min_y = pixels_per_point * cb.rect.min.y;
                    let rect_max_x = pixels_per_point * cb.rect.max.x;
                    let rect_max_y = pixels_per_point * cb.rect.max.y;

                    let rect_min_x = rect_min_x.clamp(0.0, screen_size_pixels.x);
                    let rect_min_y = rect_min_y.clamp(0.0, screen_size_pixels.y);
                    let rect_max_x = rect_max_x.clamp(rect_min_x, screen_size_pixels.x);
                    let rect_max_y = rect_max_y.clamp(rect_min_y, screen_size_pixels.y);

                    let rect_min_x = rect_min_x.round() as u32;
                    let rect_min_y = rect_min_y.round() as u32;
                    let rect_max_x = rect_max_x.round() as u32;
                    let rect_max_y = rect_max_y.round() as u32;

                    let r = Recti::new(
                        rect_min_x as i32,
                        rect_min_y as i32,
                        (rect_max_x - rect_min_x) as i32,
                        (rect_max_y - rect_min_y) as i32,
                    );

                    pass.set_viewport(r.x as _, r.y as _, r.width as _, r.height as _);

                    let viewport = egui::Rect::from_min_size(egui::Pos2::new(r.x as f32, r.y as f32), egui::Vec2::new(r.width as f32, r.height as f32));

                    let info = egui::PaintCallbackInfo {
                        viewport,
                        clip_rect: clip_rect,
                        pixels_per_point,
                        screen_size_px: [screen_size_pixels.x as u32, screen_size_pixels.y as u32],
                    };

                    (cb.callback.downcast_ref::<CallbackFn>().unwrap().paint)(&info, pass);

                    pass.set_viewport(0, 0, self.canvas_width, self.canvas_height);
                }
            }
        }

        for id in &egui_texture.free {
            match id {
                egui::TextureId::Managed(id) => {
                    self.egui_textures.remove(id);
                }
                _ => (),
            }
        }
    }

    fn paint_mesh(&mut self, pass: &mut Pass, vertices: &Vec<Vertex>, indices: &Vec<u16>, screen_size: Vec2f) {
        let max_part_size = 3 * 4096;
        let index_count = mesh.indices.len();
        let parts = mesh.indices.len() / max_part_size;

        for i in 0..parts + 1 {
            let remaining_count = index_count - i * max_part_size;
            if remaining_count > 0 {
                let part_index_count = usize::min(remaining_count, max_part_size);
                let vs = vertices[i * max_part_size..i * max_part_size + part_index_count].to_vec();

                let tri_count = vs.len() / 3;
                pass.update_device_buffer(&mut self.vertex_buffer, 0, Arc::new(vs));

                let bindings = Bindings {
                    vertex_buffers: vec![self.vertex_buffer.clone()],
                    index_buffer: None,

                    vertex_images: Vec::new(),
                    pixel_images: Vec::from([self.get_texture(mesh.texture_id)]),
                };

                let u = Uniforms { u_screen_size: screen_size };
                pass.draw(&self.pipeline, &bindings, Arc::new(GenPayload::from(u)), tri_count as u32, 1);
            }
        }
    }
}
