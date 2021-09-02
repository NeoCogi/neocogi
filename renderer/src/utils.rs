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
use rs_ctypes::*;
use rs_math3d::*;

///
/// create a frame buffer with only color & depth render targets
///
pub fn create_color_depth_frame_buffer(driver: &mut DriverPtr, width: usize, height: usize) -> Option<FrameBufferPtr> {
    let color_tex_desc      = SamplerDesc::default(width, height).with_pixel_format(PixelFormat::RGBA8(MinMagFilter::default()));
    let color_buffer_desc   = TextureDesc { sampler_desc: color_tex_desc, payload: None };
    let color_buffer        = driver.create_texture(color_buffer_desc).unwrap();

    let depth_tex_desc      = SamplerDesc::default(width, height).with_pixel_format(PixelFormat::D32);
    let depth_buffer_desc   = RenderTargetDesc { sampler_desc: depth_tex_desc, sample_count: 0 };
    let depth_buffer        = driver.create_render_target(depth_buffer_desc).unwrap();

    let fb_desc             = FrameBufferDesc {
        color_attachements: [Some(SurfaceAttachment::Texture(color_buffer)), None, None, None],
        depth_stencil_attachement: SurfaceAttachment::RenderTarget(depth_buffer)
    };

    driver.create_frame_buffer(fb_desc)
}

///
/// create a frame buffer with color, normal & depth render targets
///
pub fn create_color_normal_depth_frame_buffer(driver: &mut DriverPtr, width: usize, height: usize) -> Option<FrameBufferPtr> {
    let normal_tex_desc     = SamplerDesc::default(width, height).with_pixel_format(PixelFormat::RGBA32F);
    let normal_buffer_desc  = TextureDesc { sampler_desc: normal_tex_desc, payload: None };
    let normal_buffer       = driver.create_texture(normal_buffer_desc).unwrap();

    let color_tex_desc      = SamplerDesc::default(width, height).with_pixel_format(PixelFormat::RGBA8(MinMagFilter::default()));
    let color_buffer_desc   = TextureDesc { sampler_desc: color_tex_desc, payload: None };
    let color_buffer        = driver.create_texture(color_buffer_desc).unwrap();

    let depth_tex_desc      = SamplerDesc::default(width, height).with_pixel_format(PixelFormat::D32);
    let depth_buffer_desc   = RenderTargetDesc { sampler_desc: depth_tex_desc, sample_count: 0 };
    let depth_buffer        = driver.create_render_target(depth_buffer_desc).unwrap();

    let fb_desc             = FrameBufferDesc {
        color_attachements: [
            Some(SurfaceAttachment::Texture(color_buffer)),
            Some(SurfaceAttachment::Texture(normal_buffer)),
            None,
            None],
        depth_stencil_attachement: SurfaceAttachment::RenderTarget(depth_buffer)
    };
    driver.create_frame_buffer(fb_desc)
}

crate::render_data! {
    vertex QuadVertex {
        position    : Vec2f,
        uv          : Vec2f,
    }
}

pub struct ScreenQuad {
    vb          : DeviceBufferPtr,
    ib          : DeviceBufferPtr,
    u_pipeline  : PipelinePtr,
    f_pipeline  : PipelinePtr,
}

impl ScreenQuad {
    pub fn get_vb(&self) -> &DeviceBufferPtr { &self.vb }
    pub fn get_ib(&self) -> &DeviceBufferPtr { &self.ib }
}

static COPY_VERTEX_SHADER : &'static str = "
#version 300 es
precision highp float;
in          vec2        position;
in          vec2        uv;

out highp   vec2        vUV;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    vUV = uv;
}\0";

static COPY_UINT_PIXEL_SHADER : &'static str = "
#version 300 es
precision highp float;
precision highp usampler2D;

in highp    vec2        vUV;

uniform     usampler2D   uTexture;

out         vec4        fragColor;

void main() {
    uvec4 texel = texture(uTexture, vUV);
    vec4 col = vec4(texel) / 255.0;
    fragColor = col;
}\0";

static COPY_FLOAT_PIXEL_SHADER : &'static str = "
#version 300 es
precision highp float;
precision highp usampler2D;

in highp    vec2        vUV;

uniform     sampler2D   uTexture;

out         vec4        fragColor;

void main() {
    fragColor = texture(uTexture, vUV);
}\0";


impl ScreenQuad {
    fn create_copy_shader(driver: &mut DriverPtr, orig_surface_type: OrigSurfaceType) -> ShaderPtr {
        let shader_desc =
        ShaderDesc {
            vertex_shader       : String::from(COPY_VERTEX_SHADER),
            pixel_shader        : String::from(
                match orig_surface_type {
                    OrigSurfaceType::UInt => COPY_UINT_PIXEL_SHADER,
                    OrigSurfaceType::Float => COPY_FLOAT_PIXEL_SHADER,
                }),

            vertex_attributes   : vec!{ QuadVertex::get_attribute_names() },
            vertex_uniforms     : Vec::new(),
            vertex_surfaces     : Vec::new(),

            pixel_uniforms      : Vec::new(),
            pixel_surfaces      : Vec::from([String::from("uTexture")]),
        };

        driver.create_shader(shader_desc).unwrap()
    }

    fn create_copy_pipeline(driver: &mut DriverPtr, orig_surface_type: OrigSurfaceType) -> PipelinePtr {
        let vertex_layout = VertexBufferLayout {
            buffer_id           : 0,
            vertex_attributes   : QuadVertex::get_attribute_descriptors(),
            stride              : QuadVertex::stride(),
            divisor             : 0,
        };

        let model_pipeline_desc = PipelineDesc {
            primitive_type      : PrimitiveType::Triangles,
            shader              : Self::create_copy_shader(driver, orig_surface_type),
            buffer_layouts      : vec! { vertex_layout },
            uniform_descs       : vec! {},
            index_type          : IndexType::UInt32,
            face_winding        : FaceWinding::CCW,
            cull_mode           : CullMode::Winding,
            depth_write         : true,
            depth_test          : true,
            blend               : BlendOp::None,
        };

        driver.create_pipeline(model_pipeline_desc).unwrap()
    }

    pub fn new(driver: &mut DriverPtr) -> Self {

        let quad_verts  = vec! {
            QuadVertex { position: Vec2f::new(-1.0, -1.0), uv: Vec2f::new(0.0, 0.0) },
            QuadVertex { position: Vec2f::new( 1.0, -1.0), uv: Vec2f::new(1.0, 0.0) },
            QuadVertex { position: Vec2f::new( 1.0,  1.0), uv: Vec2f::new(1.0, 1.0) },
            QuadVertex { position: Vec2f::new(-1.0,  1.0), uv: Vec2f::new(0.0, 1.0) },
        };

        let quad_index  : Vec<u32> = vec! {
            0, 1, 2,
            2, 3, 0,
        };

        let vb_desc = DeviceBufferDesc::Vertex(Usage::Static(Box::new(quad_verts)));
        let vb = driver.create_device_buffer(vb_desc).unwrap();

        let ib_desc = DeviceBufferDesc::Index(Usage::Static(Box::new(quad_index)));
        let ib = driver.create_device_buffer(ib_desc).unwrap();

        Self {
            vb,
            ib,
            u_pipeline: Self::create_copy_pipeline(driver, OrigSurfaceType::UInt),
            f_pipeline: Self::create_copy_pipeline(driver, OrigSurfaceType::Float),
        }
    }

    pub fn render(&self, driver: &mut DriverPtr, tex: &TexturePtr) {
        let bindings = Bindings {
            vertex_buffers  : vec!{ self.vb.clone() },
            index_buffer    : Some(self.ib.clone()),

            vertex_images   : Vec::from([]),
            pixel_images    : Vec::from([tex.clone()]),
        };

        let pipeline    =
            match tex.desc().sampler_desc.pixel_format.to_orig_surface_type() {
                OrigSurfaceType::UInt   => &self.u_pipeline,
                OrigSurfaceType::Float  => &self.f_pipeline,
            };

        driver.draw(pipeline, &bindings, core::ptr::null() as *const c_void, 2, 1);
    }
}