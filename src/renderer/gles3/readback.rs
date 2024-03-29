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
use super::super::gl::types::*;
use super::super::*;
use crate::renderer::gles3::*;
use crate::rs_math3d::*;

use super::renderer::*;
use std::ffi::c_void;
use std::sync::*;

////////////////////////////////////////////////////////////////////////////////
// Readback surface
//
// ES 3.0 & WebGL 2 do not provide gl::TEXTURE buffer objects, we have no choice
// but to simulate reading back using this mechanism :P
////////////////////////////////////////////////////////////////////////////////

unsafe fn alloc_pixel_array<T>(size: usize) -> *mut T {
    let v = Vec::with_capacity(size);
    core::mem::ManuallyDrop::new(v).as_mut_ptr()
}

crate::render_data! {
    vertex QuadVertex {
        position    : Vec2f,
        uv          : Vec2f,
    }
}

pub(crate) struct ReadbackDriver {
    u_fb: Option<FrameBufferPtr>,
    f_fb: Option<FrameBufferPtr>,
    u_pipeline: PipelinePtr, // unsigned intX pipeline
    f_pipeline: PipelinePtr, // floating point pipeline
    vb: DeviceBufferPtr,
    ib: DeviceBufferPtr,

    gles_driver: DriverPtrInternal,
}

static COPY_VERTEX_SHADER: &'static str = "
#version 300 es
precision highp float;
in          vec2        position;
in          vec2        uv;

out highp   vec2        vUV;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    vUV = uv;
}";

static COPY_UINT_PIXEL_SHADER: &'static str = "
#version 300 es
precision highp float;
precision highp usampler2D;

in highp    vec2        vUV;

uniform     usampler2D  uTexture;

out         uvec4       fragColor;

void main() {
    fragColor = texture(uTexture, vUV);
}";

static COPY_FLOAT_PIXEL_SHADER: &'static str = "
#version 300 es
precision highp float;
precision highp usampler2D;

in highp    vec2        vUV;

uniform     sampler2D   uTexture;

out         vec4        fragColor;

void main() {
    fragColor = texture(uTexture, vUV);
}";

impl ReadbackDriver {
    pub fn new(driver: &mut DriverPtrInternal) -> Self {
        let orig = driver.clone();
        let mut drv_lock = driver.lock();
        let driver = drv_lock.as_deref_mut().unwrap();

        let quad_verts = vec![
            QuadVertex {
                position: Vec2f::new(-1.0, -1.0),
                uv: Vec2f::new(0.0, 0.0),
            },
            QuadVertex {
                position: Vec2f::new(1.0, -1.0),
                uv: Vec2f::new(1.0, 0.0),
            },
            QuadVertex {
                position: Vec2f::new(1.0, 1.0),
                uv: Vec2f::new(1.0, 1.0),
            },
            QuadVertex {
                position: Vec2f::new(-1.0, 1.0),
                uv: Vec2f::new(0.0, 1.0),
            },
        ];

        let quad_index: Vec<u32> = vec![0, 1, 2, 2, 3, 0];

        let vb_desc = DeviceBufferDesc::Vertex(Usage::Static(Arc::new(quad_verts)));
        let vb = driver.create_device_buffer(vb_desc).unwrap();

        let ib_desc = DeviceBufferDesc::Index(Usage::Static(Arc::new(quad_index)));
        let ib = driver.create_device_buffer(ib_desc).unwrap();

        Self {
            vb: vb,
            ib: ib,
            u_fb: None, // Self::create_fb(driver, OrigSurfaceType::UInt),
            f_fb: None, // Self::create_fb(driver, OrigSurfaceType::Float),
            u_pipeline: Self::create_copy_pipeline(driver, OrigSurfaceType::UInt),
            f_pipeline: Self::create_copy_pipeline(driver, OrigSurfaceType::Float),

            gles_driver: orig,
        }
    }

    fn create_copy_shader(
        driver: &mut dyn Driver,
        orig_surface_type: OrigSurfaceType,
    ) -> ShaderPtr {
        let shader_desc = ShaderDesc {
            vertex_shader: String::from(COPY_VERTEX_SHADER),
            pixel_shader: String::from(match orig_surface_type {
                OrigSurfaceType::UInt => COPY_UINT_PIXEL_SHADER,
                OrigSurfaceType::Float => COPY_FLOAT_PIXEL_SHADER,
            }),

            vertex_attributes: vec![QuadVertex::get_attribute_names()],
            vertex_uniforms: Vec::new(),
            vertex_surfaces: Vec::new(),

            pixel_uniforms: Vec::new(),
            pixel_surfaces: Vec::from([String::from("uTexture")]),
        };

        driver.create_shader(shader_desc).unwrap()
    }

    fn create_copy_pipeline(
        driver: &mut dyn Driver,
        orig_surface_type: OrigSurfaceType,
    ) -> PipelinePtr {
        let vertex_layout = VertexBufferLayout {
            buffer_id: 0,
            vertex_attributes: QuadVertex::get_attribute_descriptors(),
            stride: QuadVertex::stride(),
            divisor: 0,
        };

        let model_pipeline_desc = PipelineDesc {
            primitive_type: PrimitiveType::Triangles,
            shader: Self::create_copy_shader(driver, orig_surface_type),
            buffer_layouts: vec![vertex_layout],
            uniform_descs: vec![],
            index_type: IndexType::UInt32,
            face_winding: FaceWinding::CCW,
            cull_mode: CullMode::Winding,
            depth_write: true,
            depth_test: true,
            blend: BlendOp::None,
            polygon_offset: PolygonOffset::None,
        };

        driver.create_pipeline(model_pipeline_desc).unwrap()
    }

    fn create_fb(driver: &mut dyn Driver, orig_surface_type: OrigSurfaceType) -> FrameBufferPtr {
        let caps = driver.get_caps();
        let width = caps.max_2d_surface_dimension.width as usize;
        let height = caps.max_2d_surface_dimension.height as usize;

        println!("create readback buffers: 4 * 4 x {} x {}", width, height);
        println!(
            "memory: {} MB / buffer",
            (width * height * 4 * 4) / 1024 / 1024
        );

        let format = match orig_surface_type {
            OrigSurfaceType::UInt => PixelFormat::RGBA32U,
            OrigSurfaceType::Float => PixelFormat::RGBA32F,
        };

        let color_tex_desc = SamplerDesc::default(width, height).with_pixel_format(format);
        let color_buffer_desc = TextureDesc {
            sampler_desc: color_tex_desc,
            payload: None,
        };
        let color_buffer = driver.create_texture(color_buffer_desc).unwrap();

        let depth_tex_desc =
            SamplerDesc::default(width, height).with_pixel_format(PixelFormat::D32);
        let depth_buffer_desc = RenderTargetDesc {
            sampler_desc: depth_tex_desc,
            sample_count: 0,
        };
        let depth_buffer = driver.create_render_target(depth_buffer_desc).unwrap();

        let fb_desc = FrameBufferDesc {
            color_attachements: [
                Some(SurfaceAttachment::Texture(color_buffer)),
                None,
                None,
                None,
            ],
            depth_stencil_attachement: SurfaceAttachment::RenderTarget(depth_buffer),
        };

        driver.create_frame_buffer(fb_desc).unwrap()
    }

    fn texture_type(surface: &TexturePtr) -> OrigSurfaceType {
        surface
            .desc()
            .sampler_desc
            .pixel_format
            .to_orig_surface_type()
    }

    fn pixel_format(surface: &TexturePtr) -> &PixelFormat {
        let desc = surface.desc();
        &desc.sampler_desc.pixel_format
    }

    fn gl_format(pf: &PixelFormat) -> GLenum {
        match pf {
            PixelFormat::RGB8U => gl::RGB_INTEGER,
            PixelFormat::RGBA8U => gl::RGBA_INTEGER,
            PixelFormat::R8U => gl::RED_INTEGER,
            PixelFormat::RGB32U => gl::RGB_INTEGER,
            PixelFormat::RGBA32U => gl::RGBA_INTEGER,
            PixelFormat::R32U => gl::RED_INTEGER,

            PixelFormat::RGB32F => gl::RGB,
            PixelFormat::RGBA32F => gl::RGBA,
            PixelFormat::R32F => gl::RED,

            PixelFormat::D16 => gl::RED,
            PixelFormat::D32 => gl::RED,
            PixelFormat::D24S8 => gl::RED,
            PixelFormat::D32S8 => gl::RED,

            PixelFormat::RGB8(_) => gl::RGB,
            PixelFormat::RGBA8(_) => gl::RGBA,
            PixelFormat::R8(_) => gl::RED,
        }
    }

    fn gl_elem_type(pf: &PixelFormat) -> GLenum {
        match &pf {
            PixelFormat::RGB8U => gl::UNSIGNED_INT,
            PixelFormat::RGBA8U => gl::UNSIGNED_INT,
            PixelFormat::R8U => gl::UNSIGNED_INT,
            PixelFormat::RGB32U => gl::UNSIGNED_INT,
            PixelFormat::RGBA32U => gl::UNSIGNED_INT,
            PixelFormat::R32U => gl::UNSIGNED_INT,

            PixelFormat::RGB32F => gl::FLOAT,
            PixelFormat::RGBA32F => gl::FLOAT,
            PixelFormat::R32F => gl::FLOAT,

            PixelFormat::D16 => gl::FLOAT,
            PixelFormat::D32 => gl::FLOAT,
            PixelFormat::D24S8 => gl::FLOAT,
            PixelFormat::D32S8 => gl::FLOAT,

            PixelFormat::RGB8(_) => gl::FLOAT,
            PixelFormat::RGBA8(_) => gl::FLOAT,
            PixelFormat::R8(_) => gl::FLOAT,
        }
    }

    unsafe fn alloc_pixels(surface: &TexturePtr, width: usize, height: usize) -> *mut u8 {
        let desc = surface.desc();
        match desc.sampler_desc.pixel_format {
            PixelFormat::RGB8U => alloc_pixel_array::<Vector3<u8>>(width * height) as *mut u8,
            PixelFormat::RGBA8U => alloc_pixel_array::<Vector4<u8>>(width * height) as *mut u8,
            PixelFormat::R8U => alloc_pixel_array::<u8>(width * height) as *mut u8,
            PixelFormat::RGB32U => alloc_pixel_array::<Vector3<u32>>(width * height) as *mut u8,
            PixelFormat::RGBA32U => alloc_pixel_array::<Vector4<u32>>(width * height) as *mut u8,
            PixelFormat::R32U => alloc_pixel_array::<u32>(width * height) as *mut u8,
            PixelFormat::RGB32F => alloc_pixel_array::<Vec3f>(width * height) as *mut u8,
            PixelFormat::RGBA32F => alloc_pixel_array::<Vec4f>(width * height) as *mut u8,
            PixelFormat::R32F => alloc_pixel_array::<f32>(width * height) as *mut u8,
            PixelFormat::RGB8(_) => alloc_pixel_array::<Vec3f>(width * height) as *mut u8,
            PixelFormat::RGBA8(_) => alloc_pixel_array::<Vec4f>(width * height) as *mut u8,
            PixelFormat::R8(_) => alloc_pixel_array::<f32>(width * height) as *mut u8,

            PixelFormat::D16 => alloc_pixel_array::<f32>(width * height) as *mut u8,
            PixelFormat::D32 => alloc_pixel_array::<f32>(width * height) as *mut u8,
            PixelFormat::D24S8 => alloc_pixel_array::<f32>(width * height) as *mut u8,
            PixelFormat::D32S8 => alloc_pixel_array::<f32>(width * height) as *mut u8,
        }
    }

    unsafe fn data_to_readback(
        data: *mut u8,
        width: usize,
        height: usize,
        pf: &PixelFormat,
    ) -> ReadbackPayload {
        match pf {
            PixelFormat::RGB8U => ReadbackPayload::RGB32U(Vec::from_raw_parts(
                data as *mut _,
                width * height,
                width * height,
            )),
            PixelFormat::RGBA8U => ReadbackPayload::RGBA32U(Vec::from_raw_parts(
                data as *mut _,
                width * height,
                width * height,
            )),
            PixelFormat::R8U => ReadbackPayload::R32U(Vec::from_raw_parts(
                data as *mut _,
                width * height,
                width * height,
            )),
            PixelFormat::RGB32U => ReadbackPayload::RGB32U(Vec::from_raw_parts(
                data as *mut _,
                width * height,
                width * height,
            )),
            PixelFormat::RGBA32U => ReadbackPayload::RGBA32U(Vec::from_raw_parts(
                data as *mut _,
                width * height,
                width * height,
            )),
            PixelFormat::R32U => ReadbackPayload::R32U(Vec::from_raw_parts(
                data as *mut _,
                width * height,
                width * height,
            )),
            PixelFormat::RGB32F => ReadbackPayload::RGB32F(Vec::from_raw_parts(
                data as *mut _,
                width * height,
                width * height,
            )),
            PixelFormat::RGBA32F => ReadbackPayload::RGBA32F(Vec::from_raw_parts(
                data as *mut _,
                width * height,
                width * height,
            )),
            PixelFormat::R32F => ReadbackPayload::R32F(Vec::from_raw_parts(
                data as *mut _,
                width * height,
                width * height,
            )),
            PixelFormat::RGB8(_) => ReadbackPayload::RGB32F(Vec::from_raw_parts(
                data as *mut _,
                width * height,
                width * height,
            )),
            PixelFormat::RGBA8(_) => ReadbackPayload::RGBA32F(Vec::from_raw_parts(
                data as *mut _,
                width * height,
                width * height,
            )),
            PixelFormat::R8(_) => ReadbackPayload::R32F(Vec::from_raw_parts(
                data as *mut _,
                width * height,
                width * height,
            )),

            PixelFormat::D16 => ReadbackPayload::Depth(Vec::from_raw_parts(
                data as *mut _,
                width * height,
                width * height,
            )),
            PixelFormat::D32 => ReadbackPayload::Depth(Vec::from_raw_parts(
                data as *mut _,
                width * height,
                width * height,
            )),
            PixelFormat::D24S8 => ReadbackPayload::Depth(Vec::from_raw_parts(
                data as *mut _,
                width * height,
                width * height,
            )),
            PixelFormat::D32S8 => ReadbackPayload::Depth(Vec::from_raw_parts(
                data as *mut _,
                width * height,
                width * height,
            )),
        }
    }

    pub fn read_surface(
        &mut self,
        surface: &TexturePtr,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<ReadbackPayload> {
        unsafe {
            let mut l = self.gles_driver.lock();
            let me2 = l.as_deref_mut().unwrap();
            let driver = &mut *(me2 as *mut dyn Driver as *mut Gles3Driver);

            match self.f_fb {
                Some(_) => (),
                None => self.f_fb = Some(Self::create_fb(driver, OrigSurfaceType::Float)),
            }

            match self.u_fb {
                Some(_) => (),
                None => self.u_fb = Some(Self::create_fb(driver, OrigSurfaceType::UInt)),
            }

            let (fb, pipeline) = match Self::texture_type(surface) {
                OrigSurfaceType::Float => (&self.f_fb, &self.f_pipeline),
                OrigSurfaceType::UInt => (&self.u_fb, &self.u_pipeline),
            };

            match fb {
                Some(fb) => {
                    let fbb = driver.get_framebuffer_gl_id(fb.res_id());
                    let mut current_fb = 0;
                    gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut current_fb);
                    let mut viewport: [GLint; 4] = [0, 0, 0, 0];
                    let mut scissor: [GLint; 4] = [0, 0, 0, 0];

                    // TODO: scissor test flags and other related states
                    gl::GetIntegerv(gl::VIEWPORT, &mut viewport as *mut [_] as *mut _);
                    gl::GetIntegerv(gl::SCISSOR_BOX, &mut scissor as *mut [_] as *mut _);

                    gl::BindFramebuffer(gl::FRAMEBUFFER, fbb);
                    Gles3Driver::check_gl_error();

                    let vw = surface.desc().sampler_desc.width() as GLsizei;
                    let vh = surface.desc().sampler_desc.height() as GLsizei;
                    gl::Viewport(0, 0, vw, vh);
                    gl::Scissor(0, 0, vw, vh);

                    let flags = gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT;
                    gl::ClearDepthf(1.0);

                    let draw_buffer: [GLenum; 4] =
                        [gl::COLOR_ATTACHMENT0, gl::NONE, gl::NONE, gl::NONE];
                    gl::DrawBuffers(4, &draw_buffer as *const GLenum);

                    let i_cols: [GLuint; 4] = [0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF];
                    gl::ClearBufferuiv(gl::COLOR as GLenum, 0, i_cols.as_ptr() as *const GLuint);
                    gl::Clear(flags);

                    let bindings = Bindings {
                        vertex_buffers: vec![self.vb.clone()],
                        index_buffer: Some(self.ib.clone()),

                        vertex_images: Vec::from([]),
                        pixel_images: Vec::from([surface.clone()]),
                    };
                    driver.draw(
                        pipeline,
                        &bindings,
                        core::ptr::null() as *const c_void,
                        2,
                        1,
                    );

                    // get the data
                    let data = Self::alloc_pixels(surface, (w * 16) as usize, h as usize);
                    assert_ne!(data, std::ptr::null_mut());
                    let pf = Self::pixel_format(surface);
                    gl::ReadBuffer(gl::COLOR_ATTACHMENT0 as GLenum);
                    gl::BindBuffer(gl::PIXEL_PACK_BUFFER, 0);
                    gl::ReadPixels(
                        x as GLint,
                        y as GLint,
                        w as GLsizei,
                        h as GLsizei,
                        Self::gl_format(&pf),
                        Self::gl_elem_type(&pf),
                        data as *mut ::core::ffi::c_void,
                    );
                    Gles3Driver::check_gl_error();
                    gl::BindFramebuffer(gl::FRAMEBUFFER, current_fb as GLuint);
                    Gles3Driver::check_gl_error();
                    gl::Viewport(viewport[0], viewport[1], viewport[2], viewport[3]);
                    gl::Scissor(scissor[0], scissor[1], scissor[2], scissor[3]);

                    Some(Self::data_to_readback(data, w as usize, h as usize, &pf))
                }
                None => None,
            }
        }
    }
}

impl Driver for ReadbackDriver {
    fn get_caps(&self) -> DriverCaps {
        let lock = self.gles_driver.lock();
        let driver = lock.unwrap();
        driver.get_caps()
    }

    fn create_device_buffer(&mut self, desc: DeviceBufferDesc) -> Option<DeviceBufferPtr> {
        let mut lock = self.gles_driver.lock();
        let driver = lock.as_mut().unwrap();
        driver.create_device_buffer(desc)
    }

    fn create_texture(&mut self, desc: TextureDesc) -> Option<TexturePtr> {
        let mut lock = self.gles_driver.lock();
        let driver = lock.as_mut().unwrap();
        driver.create_texture(desc)
    }

    fn create_render_target(&mut self, desc: RenderTargetDesc) -> Option<RenderTargetPtr> {
        let mut lock = self.gles_driver.lock();
        let driver = lock.as_mut().unwrap();
        driver.create_render_target(desc)
    }

    fn create_shader(&mut self, desc: ShaderDesc) -> Option<ShaderPtr> {
        let mut lock = self.gles_driver.lock();
        let driver = lock.as_mut().unwrap();
        driver.create_shader(desc)
    }

    fn create_pipeline(&mut self, desc: PipelineDesc) -> Option<PipelinePtr> {
        let mut lock = self.gles_driver.lock();
        let driver = lock.as_mut().unwrap();
        driver.create_pipeline(desc)
    }

    fn create_frame_buffer(&mut self, desc: FrameBufferDesc) -> Option<FrameBufferPtr> {
        let mut lock = self.gles_driver.lock();
        let driver = lock.as_mut().unwrap();
        driver.create_frame_buffer(desc)
    }

    fn delete_resource(&mut self, resource_type: &ResourceType, res_id: usize) {
        let mut lock = self.gles_driver.lock();
        let driver = lock.as_mut().unwrap();
        driver.delete_resource(resource_type, res_id)
    }

    fn render_pass(&mut self, pass: &mut Pass) {
        let mut lock = self.gles_driver.lock();
        let driver = lock.as_mut().unwrap();
        driver.render_pass(pass)
    }

    fn read_back(
        &mut self,
        surface: &TexturePtr,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<ReadbackPayload> {
        self.read_surface(surface, x, y, w, h)
    }
}

impl Drop for ReadbackDriver {
    fn drop(&mut self) {
        println!("ReadBackDriver dropped - All is good!")
    }
}

pub fn get_driver() -> DriverPtr {
    unsafe {
        let mut range: [GLint; 2] = [0, 0];
        let mut precision = 0;

        gl::GetShaderPrecisionFormat(
            gl::FRAGMENT_SHADER,
            gl::HIGH_FLOAT,
            range.as_mut_ptr(),
            &mut precision,
        );
        println!("highp float range: {:?} - precision: {}", range, precision);

        gl::GetShaderPrecisionFormat(
            gl::FRAGMENT_SHADER,
            gl::HIGH_INT,
            range.as_mut_ptr(),
            &mut precision,
        );
        println!("highp int range: {:?} - precision: {}", range, precision);

        gl::GetShaderPrecisionFormat(
            gl::FRAGMENT_SHADER,
            gl::MEDIUM_FLOAT,
            range.as_mut_ptr(),
            &mut precision,
        );
        println!(
            "mediump float range: {:?} - precision: {}",
            range, precision
        );

        gl::GetShaderPrecisionFormat(
            gl::FRAGMENT_SHADER,
            gl::MEDIUM_INT,
            range.as_mut_ptr(),
            &mut precision,
        );
        println!("mediump int range: {:?} - precision: {}", range, precision);

        gl::GetShaderPrecisionFormat(
            gl::FRAGMENT_SHADER,
            gl::LOW_FLOAT,
            range.as_mut_ptr(),
            &mut precision,
        );
        println!("lowp float range: {:?} - precision: {}", range, precision);

        gl::GetShaderPrecisionFormat(
            gl::FRAGMENT_SHADER,
            gl::LOW_INT,
            range.as_mut_ptr(),
            &mut precision,
        );
        println!("lowp int range: {:?} - prselfecision: {}", range, precision);
    }
    let mut drv = renderer::Gles3Driver::new();
    DriverPtr::from(Arc::new(Mutex::new(ReadbackDriver::new(&mut drv))))
}
