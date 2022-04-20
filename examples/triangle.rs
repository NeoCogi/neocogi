extern crate neocogi;

use neocogi::glfw;
use neocogi::glfw::{Action, Context, Key};

use neocogi::*;
use neocogi::rs_math3d::*;

use neocogi::renderer::*;

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

    let mut quit = false;
    while !window.should_close() {
        let (width, height) = window.get_framebuffer_size();

        let pass = Pass {
            frame_buffer: None,
            color_actions: [
                ColorPassAction::Clear(color4b(0x7F, 0x7F, 0x7F, 0xFF)),
                ColorPassAction::Previous,
                ColorPassAction::Previous,
                ColorPassAction::Previous,
            ],
            depth_action: DepthPassAction::Clear(1.0),
            width: width as usize,
            height: height as usize,
        };

        driver.begin_pass(&pass);
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