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

#![warn(clippy::all)]
#![allow(clippy::single_match)]

use crate::ui;
// Re-export dependencies.
use super::*;

pub fn handle_event(event: glfw::WindowEvent, window: &mut glfw::Window, ctx: &mut ui::Context) {
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
        glfw::WindowEvent::Key(key, scancode, action, modifiers) => {
            let mut keymod = KeyModifier::NONE;
            if key == glfw::Key::Enter {
                keymod |= KeyModifier::RETURN
            }
            if modifiers == glfw::Modifiers::Alt {
                keymod |= KeyModifier::ALT
            } else if modifiers == glfw::Modifiers::Control {
                keymod |= KeyModifier::CTRL
            } else if modifiers == glfw::Modifiers::Shift {
                keymod |= KeyModifier::SHIFT
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
