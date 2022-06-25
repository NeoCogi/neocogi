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
use neocogi::glfw::{Action, Context, Key};

use neocogi::*;
use neocogi::rs_math3d::*;

use neocogi::renderer::*;

use std::sync::*;

static VERTEX_SHADER : &'static str = "
#version 300 es
in  highp   vec4        position;
in  highp   vec4        color;
out highp   vec4        v_color;
void main() {
    gl_Position = position;
    v_color     = color;
}";

static PIXEL_SHADER : &'static str = "
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

fn init_render_objects(driver: &mut DriverPtr) -> PipelinePtr {
    let mut model_attribs = Vec::new();
    model_attribs.push(Vertex::get_attribute_names());

    let model_shader_desc =
        ShaderDesc {
            vertex_shader       : String::from(VERTEX_SHADER),
            pixel_shader        : String::from(PIXEL_SHADER),

            vertex_attributes   : model_attribs,
            vertex_uniforms     : Vec::new(),
            vertex_surfaces     : Vec::new(),

            pixel_uniforms      : Vec::new(),
            pixel_surfaces      : Vec::new(),
        };

    let model_program = driver.create_shader(model_shader_desc).unwrap();

    let vertex_layout = VertexBufferLayout {
        buffer_id           : 0,
        vertex_attributes   : Vertex::get_attribute_descriptors(),
        stride              : Vertex::stride(),
        divisor             : 0,
    };

    let tri_pipeline_desc = PipelineDesc {
        primitive_type      : PrimitiveType::Triangles,
        shader              : model_program.clone(),
        buffer_layouts      : vec! { vertex_layout.clone() },
        uniform_descs       : Vec::new(),
        index_type          : IndexType::None,
        face_winding        : FaceWinding::CCW,
        cull_mode           : CullMode::None,
        depth_write         : true,
        depth_test          : true,
        blend               : BlendOp::None,
        polygon_offset      : PolygonOffset::None,
    };

    driver.create_pipeline(tri_pipeline_desc).unwrap()

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
        .create_window(1024, 900, "Triangle", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_char_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.make_current();
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    let mut driver = renderer::get_driver();

    let vertices = vec! {
        Vertex { position: Vec4f::new(-0.5, -0.5, 0.0, 1.0), color: Vec4f::new(1.0, 0.0, 0.0, 1.0) },
        Vertex { position: Vec4f::new(0.5, -0.5,  0.0, 1.0), color: Vec4f::new(0.0, 0.0, 1.0, 1.0) },
        Vertex { position: Vec4f::new(0.0, 0.5,   0.0, 1.0), color: Vec4f::new(0.0, 1.0, 0.0, 1.0) },
    };

    let mut vertex_buffer   = driver.create_device_buffer(DeviceBufferDesc::Vertex(Usage::Dynamic(3 * std::mem::size_of::<Vertex>()))).unwrap();
    let pipeline = init_render_objects(&mut driver);

    let mut quit = false;
    while !window.should_close() {
        let (width, height) = window.get_framebuffer_size();

        let pass = Pass {
            frame_buffer: None,
            color_actions: [
                ColorPassAction::Clear(color4b(0x00, 0x00, 0x00, 0x00)),
                ColorPassAction::Previous,
                ColorPassAction::Previous,
                ColorPassAction::Previous,
            ],
            depth_action: DepthPassAction::Clear(1.0),
            width: width as usize,
            height: height as usize,
        };

        driver.begin_pass(&pass);
        let bindings = Bindings {
            vertex_buffers  : vec!{ vertex_buffer.clone() },
            index_buffer    : None,

            vertex_images   : Vec::new(),
            pixel_images    : Vec::new(),
        };

        driver.update_device_buffer(&mut vertex_buffer, 0, Arc::new(vertices.to_vec()));
        driver.draw(&pipeline, &bindings, std::ptr::null(), 1, 1);
        driver.end_pass();
        window.swap_buffers();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, _, _) |
                glfw::WindowEvent::Close  => quit = true,
                _ => ()
            }
        }

        if quit {
            window.set_should_close(true)
        }
    }
}