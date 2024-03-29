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
use crate::ui::RendererBackEnd;
use glfw::Context;
use std::sync::*;

render_data! {
    vertex Vertex {
        a_pos   : Vec2f,
        a_tc    : Vec2f,
        s_rgba  : Color4b,

    }

    uniforms Uniforms {
        u_transform   : Mat4f,
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
    uniform highp mat4 u_transform;
    in highp vec2 a_pos;
    in highp vec4 s_rgba;
    in highp vec2 a_tc;
    out highp vec4 v_rgba;
    out vec2 v_tc;

    void main() {
        gl_Position = u_transform * vec4(a_pos, 0.0, 1.0);
        v_rgba = s_rgba;
        v_tc = a_tc;
    }
"#;

const FS_SRC: &str = r#"
    #version 300 es
    uniform lowp sampler2D u_sampler;
    in highp vec4 v_rgba;
    in highp vec2 v_tc;
    layout(location = 0) out lowp vec4 f_color;

    void main() {
        highp vec4 tcol = texture(u_sampler, v_tc).rrrr;
        f_color = vec4(v_rgba.rgb, tcol.a * v_rgba.a);
    }
"#;

const MAX_VERTEX_COUNT: usize = 65536;
const MAX_INDEX_COUNT: usize = 65536;

pub struct Renderer {
    pipeline: PipelinePtr,
    vertex_buffer: DeviceBufferPtr,
    index_buffer: DeviceBufferPtr,

    canvas_width: u32,
    canvas_height: u32,
    ui_texture: TexturePtr,

    vertices: Vec<Vertex>,
    indices: Vec<u16>,

    queue: Option<PassCommandQueue>,

    clip: Recti,

    draw_call_count: usize,
}

pub struct Input<P: Default, R: super::RendererBackEnd<P>> {
    _unused: PhantomData<(P, R)>,
}

impl<P: Default, R: super::RendererBackEnd<P>> Input<P, R> {
    pub fn new() -> Self {
        Self {
            _unused: PhantomData::default(),
        }
    }
}

impl Renderer {
    pub fn new(drv: &mut DriverPtr, canvas_width: u32, canvas_height: u32) -> Self {
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

        let blend = Blend {
            src_factor_rgb: BlendFactor::SrcAlpha,
            src_factor_alpha: BlendFactor::One,

            dst_factor_rgb: BlendFactor::OneMinusSrcAlpha,
            dst_factor_alpha: BlendFactor::Zero,
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
            blend: BlendOp::Add(blend),
            polygon_offset: PolygonOffset::None,
        };

        let pipeline = drv.create_pipeline(pipeline_desc).unwrap();

        let vertex_buffer = drv
            .create_device_buffer(DeviceBufferDesc::Vertex(Usage::Dynamic(MAX_VERTEX_COUNT * std::mem::size_of::<Vertex>())))
            .unwrap();

        let index_buffer = drv
            .create_device_buffer(DeviceBufferDesc::Vertex(Usage::Dynamic(MAX_INDEX_COUNT * std::mem::size_of::<u16>())))
            .unwrap();

        let tex_desc = TextureDesc {
            sampler_desc: SamplerDesc::default(ATLAS.width, ATLAS.height)
                .with_pixel_format(PixelFormat::R8(
                    MinMagFilter::default().with_mag_filter(Filter::Nearest).with_min_filter(Filter::Nearest),
                ))
                .with_wrap_mode(WrapMode::ClampToEdge),
            payload: Some(Arc::new(ATLAS.pixels.to_vec())),
        };

        let ui_texture = drv.create_texture(tex_desc).unwrap();
        Self {
            pipeline,
            canvas_width,
            canvas_height,
            vertex_buffer,
            index_buffer,
            ui_texture,
            vertices: Vec::new(),
            indices: Vec::new(),
            queue: None,
            clip: Recti {
                x: 0,
                y: 0,
                width: canvas_width as _,
                height: canvas_height as _,
            },
            draw_call_count: 0,
        }
    }

    fn push_quad_vertices(&mut self, v0: &Vertex, v1: &Vertex, v2: &Vertex, v3: &Vertex) {
        if self.vertices.len() + 4 >= MAX_VERTEX_COUNT || self.indices.len() + 6 >= MAX_INDEX_COUNT {
            //(self as &mut super::Renderer<_>).flush();
            self.flush();
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

    pub fn clip_rect(dst_r: Recti, src_r: Recti, clip_r: Recti) -> Option<(Recti, Recti)> {
        match dst_r.intersect(&clip_r) {
            Some(rect) if rect.width == dst_r.width && rect.height == dst_r.height => Some((dst_r, src_r)),
            Some(rect) if rect.width != 0 && rect.height != 0 => {
                let dx = dst_r.x as f32;
                let dy = dst_r.y as f32;
                let dw = dst_r.width as f32;
                let dh = dst_r.height as f32;

                let rx = rect.x as f32;
                let ry = rect.y as f32;
                let rw = rect.width as f32;
                let rh = rect.height as f32;

                let tx = (rx - dx) / dw;
                let ty = (ry - dy) / dh;
                let tw = (rx + rw - dx) / dw;
                let th = (ry + rh - dy) / dh;

                let sx = src_r.x as f32;
                let sy = src_r.y as f32;
                let sw = src_r.width as f32;
                let sh = src_r.height as f32;

                let st_x = sx + tx * sw;
                let st_y = sy + ty * sh;
                let st_w = sx + tw * sw - st_x;
                let st_h = sy + th * sh - st_y;

                Some((rect, Recti::new(st_x as _, st_y as _, st_w as _, st_h as _)))
            }
            _ => None,
        }
    }

    pub fn push_rect(&mut self, dst: Recti, src: Recti, color: Color4b) {
        let (dst, src) = match Self::clip_rect(dst, src, self.clip) {
            None => return,
            Some((d, s)) => (d, s),
        };

        let x = src.x as f32 / ATLAS.width as f32;
        let y = src.y as f32 / ATLAS.height as f32;
        let w = src.width as f32 / ATLAS.width as f32;
        let h = src.height as f32 / ATLAS.height as f32;

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

        self.push_quad_vertices(&v0, &v1, &v2, &v3);
    }

    pub fn get_draw_call_count(&self) -> usize {
        self.draw_call_count
    }
}

impl super::RendererBackEnd<PassCommandQueue> for Renderer {
    fn frame_size(&self) -> (usize, usize) {
        (self.canvas_width as _, self.canvas_height as _)
    }

    fn begin_frame(&mut self, width: usize, height: usize) {
        assert_eq!(self.queue.is_none(), true);
        self.canvas_width = width as _;
        self.canvas_height = height as _;

        let mut queue = PassCommandQueue::new();
        queue.set_viewport(0, 0, self.canvas_width, self.canvas_height);

        self.queue = Some(queue);
        self.draw_call_count = 0;
        self.clip = Rect::new(0, 0, width as _, height as _);
    }

    fn end_frame(&mut self) -> PassCommandQueue {
        self.flush();
        let mut queue = None;
        core::mem::swap(&mut self.queue, &mut queue);
        queue.unwrap()
    }

    fn draw_rect(&mut self, rect: Recti, color: Color4b) {
        self.push_rect(rect, ATLAS.icons[WHITE].1.rect, color);
    }

    fn draw_text(&mut self, font: FontId, text: &str, pos: Vec2i, color: Color4b) {
        let font_size = ATLAS.fonts[font.0].1.line_size as i32;
        let mut dst = Rect::new(pos.x, pos.y, 0, 0);
        for p in text.chars() {
            if (p as usize) < 127 {
                let chr = usize::min(p as usize, 127);
                let entry = &ATLAS.fonts[font.0].1.entries[chr - 32];
                let src = entry.rect;
                dst.width = src.width;
                dst.height = src.height;
                let mut d = dst;
                d.x += entry.offset.x;
                d.y += font_size - entry.rect.height - entry.offset.y;

                self.push_rect(d, src, color);
                dst.x += entry.advance.x;
            }
        }
    }

    fn draw_icon(&mut self, id: usize, r: Recti, color: Color4b) {
        let src = ATLAS.icons[id as usize].1.rect;
        let x = r.x + (r.width - src.width) / 2;
        let y = r.y + (r.height - src.height) / 2;
        self.push_rect(Rect::new(x, y, src.width, src.height), src, color);
    }

    fn set_clip_rect(&mut self, rect: Recti) {
        self.clip = rect;
    }

    fn add_render_pass_commands(&mut self, commands: PassCommandQueue) {
        self.flush();
        self.queue.as_mut().unwrap().append(commands);
    }

    fn get_char_width(&self, font: FontId, c: char) -> usize {
        ATLAS.fonts[font.0].1.entries[c as usize - 32].rect.width as usize
    }

    fn get_font_height(&self, font: FontId) -> usize {
        ATLAS.fonts[font.0].1.font_size
    }

    fn flush(&mut self) {
        if self.vertices.len() != 0 && self.indices.len() != 0 {
            self.queue
                .as_mut()
                .unwrap()
                .update_device_buffer(&mut self.vertex_buffer, 0, Arc::new(self.vertices.clone()));
            self.queue
                .as_mut()
                .unwrap()
                .update_device_buffer(&mut self.index_buffer, 0, Arc::new(self.indices.clone()));

            let bindings = Bindings {
                vertex_buffers: vec![self.vertex_buffer.clone()],
                index_buffer: Some(self.index_buffer.clone()),

                vertex_images: Vec::new(),
                pixel_images: Vec::from([self.ui_texture.clone()]),
            };

            let u = Uniforms {
                u_transform: transforms::ortho4(0.0, self.canvas_width as f32, self.canvas_height as f32, 0.0, -1.0, 0.0),
            };
            self.queue
                .as_mut()
                .unwrap()
                .draw(&self.pipeline, &bindings, Arc::new(GenPayload::from(u)), (self.indices.len() / 3) as u32, 1);
            self.draw_call_count += 1;
        }
        self.vertices.clear();
        self.indices.clear();
    }

    fn set_atlas(_atlas: &Atlas) {
        todo!()
    }
}

impl<P: Sized + Default, R: super::RendererBackEnd<P>> Input<P, R> {
    pub fn handle_event(&mut self, event: glfw::WindowEvent, window: &mut glfw::Window, ctx: &mut ui::Context<P, R>) {
        match event {
            glfw::WindowEvent::CursorPos(x, y) => ctx.input_mousemove(x as i32, y as i32),
            glfw::WindowEvent::Char(ch) => ctx.input_text(String::from(ch).as_str()),
            glfw::WindowEvent::MouseButton(mb, ac, _) => {
                let (x, y) = window.get_cursor_pos();
                let b = match mb {
                    glfw::MouseButtonLeft => ui::MouseButton::LEFT,
                    glfw::MouseButtonRight => ui::MouseButton::RIGHT,
                    _ => ui::MouseButton::NONE,
                };

                match ac {
                    glfw::Action::Press => ctx.input_mousedown(x as i32, y as i32, b),
                    glfw::Action::Release => ctx.input_mouseup(x as i32, y as i32, b),
                    _ => (),
                }
            }
            glfw::WindowEvent::Scroll(x, y) => ctx.input_scroll(x as i32, y as i32),
            glfw::WindowEvent::Key(key, _, action, modifiers) => {
                let mut keymod = KeyModifier::NONE;
                if key == glfw::Key::Enter {
                    keymod |= KeyModifier::RETURN
                } else if key == glfw::Key::LeftShift || key == glfw::Key::RightShift {
                    keymod |= KeyModifier::SHIFT
                }

                if modifiers == glfw::Modifiers::Alt {
                    keymod |= KeyModifier::ALT
                } else if modifiers == glfw::Modifiers::Control {
                    keymod |= KeyModifier::CTRL
                } else if key == glfw::Key::Backspace {
                    keymod |= KeyModifier::BACKSPACE
                }

                match action {
                    glfw::Action::Press => ctx.input_keydown(keymod),
                    glfw::Action::Release => ctx.input_keyup(keymod),
                    _ => (),
                }
            }
            _ => (),
        }
    }
}

pub struct App {
    glfw: glfw::Glfw,
    window: glfw::Window,
    driver: DriverPtr,
    context: super::Context<PassCommandQueue, Renderer>,
    input: Input<PassCommandQueue, Renderer>,
    events: mpsc::Receiver<(f64, glfw::WindowEvent)>,
}

impl App {
    pub fn new(window_title: &str) -> Self {
        // initialize GLFW3 with OpenGL ES 3.0
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextCreationApi(glfw::ContextCreationApi::Egl));
        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::OpenGlEs));
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 0));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        glfw.window_hint(glfw::WindowHint::DoubleBuffer(true));
        glfw.window_hint(glfw::WindowHint::Resizable(true));
        //glfw.window_hint(glfw::WindowHint::Floating(true));

        let (mut window, events) = glfw
            .create_window(1024, 900, window_title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");

        window.set_all_polling(true);
        window.make_current();
        glfw.set_swap_interval(glfw::SwapInterval::Sync(0));

        let mut driver = renderer::get_driver();

        let (width, height) = window.get_framebuffer_size();
        let renderer = system::Renderer::new(&mut driver, width as u32, height as u32);
        let input = Input::new();
        let context = ui::Context::new(renderer);

        Self {
            glfw,
            window,
            events,
            context,
            input,
            driver,
        }
    }

    pub fn run<Res, F: FnMut(&mut DriverPtr, &mut super::Context<PassCommandQueue, Renderer>, Res) -> Res>(mut self, initial: Res, mut process_frame: F) {
        let mut res = initial;
        'running: while !self.window.should_close() {
            let (width, height) = self.window.get_framebuffer_size();

            let mut driver = self.driver.clone();
            let (queue, res2) = self.context.frame(width as _, height as _, |ctx| process_frame(&mut driver, ctx, res));
            res = res2;

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
            pass.queue.append(queue);
            self.driver.render_pass(&mut pass);
            self.window.swap_buffers();

            self.glfw.wait_events_timeout(0.007);
            for (_, event) in glfw::flush_messages(&self.events) {
                match event {
                    glfw::WindowEvent::Close | glfw::WindowEvent::Key(glfw::Key::Escape, ..) => break 'running,

                    _ => self.input.handle_event(event, &mut self.window, &mut self.context),
                }
            }
        }

        self.window.close();
    }
}
