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
use rs_ctypes::*;
use super::super::*;
use super::super::gl::types::*;
use crate::rs_math3d::*;
use super::readback::*;

use std::collections::{VecDeque};
use core::ops::{Index};
use core::sync::atomic::*;
use std::sync::*;

fn color4b_to_color4f(col: Color4b) -> Vec4f {
    let r = col.x as f32 / 255.0;
    let g = col.y as f32 / 255.0;
    let b = col.z as f32 / 255.0;
    let a = col.w as f32 / 255.0;
    Vec4f::new(r, g, b, a)
}

unsafe fn alloc_string<'a>(size: usize) -> &'a mut str {
    let l = std::alloc::Layout::array::<u8>(size as usize).unwrap();
    let sptr = std::alloc::alloc(l);
    let sl = std::slice::from_raw_parts_mut(sptr, size as usize);
    std::str::from_utf8_unchecked_mut(sl)
}

unsafe fn free_string<'a>(s: &mut str, size: usize) {
    let l = std::alloc::Layout::array::<u8>(size as usize).unwrap();
    std::alloc::dealloc(s.as_mut_ptr(), l);
}

pub struct GLProgram {
    prog_id     : GLuint,
}

impl Drop for GLProgram {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.prog_id) };
    }
}

trait GLUniformBlock {
    fn setup(&self);
}

trait GLVertexFormat {
    fn gl_elem_count(&self) -> GLuint;
    fn gl_elem_type(&self) -> GLenum;
    fn gl_is_normalized(&self) -> GLboolean;
}

impl GLVertexFormat for VertexFormat {
    fn gl_elem_count(&self) -> GLuint {
        match self {
            VertexFormat::Byte      => 1,
            VertexFormat::Byte2     => 2,
            VertexFormat::Byte3     => 3,
            VertexFormat::Byte4     => 4,

            VertexFormat::SByte     => 1,
            VertexFormat::SByte2    => 2,
            VertexFormat::SByte3    => 3,
            VertexFormat::SByte4    => 4,

            VertexFormat::Int       => 1,
            VertexFormat::Int2      => 2,
            VertexFormat::Int3      => 3,
            VertexFormat::Int4      => 4,

            VertexFormat::UInt      => 1,
            VertexFormat::UInt2     => 2,
            VertexFormat::UInt3     => 3,
            VertexFormat::UInt4     => 4,

            VertexFormat::Short     => 1,
            VertexFormat::Short2    => 2,
            VertexFormat::Short3    => 3,
            VertexFormat::Short4    => 4,

            VertexFormat::Float     => 1,
            VertexFormat::Float2    => 2,
            VertexFormat::Float3    => 3,
            VertexFormat::Float4    => 4,

            VertexFormat::Float2x2  => 4,
            VertexFormat::Float3x3  => 9,
            VertexFormat::Float4x4  => 16,
        }
    }

    fn gl_elem_type(&self) -> GLenum {
        match self {
            VertexFormat::Byte      => gl::UNSIGNED_BYTE,
            VertexFormat::Byte2     => gl::UNSIGNED_BYTE,
            VertexFormat::Byte3     => gl::UNSIGNED_BYTE,
            VertexFormat::Byte4     => gl::UNSIGNED_BYTE,

            VertexFormat::SByte     => gl::BYTE,
            VertexFormat::SByte2    => gl::BYTE,
            VertexFormat::SByte3    => gl::BYTE,
            VertexFormat::SByte4    => gl::BYTE,

            VertexFormat::Int       => gl::INT,
            VertexFormat::Int2      => gl::INT,
            VertexFormat::Int3      => gl::INT,
            VertexFormat::Int4      => gl::INT,

            VertexFormat::UInt      => gl::UNSIGNED_INT,
            VertexFormat::UInt2     => gl::UNSIGNED_INT,
            VertexFormat::UInt3     => gl::UNSIGNED_INT,
            VertexFormat::UInt4     => gl::UNSIGNED_INT,

            VertexFormat::Short     => gl::SHORT,
            VertexFormat::Short2    => gl::SHORT,
            VertexFormat::Short3    => gl::SHORT,
            VertexFormat::Short4    => gl::SHORT,

            VertexFormat::Float     => gl::FLOAT,
            VertexFormat::Float2    => gl::FLOAT,
            VertexFormat::Float3    => gl::FLOAT,
            VertexFormat::Float4    => gl::FLOAT,

            VertexFormat::Float2x2  => gl::FLOAT,
            VertexFormat::Float3x3  => gl::FLOAT,
            VertexFormat::Float4x4  => gl::FLOAT,
        }
    }

    fn gl_is_normalized(&self) -> GLboolean {
        let r = match self {
            VertexFormat::Byte      => true,
            VertexFormat::Byte2     => true,
            VertexFormat::Byte3     => true,
            VertexFormat::Byte4     => true,

            VertexFormat::SByte     => true,
            VertexFormat::SByte2    => true,
            VertexFormat::SByte3    => true,
            VertexFormat::SByte4    => true,

            VertexFormat::Int       => false,
            VertexFormat::Int2      => false,
            VertexFormat::Int3      => false,
            VertexFormat::Int4      => false,

            VertexFormat::UInt      => false,
            VertexFormat::UInt2     => false,
            VertexFormat::UInt3     => false,
            VertexFormat::UInt4     => false,

            VertexFormat::Short     => false,
            VertexFormat::Short2    => false,
            VertexFormat::Short3    => false,
            VertexFormat::Short4    => false,

            VertexFormat::Float     => false,
            VertexFormat::Float2    => false,
            VertexFormat::Float3    => false,
            VertexFormat::Float4    => false,

            VertexFormat::Float2x2  => false,
            VertexFormat::Float3x3  => false,
            VertexFormat::Float4x4  => false,
        };
        r as GLboolean
    }
}

fn uniform_ptr_to_slice<'a, T>(ptr: *const c_void, offset: usize, count: usize) -> &'a [T] {
    let cptr = ptr as *const u8;
    let _cptr = unsafe { cptr.offset(offset as isize) };
    let tptr = _cptr as *const T;
    unsafe { core::slice::from_raw_parts(tptr, count) }
}

fn setup_uniforms(uniforms: *const c_void, data_desc_layout: &[UniformDataDesc], prg_desc_layout: &[(String, GLuint)]) {
    unsafe {
        for i in 0..data_desc_layout.len() {
            let offset = data_desc_layout[i].offset();
            let location = prg_desc_layout[i].1 as GLint;
            match &data_desc_layout[i].desc().format() {
                UniformDataType::UInt => { let s : &[u32]     = uniform_ptr_to_slice(uniforms, offset, 1);  gl::Uniform1uiv(location, 1, s.as_ptr()); },
                UniformDataType::UInt2=> { let s : &[u32]     = uniform_ptr_to_slice(uniforms, offset, 2);  gl::Uniform2uiv(location, 1, s.as_ptr()); },
                UniformDataType::UInt3=> { let s : &[u32]     = uniform_ptr_to_slice(uniforms, offset, 3);  gl::Uniform3uiv(location, 1, s.as_ptr()); },
                UniformDataType::UInt4=> { let s : &[u32]     = uniform_ptr_to_slice(uniforms, offset, 4);  gl::Uniform4uiv(location, 1, s.as_ptr()); },
                UniformDataType::Int  => { let s : &[i32]     = uniform_ptr_to_slice(uniforms, offset, 1);  gl::Uniform1iv(location, 1, s.as_ptr()); },
                UniformDataType::Int2 => { let s : &[i32]     = uniform_ptr_to_slice(uniforms, offset, 2);  gl::Uniform2iv(location, 1, s.as_ptr()); },
                UniformDataType::Int3 => { let s : &[i32]     = uniform_ptr_to_slice(uniforms, offset, 3);  gl::Uniform3iv(location, 1, s.as_ptr()); },
                UniformDataType::Int4 => { let s : &[i32]     = uniform_ptr_to_slice(uniforms, offset, 4);  gl::Uniform4iv(location, 1, s.as_ptr()); },
                UniformDataType::Float  => { let s : &[f32]   = uniform_ptr_to_slice(uniforms, offset, 1);  gl::Uniform1fv(location, 1, s.as_ptr()); },
                UniformDataType::Float2 => { let s : &[f32]   = uniform_ptr_to_slice(uniforms, offset, 2);  gl::Uniform2fv(location, 1, s.as_ptr()); },
                UniformDataType::Float3 => { let s : &[f32]   = uniform_ptr_to_slice(uniforms, offset, 3);  gl::Uniform3fv(location, 1, s.as_ptr()); },
                UniformDataType::Float4 => { let s : &[f32]   = uniform_ptr_to_slice(uniforms, offset, 4);  gl::Uniform4fv(location, 1, s.as_ptr()); },
                UniformDataType::Float2x2 => { let s : &[f32] = uniform_ptr_to_slice(uniforms, offset, 4);  gl::UniformMatrix2fv(location, 1, false as GLboolean, s.as_ptr()); },
                UniformDataType::Float3x3 => { let s : &[f32] = uniform_ptr_to_slice(uniforms, offset, 9);  gl::UniformMatrix3fv(location, 1, false as GLboolean, s.as_ptr()); },
                UniformDataType::Float4x4 => { let s : &[f32] = uniform_ptr_to_slice(uniforms, offset, 16); gl::UniformMatrix4fv(location, 1, false as GLboolean, s.as_ptr()); },
            }
        }
    }
}

struct GLDeviceBuffer {
    gl_id           : GLuint,
    desc            : DeviceBufferDesc,
}

impl Drop for GLDeviceBuffer {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.gl_id as *const GLuint) };
    }
}

struct GLTexture {
    gl_id   : GLuint,
}

impl Drop for GLTexture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.gl_id as *const GLuint);
        }
    }
}

struct GLRenderTarget {
    gl_id   : GLuint,
}

impl Drop for GLRenderTarget {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteRenderbuffers(1, &self.gl_id as *const GLuint);
        }
    }
}

trait GLPixelFormat {
    fn gl_internal_format(&self) -> GLuint;
    fn gl_format(&self) -> GLuint;
    fn gl_elem_type(&self) -> GLenum;
    fn gl_pixel_size(&self) -> usize;
}

impl GLPixelFormat for PixelFormat {
    fn gl_internal_format(&self) -> GLenum {
        match self {
            PixelFormat::RGB8U  => gl::RGB8UI,
            PixelFormat::RGBA8U => gl::RGBA8UI,
            PixelFormat::R8U    => gl::R8UI,
            PixelFormat::RGB32U => gl::RGB32UI,
            PixelFormat::RGBA32U=> gl::RGBA32UI,
            PixelFormat::R32U   => gl::R32UI,

            PixelFormat::RGB32F => gl::RGB32F,
            PixelFormat::RGBA32F=> gl::RGBA32F,
            PixelFormat::R32F   => gl::R32F,

            PixelFormat::D16    => gl::DEPTH_COMPONENT16,
            PixelFormat::D32    => gl::DEPTH_COMPONENT32F,
            PixelFormat::D24S8  => gl::DEPTH24_STENCIL8,
            PixelFormat::D32S8  => gl::DEPTH32F_STENCIL8,

            PixelFormat::RGB8(_)    => gl::RGB,
            PixelFormat::RGBA8(_)   => gl::RGBA,
            PixelFormat::R8(_)      => gl::RED,
        }
    }

    fn gl_format(&self) -> GLenum {
        match self {
            PixelFormat::RGB8U  => gl::RGB_INTEGER,
            PixelFormat::RGBA8U => gl::RGBA_INTEGER,
            PixelFormat::R8U    => gl::RED_INTEGER,
            PixelFormat::RGB32U => gl::RGB_INTEGER,
            PixelFormat::RGBA32U=> gl::RGBA_INTEGER,
            PixelFormat::R32U   => gl::RED_INTEGER,

            PixelFormat::RGB32F => gl::RGB,
            PixelFormat::RGBA32F=> gl::RGBA,
            PixelFormat::R32F   => gl::RED,

            PixelFormat::D16    => gl::DEPTH_COMPONENT,
            PixelFormat::D32    => gl::DEPTH_COMPONENT,
            PixelFormat::D24S8  => gl::DEPTH_STENCIL,
            PixelFormat::D32S8  => gl::DEPTH_STENCIL,

            PixelFormat::RGB8(_)    => gl::RGB,
            PixelFormat::RGBA8(_)   => gl::RGBA,
            PixelFormat::R8(_)      => gl::RED,
        }
    }

    fn gl_elem_type(&self) -> GLenum {
        match &self {
            PixelFormat::RGB8U  => gl::UNSIGNED_BYTE,
            PixelFormat::RGBA8U => gl::UNSIGNED_BYTE,
            PixelFormat::R8U    => gl::UNSIGNED_BYTE,
            PixelFormat::RGB32U => gl::UNSIGNED_INT,
            PixelFormat::RGBA32U=> gl::UNSIGNED_INT,
            PixelFormat::R32U   => gl::UNSIGNED_INT,

            PixelFormat::RGB32F => gl::FLOAT,
            PixelFormat::RGBA32F=> gl::FLOAT,
            PixelFormat::R32F   => gl::FLOAT,

            PixelFormat::D16    => gl::UNSIGNED_SHORT,
            PixelFormat::D32    => gl::FLOAT,
            PixelFormat::D24S8  => gl::UNSIGNED_INT_24_8,
            PixelFormat::D32S8  => gl::FLOAT_32_UNSIGNED_INT_24_8_REV,

            PixelFormat::RGB8(_)    => gl::UNSIGNED_BYTE,
            PixelFormat::RGBA8(_)   => gl::UNSIGNED_BYTE,
            PixelFormat::R8(_)      => gl::UNSIGNED_BYTE,
        }
    }

    fn gl_pixel_size(&self) -> usize {
        match &self {
            PixelFormat::RGB8U  => 3,
            PixelFormat::RGBA8U => 4,
            PixelFormat::R8U    => 1,
            PixelFormat::RGB32U => 3 * 4,
            PixelFormat::RGBA32U=> 4 * 4,
            PixelFormat::R32U   => 4,

            PixelFormat::RGB32F => 3 * 4,
            PixelFormat::RGBA32F=> 4 * 4,
            PixelFormat::R32F   => 4,

            PixelFormat::D16    => 2,
            PixelFormat::D32    => 4,
            PixelFormat::D24S8  => 4,
            PixelFormat::D32S8  => 5,

            PixelFormat::RGB8(_)    => 3,
            PixelFormat::RGBA8(_)   => 4,
            PixelFormat::R8(_)      => 1,
        }
    }
}

struct GLShader {
    gl_id               : GLuint,

    vertex_attributes   : Vec<Vec<(String, GLuint)>>,

    vertex_uniforms     : Vec<(String, GLuint)>,
    vertex_surfaces     : Vec<(String, GLuint)>,

    pixel_uniforms      : Vec<(String, GLuint)>,
    pixel_surfaces      : Vec<(String, GLuint)>,
}

impl Drop for GLShader {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.gl_id) }
    }
}

struct GLPipeline {
    desc                : PipelineDesc,
}

struct GLFrameBuffer {
    gl_id               : GLuint,
    desc                : FrameBufferDesc,
}

impl Drop for GLFrameBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.gl_id as *const GLuint)
        }
    }
}

trait GLBlendFactor {
    fn gl_blend_factor(&self) -> GLenum;
}

impl GLBlendFactor for BlendFactor {
    fn gl_blend_factor(&self) -> GLenum {
        match self {
            BlendFactor::Zero               => gl::ZERO,
            BlendFactor::One                => gl::ONE,
            BlendFactor::SrcColor           => gl::SRC_COLOR,
            BlendFactor::OneMinusSrcColor   => gl::ONE_MINUS_SRC_COLOR,
            BlendFactor::SrcAlpha           => gl::SRC_ALPHA,
            BlendFactor::OneMinusSrcAlpha   => gl::ONE_MINUS_SRC_ALPHA,
            BlendFactor::DstColor           => gl::DST_COLOR,
            BlendFactor::OneMinusDstColor   => gl::ONE_MINUS_DST_COLOR,
            BlendFactor::DstAlpha           => gl::DST_ALPHA,
            BlendFactor::OneMinusDstAlpha   => gl::ONE_MINUS_DST_ALPHA,
            BlendFactor::SrcAlphaSaturate   => gl::SRC_ALPHA_SATURATE,
            BlendFactor::ConstantColor         => gl::CONSTANT_COLOR,
            BlendFactor::OneMinusConstantColor => gl::ONE_MINUS_CONSTANT_COLOR,
            BlendFactor::ConstantAlpha         => gl::CONSTANT_ALPHA,
            BlendFactor::OneMinusConstantAlpha => gl::ONE_MINUS_CONSTANT_ALPHA,
        }
    }
}
////////////////////////////////////////////////////////////////////////////////
/// Resource Container
////////////////////////////////////////////////////////////////////////////////

struct ResourceContainer<T> {
    res             : Vec<Option<T>>,
    free_res        : VecDeque<usize>,
}

impl<T> ResourceContainer<T> {
    fn new() -> Self { Self { res: Vec::new(), free_res: VecDeque::new() } }

    fn add(&mut self, t: T) -> usize {
        match self.free_res.len() {
            0 => {
                let idx = self.res.len();
                self.res.push(Some(t));
                idx
            },
            _ => {
                let idx = self.free_res.pop_front().unwrap();
                self.res[idx] = Some(t);
                idx
            }
        }
    }

    fn remove(&mut self, idx: usize) {
        match self.res[idx].as_ref() {
            Some(_) => {
                self.res[idx] = None;
                self.free_res.push_back(idx);
            },
            None => panic!("Deleting an already deleted object")
        }
    }
}

impl<T> Index<usize> for ResourceContainer<T> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        match &self.res[idx] {
            Some(t) => &t,
            None => panic!("Accessing invalid object index")
        }
    }
}

pub struct NullPayload {
    size            : usize,
}

impl Payload for NullPayload {
    fn size(&self) -> usize { self.size }
    fn ptr(&self) -> *const u8 { ::core::ptr::null() }
}

impl Drop for NullPayload {
    fn drop(&mut self) {}
}


////////////////////////////////////////////////////////////////////////////////
/// Driver
////////////////////////////////////////////////////////////////////////////////
pub struct Gles3Driver {
    device_buffers  : ResourceContainer<GLDeviceBuffer>,
    textures        : ResourceContainer<GLTexture>,
    render_targets  : ResourceContainer<GLRenderTarget>,
    shaders         : ResourceContainer<GLShader>,
    pipelines       : ResourceContainer<GLPipeline>,
    framebuffers    : ResourceContainer<GLFrameBuffer>,

    read_back_state : Option<ReadbackState>,

    rc              : AtomicIsize,

    caps            : DriverCaps,
}

impl Gles3Driver {
    fn new() -> Self {
        let mut max_rt_size    = 0;
        let mut max_tex_size   = 0;

        unsafe {
            gl::GetIntegerv(gl::MAX_RENDERBUFFER_SIZE, &mut max_rt_size as *mut GLint);
            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut max_tex_size as *mut GLint);
        }

        let min_surface_size    = std::cmp::min(4096, std::cmp::min(max_rt_size, max_tex_size));
        Self {
            device_buffers  : ResourceContainer::new(),
            textures        : ResourceContainer::new(),
            render_targets  : ResourceContainer::new(),
            shaders         : ResourceContainer::new(),
            pipelines       : ResourceContainer::new(),
            framebuffers    : ResourceContainer::new(),
            rc              : AtomicIsize::new(0),

            read_back_state : None,

            caps            : DriverCaps {
                max_2d_surface_dimension    : Dimensioni::new(min_surface_size, min_surface_size),
            }
        }
    }

    pub fn get_framebuffer_gl_id(&self, fb_id: usize) -> GLuint {
        self.framebuffers[fb_id].gl_id
    }

    fn initialize(mut self) -> DriverPtr {
        self.read_back_state    = Some(ReadbackState::new(&mut self));
        unsafe {
            gl::Enable(gl::SCISSOR_TEST);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            IntrusivePtr::from_raw_no_increment(IntrusivePtr::new(self).into_raw_mut() as *mut dyn Driver)
        }
    }

    fn buffer_type_to_gl(bt: &DeviceBufferDesc) -> GLenum {
        match bt {
            DeviceBufferDesc::Vertex(_)  => gl::ARRAY_BUFFER,
            DeviceBufferDesc::Index(_)   => gl::ELEMENT_ARRAY_BUFFER,
            DeviceBufferDesc::Pixel(_)   => gl::PIXEL_UNPACK_BUFFER,
        }
    }

    fn buffer_usage_to_gl(bt: &DeviceBufferDesc) -> GLenum {
        let usage =
            match bt {
                DeviceBufferDesc::Vertex(u) |
                DeviceBufferDesc::Index(u)  |
                DeviceBufferDesc::Pixel(u)  => u,
            };

        match usage {
            Usage::Static(_)    => gl::STATIC_DRAW,
            Usage::Streamed(_)  => gl::STREAM_DRAW,
            Usage::Dynamic(_)   => gl::DYNAMIC_DRAW,
        }
    }

    fn buffer_data(bt: &DeviceBufferDesc) -> Option<*const u8> {
        let usage =
            match bt {
                DeviceBufferDesc::Vertex(u) |
                DeviceBufferDesc::Index(u)  |
                DeviceBufferDesc::Pixel(u)  => u,
            };

        match usage {
            Usage::Static(b)    => Some(b.ptr()),
            Usage::Streamed(_)  => None,
            Usage::Dynamic(_)   => None,
        }
    }

    fn erase_buffer_data(bt: &DeviceBufferDesc) -> DeviceBufferDesc {
        let usage =
            match bt {
                DeviceBufferDesc::Vertex(u) |
                DeviceBufferDesc::Index(u)  |
                DeviceBufferDesc::Pixel(u)  => u,
            };

        let usage =
            match usage {
                Usage::Static(p)    => Usage::Static(Arc::new(NullPayload{ size: p.size() })),
                Usage::Streamed(s)  => Usage::Streamed(*s),
                Usage::Dynamic (s)  => Usage::Dynamic(*s),
            };

        match bt {
            DeviceBufferDesc::Vertex(_)  => DeviceBufferDesc::Vertex(usage),
            DeviceBufferDesc::Index(_)   => DeviceBufferDesc::Index(usage),
            DeviceBufferDesc::Pixel(_)   => DeviceBufferDesc::Pixel(usage),
        }
    }

    fn erase_texture_data(desc: &TextureDesc) -> TextureDesc {
        TextureDesc {
            sampler_desc: desc.sampler_desc.clone(),
            payload     : None,
        }
    }

    fn upload_texture(res: GLuint, desc: &SamplerDesc, data: Option<Arc<dyn Payload>>) {
        unsafe {
            match &desc.image_type {
                SamplerType::Sampler2D(pch_x, pch_y) => {
                    gl::BindTexture(gl::TEXTURE_2D, res);
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                    gl::PixelStorei(gl::PACK_ALIGNMENT, 1);

                    // TODO: if one day, we need to have device buffer, bind it here
                    //gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0);

                    let ptr = match &data {
                        Some(b) => b.ptr() as *const c_void,
                        None => ::core::ptr::null()
                    };

                    let (ptr2, len) = match &data {
                        Some(b) => (b.ptr(), b.size()),
                        None => (::core::ptr::null(), 0)
                    };

                    let sl = std::slice::from_raw_parts(ptr2, len);

                    gl::TexImage2D(gl::TEXTURE_2D,
                        0,
                        desc.pixel_format.gl_internal_format() as GLint,
                        pch_x.size as GLsizei,
                        pch_y.size as GLsizei,
                        0,
                        desc.pixel_format.gl_format(),
                        desc.pixel_format.gl_elem_type(),
                        ptr
                    );

                    Self::check_gl_error();

                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, Self::gl_wrap(&pch_x.wrap) as GLint);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, Self::gl_wrap(&pch_y.wrap) as GLint);
                    match &desc.pixel_format {
                        PixelFormat::R8(min_mag) | PixelFormat::RGB8(min_mag) | PixelFormat::RGBA8(min_mag) => {
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, Self::gl_filter(&min_mag.min_filter) as GLint);
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, Self::gl_filter(&min_mag.mag_filter) as GLint);
                        },
                        _ => {
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
                        }
                    }
                }
            }
        }
    }
    fn create_texture(desc: &SamplerDesc, data: Option<Arc<dyn Payload>>) -> GLuint {
        unsafe {

            let mut res : GLuint = 0;
            gl::GenTextures(1, &mut res);
            Self::upload_texture(res, desc, data);
            res
        }
    }

    fn create_render_target(desc: &SamplerDesc, _sample_size: usize) -> GLuint {
        unsafe {
            let mut res : GLuint = 0;
            gl::GenRenderbuffers(1, &mut res);
            match &desc.image_type {
                SamplerType::Sampler2D(pch_x, pch_y) => {
                    gl::BindRenderbuffer(gl::RENDERBUFFER, res);
                    gl::RenderbufferStorage(gl::RENDERBUFFER,
                        desc.pixel_format.gl_internal_format(),
                        pch_x.size as GLsizei,
                        pch_y.size as GLsizei
                    );
                    if gl::GetError() != gl::NO_ERROR {
                        panic!("Error creating render target");
                    }
                }
            }
            res
        }
    }

    fn gl_wrap(wm: &WrapMode) -> GLenum {
        match wm {
            WrapMode::Repeat => gl::REPEAT,
            WrapMode::ClampToEdge => gl::CLAMP_TO_EDGE,
            WrapMode::ClampToBorder => panic!("unsupported wrap mode!"),
            WrapMode::MirroredRepeat => gl::MIRRORED_REPEAT,
        }
    }

    fn gl_filter(filter: &Filter) -> GLenum {
        match filter {
            Filter::Nearest => gl::NEAREST,
            Filter::Linear => gl::LINEAR,
            Filter::NearestMipmapNearest => gl::NEAREST_MIPMAP_NEAREST,
            Filter::NearestMipmapLinear => gl::NEAREST_MIPMAP_LINEAR,
            Filter::LinearMipmapNearest => gl::LINEAR_MIPMAP_NEAREST,
            Filter::LinearMipmapLinear => gl::LINEAR_MIPMAP_LINEAR,
        }
    }

    fn load_shader(src: &str, ty: GLenum) -> Option<GLuint> {
        unsafe {
            let shader = gl::CreateShader(ty);
            if shader == 0 {
                return None
            }

            let c_str = std::ffi::CString::new(src.as_bytes()).unwrap();
            gl::ShaderSource(shader, 1, &(c_str.as_ptr() as *const i8), core::ptr::null());
            gl::CompileShader(shader);

            let mut compiled = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compiled);

            let mut info_len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut info_len);
            if info_len > 1 {
                let s = alloc_string(info_len as usize);
                gl::GetShaderInfoLog(shader, info_len as GLsizei, core::ptr::null_mut(), s.as_ptr() as *mut GLchar);
                let sht =
                    match ty {
                        gl::VERTEX_SHADER => "vertex shader",
                        gl::FRAGMENT_SHADER => "fragment shader",
                        _ => panic!("invalid shader type")
                    };

                println!("[{}] Compilation Log: {}", sht, s);
                free_string(s, info_len as usize);
            }

            if compiled == 0 {
                gl::DeleteShader(shader);
                return None
            }
            Some(shader)
        }
    }

    fn delete_device_buffer(&mut self, buff: usize) {
        self.device_buffers.remove(buff)
    }

    fn delete_texture(&mut self, surf: usize) {
        self.textures.remove(surf)
    }

    fn delete_render_target(&mut self, surf: usize) {
        self.render_targets.remove(surf)
    }

    fn delete_shader(&mut self, shader: usize) {
        self.shaders.remove(shader)
    }

    fn delete_pipeline(&mut self, pipe: usize) {
        self.pipelines.remove(pipe)
    }

    fn delete_frame_buffer(&mut self, pass: usize) {
        self.framebuffers.remove(pass)
    }

    pub fn check_gl_error() {
        unsafe {
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                panic!("Error : {:#X}", error);
            }
        }
    }

    fn read_back_state(&mut self) -> &ReadbackState {
        match &self.read_back_state {
            Some(rb) => &rb,
            _ => panic!("readback is None")
        }
    }
}


impl Driver for Gles3Driver {
    fn get_caps(&self) -> &DriverCaps { &self.caps }

    fn create_device_buffer(&mut self, desc: DeviceBufferDesc) -> Option<DeviceBufferPtr> {
        unsafe {
            let data = Self::buffer_data(&desc);
            let mut buff = 0;
            gl::GenBuffers(1, &mut buff);
            gl::BindBuffer(Self::buffer_type_to_gl(&desc), buff);
            let buff_data =
                match data {
                    Some(d) => d,
                    None    => std::ptr::null(),
                };

            gl::BufferData(Self::buffer_type_to_gl(&desc), desc.size() as GLsizeiptr, buff_data as *const rs_ctypes::c_void, Self::buffer_usage_to_gl(&desc));

            let gl_buff = GLDeviceBuffer { gl_id: buff, desc: Self::erase_buffer_data(&desc) };
            let idx = self.device_buffers.add(gl_buff);

            let iptr : IntrusivePtr<dyn Driver>= IntrusivePtr::from_raw_increment(self as *mut Self as *mut dyn Driver);

            Some(DeviceBufferPtr::new(DeviceBuffer::new(ResourceType::DeviceBuffer, idx, desc, Some(iptr))))
        }
    }

    fn create_texture(&mut self, desc: TextureDesc) -> Option<TexturePtr> {
        let new_desc = Self::erase_texture_data(&desc);
        let idx = Self::create_texture(&desc.sampler_desc, desc.payload);
        let img = GLTexture { gl_id: idx };
        let idx = self.textures.add(img);

        let iptr : IntrusivePtr<dyn Driver>= unsafe { IntrusivePtr::from_raw_increment(self as *mut Self as *mut dyn Driver) };

        Some(TexturePtr::new(Texture::new(ResourceType::Texture, idx, new_desc, Some(iptr))))
    }

    fn create_render_target(&mut self, desc: RenderTargetDesc) -> Option<RenderTargetPtr> {
        let idx = Self::create_render_target(&desc.sampler_desc, desc.sample_count);
        let img = GLRenderTarget { gl_id: idx };
        let idx = self.render_targets.add(img);

        let iptr : IntrusivePtr<dyn Driver>= unsafe { IntrusivePtr::from_raw_increment(self as *mut Self as *mut dyn Driver) };

        Some(RenderTargetPtr::new(RenderTarget::new(ResourceType::RenderTarget, idx, desc, Some(iptr))))
    }

    fn create_shader(&mut self, desc: ShaderDesc) -> Option<ShaderPtr> {
        let desc_copy2 = desc.clone();
        unsafe {
            let program_object = gl::CreateProgram();
            if program_object == 0 {
                return None
            }

            let vertex_shader    = Self::load_shader(desc.vertex_shader.as_str(), gl::VERTEX_SHADER);
            let fragment_shader  = Self::load_shader(desc.pixel_shader.as_str(), gl::FRAGMENT_SHADER);

            match (vertex_shader, fragment_shader) {
                (None, None) => (),
                (None, Some(f)) => gl::DeleteShader(f),
                (Some(v), None) => gl::DeleteShader(v),
                (Some(v), Some(f)) => {
                    gl::AttachShader(program_object, v);
                    gl::AttachShader(program_object, f);
                    gl::LinkProgram(program_object);

                    let mut linked = 0;
                    gl::GetProgramiv(program_object, gl::LINK_STATUS, &mut linked);
                    let mut info_len = 0;
                    gl::GetProgramiv(program_object, gl::INFO_LOG_LENGTH, &mut info_len);
                    if info_len > 1 {
                        let s = alloc_string(info_len as usize);
                        gl::GetProgramInfoLog(program_object, info_len as GLsizei, core::ptr::null_mut(), s.as_ptr() as *mut GLchar);
                        println!("Shader Linking: {}", s);
                        free_string(s, info_len as usize);
                    }


                    // done with the shaders
                    gl::DetachShader(program_object, v);
                    gl::DetachShader(program_object, f);

                    gl::DeleteShader(f);
                    gl::DeleteShader(v);

                    if linked == 0 {
                        gl::DeleteProgram(program_object);
                        return None
                    }
                }
            }

            let mut vertex_attributes = Vec::new();

            for l in desc.vertex_attributes {
                let mut vas = Vec::new();
                for a in l {
                    let mut s = a.clone();
                    s.push('\0');

                    let au = gl::GetAttribLocation(program_object, s.as_bytes().as_ptr() as *const GLchar);
                    if au < 0 {
                        println!("attribute {} not found in shader", s);
                        return None // will leak shaders!
                    }
                    vas.push((s, au as GLuint));
                }
                vertex_attributes.push(vas);
            }

            let mut vertex_uniforms = Vec::new();

            for u in desc.vertex_uniforms {
                let mut s = u.clone();
                s.push('\0');

                let au = gl::GetUniformLocation(program_object, s.as_bytes().as_ptr() as *const GLchar);
                if au < 0 {
                    println!("uniform {} not found in shader", s);
                    return None // will leak shaders!
                }
                vertex_uniforms.push((s, au as GLuint));
            }

            let mut vertex_surfaces = Vec::new();

            for u in desc.vertex_surfaces {
                let mut s = u.clone();
                s.push('\0');

                let au = gl::GetUniformLocation(program_object, s.as_bytes().as_ptr() as *const GLchar);
                if au < 0 {
                    println!("vertex texture {} not found in shader", s);
                    return None // will leak shaders!
                }
                // TODO: use glGetActiveUniform to get sampler type
                vertex_surfaces.push((s, au as GLuint));
            }

            let mut pixel_uniforms = Vec::new();

            for u in desc.pixel_uniforms {
                let mut s = u.clone();
                s.push('\0');

                let au = gl::GetUniformLocation(program_object, s.as_bytes().as_ptr() as *const GLchar);
                if au < 0 {
                    println!("uniform {} not found in shader", s);
                    return None // will leak shaders!
                }
                pixel_uniforms.push((s, au as GLuint));
            }

            let mut pixel_surfaces = Vec::new();

            for u in desc.pixel_surfaces {
                let mut s = u.clone();
                s.push('\0');

                let au = gl::GetUniformLocation(program_object, s.as_bytes().as_ptr() as *const GLchar);
                if au < 0 {
                    println!("pixel texture {} not found in shader", s);
                    return None // will leak shaders!
                }
                // TODO: use glGetActiveUniform to get sampler type
                pixel_surfaces.push((s, au as GLuint));
            }

            let gl_shader =
                GLShader {
                    gl_id               : program_object,

                    vertex_attributes   : vertex_attributes,

                    vertex_uniforms     : vertex_uniforms,
                    vertex_surfaces     : vertex_surfaces,

                    pixel_uniforms      : pixel_uniforms,
                    pixel_surfaces      : pixel_surfaces,
                };

            let idx = self.shaders.add(gl_shader);

            let iptr : IntrusivePtr<dyn Driver>= IntrusivePtr::from_raw_increment(self as *mut Self as *mut dyn Driver);

            Some(ShaderPtr::new(Shader::new(ResourceType::Shader, idx, desc_copy2, Some(iptr))))
        }
    }

    fn create_pipeline(&mut self, desc: PipelineDesc) -> Option<PipelinePtr> {
        let idx = self.pipelines.add(GLPipeline { desc: desc.clone() });

        let iptr : IntrusivePtr<dyn Driver>= unsafe { IntrusivePtr::from_raw_increment(self as *mut Self as *mut dyn Driver) };

        Some(PipelinePtr::new(Pipeline::new(ResourceType::Pipeline, idx, desc, Some(iptr))))
    }

    fn create_frame_buffer(&mut self, desc: FrameBufferDesc) -> Option<FrameBufferPtr> {
        unsafe {
            let mut res : GLuint = 0;
            gl::GenFramebuffers(1, &mut res);
            gl::BindFramebuffer(gl::FRAMEBUFFER, res);

            let mut colors : [u32; 4] = [0; 4];
            for i in 0..4u32 {
                match &desc.color_attachements[i as usize] {
                    Some(SurfaceAttachment::Texture(ca)) => {
                        let gl_id = self.textures[ca.res_id()].gl_id;
                        colors[i as usize] = gl_id;
                        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0 + i, gl::TEXTURE_2D, gl_id, 0);
                    },
                    Some(SurfaceAttachment::RenderTarget(ca)) => {
                        let gl_id = self.render_targets[ca.res_id()].gl_id;
                        colors[i as usize] = gl_id;
                        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0 + i, gl::RENDERBUFFER, gl_id);
                    },
                    _ => ()
                }
            }

            match &desc.depth_stencil_attachement {
                SurfaceAttachment::RenderTarget(ca) =>  {
                    let gl_id = self.render_targets[ca.res_id()].gl_id;
                    gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, gl_id);
                },
                SurfaceAttachment::Texture(ca) => {
                    let gl_id = self.textures[ca.res_id()].gl_id;
                    gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, gl_id, 0);
                }
            }

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                println!("Framebuffer is not complete!");
                gl::DeleteFramebuffers(1, &mut res);
                return None
            }

            let mut color0 = 0;
            gl::GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0 as GLenum, gl::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME as GLenum, &mut color0);
            assert_ne!(colors[0], 0);
            assert_eq!(colors[0], color0 as u32);

            Self::check_gl_error();

            let idx = self.framebuffers.add(GLFrameBuffer { desc: desc.clone(), gl_id: res });

            let iptr : IntrusivePtr<dyn Driver>= IntrusivePtr::from_raw_increment(self as *mut Self as *mut dyn Driver);

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            Some(FrameBufferPtr::new(FrameBuffer::new(ResourceType::FrameBuffer, idx, desc, Some(iptr))))
        }
    }

    fn delete_resource(&mut self, resource_type: &ResourceType, res_id: usize) {
        match resource_type {
            ResourceType::DeviceBuffer  => self.delete_device_buffer(res_id),
            ResourceType::Texture       => self.delete_texture(res_id),
            ResourceType::RenderTarget  => self.delete_render_target(res_id),
            ResourceType::Shader        => self.delete_shader(res_id),
            ResourceType::Pipeline      => self.delete_pipeline(res_id),
            ResourceType::FrameBuffer   => self.delete_frame_buffer(res_id),
        }
    }

    fn draw(&mut self, pipe: &Pipeline, bindings: &Bindings, uniforms: *const c_void, prim_count: u32, instance_count: u32) {
        unsafe {
            let gl_pipe = &self.pipelines[pipe.res_id()];
            let gl_prog = &self.shaders[gl_pipe.desc.shader.res_id()];

            // blend
            match &gl_pipe.desc.blend {
                BlendOp::Add(blend) | BlendOp::Subtract(blend) => {
                    gl::Enable(gl::BLEND);
                    gl::BlendFuncSeparate(
                        blend.src_factor_rgb.gl_blend_factor(),
                        blend.dst_factor_rgb.gl_blend_factor(),
                        blend.src_factor_alpha.gl_blend_factor(),
                        blend.dst_factor_alpha.gl_blend_factor());
                },
                _ => gl::Disable(gl::BLEND),
            }

            match &gl_pipe.desc.blend {
                BlendOp::Add(_) => gl::BlendEquationSeparate(gl::FUNC_ADD, gl::FUNC_ADD),
                BlendOp::Subtract(_) => gl::BlendEquationSeparate(gl::FUNC_SUBTRACT, gl::FUNC_SUBTRACT),
                BlendOp::ReverseSubtract(_) => gl::BlendEquationSeparate(gl::FUNC_REVERSE_SUBTRACT, gl::FUNC_REVERSE_SUBTRACT),
                _ => ()
            }

            let (gl_prim, gl_elem_count) =
                match gl_pipe.desc.primitive_type {
                    PrimitiveType::Lines        => (gl::LINES, 2 * prim_count),
                    PrimitiveType::LineStrip    => (gl::LINE_STRIP, 1 + prim_count),
                    PrimitiveType::Points       => (gl::POINTS, prim_count),
                    PrimitiveType::Triangles    => (gl::TRIANGLES, 3 * prim_count),
                    PrimitiveType::TriangleStrip    => (gl::TRIANGLE_STRIP, 2 + prim_count)
                };

            match gl_pipe.desc.cull_mode {
                CullMode::None => gl::Disable(gl::CULL_FACE),
                CullMode::Winding => gl::Enable(gl::CULL_FACE),
            }

            match gl_pipe.desc.face_winding {
                FaceWinding::CCW => gl::CullFace(gl::BACK),
                FaceWinding::CW => gl::CullFace(gl::FRONT),
            }

            if gl_pipe.desc.depth_test {
                gl::Enable(gl::DEPTH_TEST)
            } else {
                gl::Disable(gl::DEPTH_TEST)
            }

            gl::DepthMask(if gl_pipe.desc.depth_write { gl::TRUE } else { gl::FALSE } as GLboolean);

            match gl_pipe.desc.polygon_offset {
                PolygonOffset::None => gl::PolygonOffset(0.0, 0.0),
                PolygonOffset::FactorUnits(factor, units) => gl::PolygonOffset(factor, units),
            }

            gl::UseProgram(gl_prog.gl_id);
            for (l, layout) in gl_pipe.desc.buffer_layouts.iter().enumerate() {
                let gl_vb = &self.device_buffers[bindings.vertex_buffers[layout.buffer_id].res_id()];
                gl::BindBuffer(gl::ARRAY_BUFFER, gl_vb.gl_id);
                for (i, a) in layout.vertex_attributes.iter().enumerate() {
                    let aidx = &gl_prog.vertex_attributes[l][i];
                    gl::EnableVertexAttribArray(aidx.1);
                    match a.format() {
                        VertexFormat::Int |
                        VertexFormat::Int2 |
                        VertexFormat::Int3 |
                        VertexFormat::Int4 |
                        VertexFormat::UInt |
                        VertexFormat::UInt2 |
                        VertexFormat::UInt3 |
                        VertexFormat::UInt4  => {
                            gl::VertexAttribIPointer(aidx.1, a.format().gl_elem_count() as GLint, a.format().gl_elem_type(), layout.stride as GLint, a.offset() as *const c_void);
                        },
                        _ => {
                            gl::VertexAttribPointer(aidx.1, a.format().gl_elem_count() as GLint, a.format().gl_elem_type(), a.format().gl_is_normalized(), layout.stride as GLint, a.offset() as *const c_void);
                        }
                    }
                    gl::VertexAttribDivisor(aidx.1, layout.divisor as GLuint);
                }
            }

            setup_uniforms(uniforms, gl_pipe.desc.uniform_descs.as_slice(), gl_prog.vertex_uniforms.as_slice());

            for (i, t) in bindings.vertex_images.iter().enumerate() {
                let location = gl_prog.vertex_surfaces[i].1;
                gl::ActiveTexture(((gl::TEXTURE0 as usize) + i) as GLenum);
                gl::BindTexture(gl::TEXTURE_2D, self.textures[t.res_id()].gl_id as GLuint);
                gl::Uniform1i(location as GLint, i as GLint);
            }

            let pixel_sampler_offset = bindings.vertex_images.len();

            for (i, t) in bindings.pixel_images.iter().enumerate() {
                let location = gl_prog.pixel_surfaces[i].1;
                gl::ActiveTexture(((gl::TEXTURE0 as usize) + i + pixel_sampler_offset) as GLenum);
                gl::BindTexture(gl::TEXTURE_2D, self.textures[t.res_id()].gl_id as GLuint);
                gl::Uniform1i(location as GLint, (i + pixel_sampler_offset) as GLint);
            }

            match &bindings.index_buffer {
                Some(ib) => {
                    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.device_buffers[ib.res_id()].gl_id);

                    let itype =
                        match gl_pipe.desc.index_type {
                            IndexType::None => panic!("attempt to bind an index buffer to a pipeline that doesn't support it"),
                            IndexType::UInt16 => gl::UNSIGNED_SHORT,
                            IndexType::UInt32 => gl::UNSIGNED_INT,
                        };

                    gl::DrawElementsInstanced(gl_prim, gl_elem_count as GLsizei, itype, core::ptr::null() as *const rs_ctypes::c_void, instance_count as GLint);
                },
                None => {
                    if gl_pipe.desc.index_type != IndexType::None {
                        panic!("no index buffer bound but index type exist in pipeline")
                    }
                    gl::DrawArraysInstanced(gl_prim, 0, gl_elem_count as GLsizei, instance_count as GLint);
                }
            }

            for l in &gl_prog.vertex_attributes {
                for v in l {
                    gl::DisableVertexAttribArray(v.1);
                }
            }
        }
    }

    fn begin_pass(&mut self, pass: &Pass) {
        unsafe {
            gl::Flush();
            gl::Viewport(0, 0, pass.width as i32, pass.height as i32);
            gl::Scissor(0, 0, pass.width as i32, pass.height as i32);

            match &pass.frame_buffer {
                None => {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                    Self::check_gl_error();
                },

                Some(fb) => {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffers[fb.res_id()].gl_id);
                    Self::check_gl_error();
                },
            }

            // set the draw buffers
            match &pass.frame_buffer {
                Some(fb) => {
                    let fb_ref = &self.framebuffers[fb.res_id()];
                    let mut draw_buffer : [GLenum; 4] = [ gl::NONE, gl::NONE, gl::NONE, gl::NONE ];
                    for (idx, attach) in fb_ref.desc.color_attachements.iter().enumerate() {
                        match attach {
                            Some(_) => {
                                draw_buffer[idx] = gl::COLOR_ATTACHMENT0 + (idx as GLenum);
                            },
                            None => (),
                        };
                    }

                    gl::DrawBuffers(4, &draw_buffer as *const GLenum);
                    Self::check_gl_error();

                    for idx in 0..4 {
                        match pass.color_actions[idx] {
                            ColorPassAction::Clear(col) => {
                                let surf = &fb_ref.desc.color_attachements[idx];
                                match surf.as_ref() {
                                    Some(surf_ref) => {
                                        match surf_ref.pixel_format().to_orig_surface_type() {
                                            OrigSurfaceType::UInt => {
                                                let i_cols : [GLuint; 4] = [col.x as GLuint, col.y as GLuint, col.z as GLuint, col.w as GLuint];
                                                gl::ClearBufferuiv(gl::COLOR as GLenum, idx as GLint, i_cols.as_ptr() as *const GLuint);
                                            },
                                            OrigSurfaceType::Float => {
                                                let float_col = color4b_to_color4f(col);
                                                let f_cols : [GLfloat; 4] = [float_col.x, float_col.y, float_col.z, float_col.w];
                                                gl::ClearBufferfv(gl::COLOR as GLenum, idx as GLint, f_cols.as_ptr() as *const GLfloat);
                                            }
                                        }
                                    },
                                    _ => {  // assume that it's the default fixed point frame buffer
                                        let float_col = color4b_to_color4f(col);
                                        let f_cols : [GLfloat; 4] = [float_col.x, float_col.y, float_col.z, float_col.w];
                                        gl::ClearBufferfv(gl::COLOR as GLenum, idx as GLint, f_cols.as_ptr() as *const GLfloat);
                                    }
                                }
                                Self::check_gl_error();
                            },
                            _ => ()
                        }
                    }

                    // clear the depth
                    match pass.depth_action {
                        DepthPassAction::Clear(f) => {
                            gl::ClearBufferfv(gl::DEPTH as GLenum, 0, &f as *const _ as *const GLfloat);
                            Self::check_gl_error();

                        },
                        _ => ()
                    }
                },
                None => {
                    // TODO: does glClearBufferfv works here?
                    let mut bits    = 0;
                    for idx in 0..4 {
                        match pass.color_actions[idx] {
                            ColorPassAction::Clear(col) => {
                                let float_col = color4b_to_color4f(col);
                                let f_cols : [GLfloat; 4] = [float_col.x, float_col.y, float_col.z, float_col.w];
                                gl::ClearColor(f_cols[0], f_cols[1], f_cols[2], f_cols[3]);

                                bits |= gl::COLOR_BUFFER_BIT;
                            },
                            _ => ()
                        }
                    }

                    match pass.depth_action {
                        DepthPassAction::Clear(depth) => {
                            gl::ClearDepthf(depth);
                            bits   |= gl::DEPTH_BUFFER_BIT;
                        },
                        _ => ()
                    }
                    gl::Clear(bits);
                    Self::check_gl_error();
                }
            }

        }
    }

    fn end_pass(&mut self) {
    }

    fn set_viewport(&mut self, x: u32, y: u32, w: u32, h: u32) {
        unsafe { gl::Viewport(x as GLint, y as GLint, w as GLsizei, h as GLsizei) }
    }

    fn set_scissor(&mut self, x: u32, y: u32, w: u32, h: u32) {
        unsafe {
            gl::Scissor(x as GLint, y as GLint, w as GLsizei, h as GLsizei)
        }
    }

    fn update_device_buffer(&mut self, dev_buf: &mut DeviceBufferPtr, offset: usize, pl: Arc<dyn Payload>) {
        unsafe {
            match self.device_buffers[dev_buf.res_id()].desc {
                DeviceBufferDesc::Vertex(Usage::Static(_))   |
                DeviceBufferDesc::Index(Usage::Static(_))    |
                DeviceBufferDesc::Pixel(Usage::Static(_))    => {
                    //return None
                    panic!("trying to update static buffer")
                },
                _ => (),    // TODO: Streamed can be done once per frame ?
            };

            let buff_size   = self.device_buffers[dev_buf.res_id()].desc.size();
            if pl.size() + offset > buff_size {
                panic!("payload of size {} exceeds device buffer size of {}", pl.size() + offset, buff_size)
            }

            let target =
                match self.device_buffers[dev_buf.res_id()].desc {
                    DeviceBufferDesc::Vertex(_)  => gl::ARRAY_BUFFER,
                    DeviceBufferDesc::Index(_)   => gl::ELEMENT_ARRAY_BUFFER,
                    DeviceBufferDesc::Pixel(_)   => gl::PIXEL_UNPACK_BUFFER,
                };
            gl::BindBuffer(target, self.device_buffers[dev_buf.res_id()].gl_id as GLuint);
            let ptr = gl::MapBufferRange(target, offset as GLintptr, pl.size() as GLsizeiptr, gl::MAP_WRITE_BIT as GLbitfield) as *mut u8;
            Self::check_gl_error();

            std::ptr::copy_nonoverlapping(pl.ptr() as *mut u8, ptr, pl.size());

            assert_eq!(gl::UnmapBuffer(target), gl::TRUE as GLboolean);
            Self::check_gl_error();
        }
    }

    fn update_texture(&mut self, dev_buf: &mut TexturePtr, pl: Arc<dyn Payload>) {
        // TODO: check payload size and format
        let res_id  = dev_buf.res_id();
        let gl_id   = self.textures[res_id].gl_id;
        Self::upload_texture(gl_id, &dev_buf.desc().sampler_desc, Some(pl));
    }

    fn read_back(&mut self, surface: &TexturePtr, x: u32, y: u32, w: u32, h: u32) -> Option<ReadbackPayload> {
        unsafe {
            let rb = self.read_back_state() as *const ReadbackState as *mut ReadbackState;
            (&mut (*rb)).read_surface(self, surface, x, y, w, h)
        }
    }
}

impl IntrusiveCounter for Gles3Driver {
    fn increment(&mut self) { self.rc.fetch_add(1, Ordering::SeqCst); }
    fn decrement(&mut self) -> isize {
        self.rc.fetch_sub(1, Ordering::SeqCst)
    }
}

unsafe impl Send for Gles3Driver {}
unsafe impl Sync for Gles3Driver {}

impl Drop for Gles3Driver {
    fn drop(&mut self) {
        println!("Gles3Driver dropped - All is good!")
    }
}

pub fn get_driver() -> DriverPtr {
    unsafe {
        let mut range : [GLint; 2] = [0, 0];
        let mut precision = 0;

        gl::GetShaderPrecisionFormat(gl::FRAGMENT_SHADER, gl::HIGH_FLOAT, range.as_mut_ptr(), &mut precision);
        println!("highp float range: {:?} - precision: {}", range, precision);

        gl::GetShaderPrecisionFormat(gl::FRAGMENT_SHADER, gl::HIGH_INT, range.as_mut_ptr(), &mut precision);
        println!("highp int range: {:?} - precision: {}", range, precision);

        gl::GetShaderPrecisionFormat(gl::FRAGMENT_SHADER, gl::MEDIUM_FLOAT, range.as_mut_ptr(), &mut precision);
        println!("mediump float range: {:?} - precision: {}", range, precision);

        gl::GetShaderPrecisionFormat(gl::FRAGMENT_SHADER, gl::MEDIUM_INT, range.as_mut_ptr(), &mut precision);
        println!("mediump int range: {:?} - precision: {}", range, precision);

        gl::GetShaderPrecisionFormat(gl::FRAGMENT_SHADER, gl::LOW_FLOAT, range.as_mut_ptr(), &mut precision);
        println!("lowp float range: {:?} - precision: {}", range, precision);

        gl::GetShaderPrecisionFormat(gl::FRAGMENT_SHADER, gl::LOW_INT, range.as_mut_ptr(), &mut precision);
        println!("lowp int range: {:?} - precision: {}", range, precision);

    }
    Gles3Driver::new().initialize()
}
