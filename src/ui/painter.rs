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
//
// Copyright (c) 2021 cohaereo
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
//
use crate::*;
use crate::rs_math3d::*;
use crate::renderer::*;

use std::collections::HashMap;

use rs_ctypes::*;

use ::egui::{
    epaint::{Color32, Mesh},
    vec2, ClippedMesh,
};

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
struct UserTexture {
    size: (usize, usize),

    /// Pending upload (will be emptied later).
    pixels: Vec<u8>,

    /// Lazily uploaded
    texture: Option<TexturePtr>,

    /// For user textures there is a choice between
    /// Linear (default) and Nearest.
    filtering: bool,

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

const MAX_ELEM_COUNT : usize = 65536 * 4;

pub struct Painter {
    driver  : DriverPtr,
    pipeline: PipelinePtr,
    index_buffer: DeviceBufferPtr,
    vertex_buffer: DeviceBufferPtr,
    canvas_width: u32,
    canvas_height: u32,
    egui_textures: HashMap<u64, TexturePtr>,
    user_textures: HashMap<u64, UserTexture>,
}

impl Painter {
    pub fn new(
        drv: &mut DriverPtr,
        window: &mut glfw::Window,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Painter {

        let program = drv.create_shader(ShaderDesc {
            vertex_shader       : String::from(VS_SRC),
            pixel_shader        : String::from(FS_SRC),

            vertex_attributes   : vec!{
               Vertex::get_attribute_names(),
            },
            vertex_uniforms     : Uniforms::get_uniform_names(),
            vertex_surfaces     : vec!{},

            pixel_uniforms      : vec!{},
            pixel_surfaces      : vec!{ String::from("u_sampler") }
        }).unwrap();

        let vertex_layout = VertexBufferLayout {
            buffer_id           : 0,
            vertex_attributes   : Vertex::get_attribute_descriptors(),
            stride              : Vertex::stride(),
            divisor             : 0,
        };

        let pipeline_desc = PipelineDesc {
            primitive_type      : PrimitiveType::Triangles,
            shader              : program,
            buffer_layouts      : vec! { vertex_layout.clone() },
            uniform_descs       : Uniforms::get_uniform_descriptors(),
            index_type          : IndexType::UInt16,
            face_winding        : FaceWinding::CCW,
            cull_mode           : CullMode::None,
            depth_write         : true,
            depth_test          : false,
            blend               : BlendOp::Add(Blend::default()),
        };

        let pipeline = drv.create_pipeline(pipeline_desc).unwrap();

        let index_buffer    = drv.create_device_buffer(DeviceBufferDesc::Index(Usage::Dynamic(MAX_ELEM_COUNT * std::mem::size_of::<u16>()))).unwrap();
        let vertex_buffer   = drv.create_device_buffer(DeviceBufferDesc::Vertex(Usage::Dynamic(MAX_ELEM_COUNT * std::mem::size_of::<Vertex>()))).unwrap();
        Painter {
            driver: drv.clone(),
            pipeline,
            canvas_width,
            canvas_height,
            index_buffer,
            vertex_buffer,
            egui_textures: Default::default(),
            user_textures: Default::default(),
        }
    }

    pub fn set_canvas_size(&mut self, width: u32, height: u32) {
        self.canvas_width   = width;
        self.canvas_height  = height;
    }

    pub fn new_user_texture(
        &mut self,
        size: (usize, usize),
        srgba_pixels: &[Color32],
        filtering: bool,
    ) -> ::egui::TextureId {
        assert_eq!(size.0 * size.1, srgba_pixels.len());

        let mut pixels: Vec<u8> = Vec::with_capacity(srgba_pixels.len() * 4);
        for srgba in srgba_pixels {
            pixels.push(srgba.r());
            pixels.push(srgba.g());
            pixels.push(srgba.b());
            pixels.push(srgba.a());
        }

        let id = self.user_textures.len() as u64;
        self.user_textures.insert(id, UserTexture {
            size,
            pixels,
            texture: None,
            filtering,
            dirty: true,
        });
        egui::TextureId::User(id)
    }

    fn upload_egui_texture(&mut self, tex_id: u64, texture: &egui::ImageData) {

        let pixels: Vec<Color32> = match &texture {
            egui::ImageData::Color(image) => {
                assert_eq!(
                    image.width() * image.height(),
                    image.pixels.len(),
                    "Mismatch between texture size and texel count"
                );
                image.pixels.clone()
            }
            egui::ImageData::Alpha(image) => {
                let gamma = 1.0;
                image
                    .srgba_pixels(gamma)
                    //.map(|color| color.to_tuple())
                    .collect()
            }
        };

        println!("uploading egui texture: {}x{}", texture.width(), texture.height());

        let tex_desc    = TextureDesc {
            sampler_desc    : SamplerDesc::default(texture.width(), texture.height())
                .with_pixel_format(PixelFormat::RGBA8(MinMagFilter::default()
                    .with_mag_filter(Filter::Linear)
                    .with_min_filter(Filter::Linear)))
                .with_wrap_mode(WrapMode::ClampToEdge),
            payload         : Some(Box::new(pixels))
        };

        let tex = self.driver.create_texture(tex_desc).unwrap();
        self.egui_textures.insert(tex_id, tex);
    }

    fn upload_user_textures(&mut self) {
        for (_, user_texture) in &mut self.user_textures {
            if !user_texture.texture.is_none() && !user_texture.dirty {
                continue;
            }

            let pixels = std::mem::take(&mut user_texture.pixels);

            let tex_desc    = TextureDesc {
                sampler_desc    : SamplerDesc::default(user_texture.size.0, user_texture.size.1).with_pixel_format(PixelFormat::RGBA8(MinMagFilter::default().with_mag_filter(Filter::Linear).with_min_filter(Filter::Linear))),
                payload         : Some(Box::new(pixels))
            };

            if user_texture.texture.is_none() {
                println!("uploading user texture");

                let tex = self.driver.create_texture(tex_desc).unwrap();
                user_texture.texture = Some(tex);
            } else {
                self.driver.update_texture(&mut user_texture.texture.as_mut().unwrap(), tex_desc.payload.unwrap())
            }
            user_texture.dirty = false;
        }
    }

    fn get_texture(&self, texture_id: ::egui::TextureId) -> TexturePtr {
        match texture_id {
            ::egui::TextureId::Managed(id) => {
                self.egui_textures[&id].clone()
            },

            ::egui::TextureId::User(id) => {
                let texture = self.user_textures[&id].texture.clone();
                texture.expect("Should have been uploaded")
            }
        }
    }

    pub fn update_user_texture_data(&mut self, texture_id: ::egui::TextureId, pixels: &[Color32]) {
        match texture_id {
            ::egui::TextureId::Managed(_) => {}
            ::egui::TextureId::User(id) => {
                let mut tex_pixels = Vec::with_capacity(pixels.len() * 4);
                for p in pixels {
                    tex_pixels.push(p.r());
                    tex_pixels.push(p.g());
                    tex_pixels.push(p.b());
                    tex_pixels.push(p.a());
                }

                let mut user_tex = self.user_textures[&id].clone();
                user_tex.pixels = tex_pixels;
                user_tex.dirty  = true;
                self.user_textures.insert(id, user_tex);

            }
        }
    }

    pub fn paint_jobs(
        &mut self,
        frame_buffer: Option<FrameBufferPtr>,
        bg_color: Option<Color32>,
        meshes: Vec<ClippedMesh>,
        egui_texture: &egui::TexturesDelta,
        pixels_per_point: f32,
    ) {
        for (id, delta) in &egui_texture.set {
            match id {
                egui::TextureId::Managed(id) => {
                    self.upload_egui_texture(*id, &delta.image);
                },
                _ => ()
            }
        }

        self.upload_user_textures();

        let screen_size_pixels = vec2(self.canvas_width as f32, self.canvas_height as f32);
        let screen_size_points = screen_size_pixels / pixels_per_point;


        for ClippedMesh(clip_rect, mesh) in meshes {

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

            //scissor Y coordinate is from the bottom
            self.driver.set_scissor (
                clip_min_x as u32,
                self.canvas_height as u32 - clip_max_y as u32,
                (clip_max_x - clip_min_x) as u32,
                (clip_max_y - clip_min_y) as u32,
            );

            if mesh.vertices.len() > 0 {
                self.paint_mesh(&mesh, Vec2f::new(screen_size_points.x, screen_size_points.y));
            }
        }

        for id in &egui_texture.free {
            match id {
                egui::TextureId::Managed(id) => {
                    self.egui_textures.remove(id);
                },
                _ => ()
            }
        }
    }

    fn paint_mesh(&mut self, mesh: &Mesh, screen_size: Vec2f) {
        debug_assert!(mesh.is_valid());
        let meshes = mesh.clone().split_to_u16();
        for mesh in meshes {
            let indices: Vec<u16> = mesh.indices.iter().map(|idx| *idx as u16).collect();

            let mut vertices: Vec<Vertex> = Vec::with_capacity(mesh.vertices.len());
            for v in &mesh.vertices {
                let vert = Vertex {
                    a_pos   : Vec2f::new(v.pos.x, v.pos.y),
                    s_rgba  : color4b(v.color[0], v.color[1], v.color[2], v.color[3]),
                    a_tc    : Vec2f::new(v.uv.x, v.uv.y),
                };

                vertices.push(vert);
            }

            self.driver.update_device_buffer(&mut self.vertex_buffer, 0, &vertices);
            self.driver.update_device_buffer(&mut self.index_buffer, 0, &indices);

            let bindings = Bindings {
                vertex_buffers  : vec!{ self.vertex_buffer.clone() },
                index_buffer    : Some(self.index_buffer.clone()),

                vertex_images   : Vec::new(),
                pixel_images    : Vec::from([self.get_texture(mesh.texture_id)]),
            };

            let u = Uniforms { u_screen_size: screen_size };
            self.driver.draw(&self.pipeline, &bindings, &u as *const Uniforms as *const c_void, (indices.len() / 3) as u32, 1);
        }
    }
}
