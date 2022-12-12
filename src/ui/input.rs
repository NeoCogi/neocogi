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
#![warn(clippy::all)]
#![allow(clippy::single_match)]

// Re-export dependencies.
use super::*;

pub struct InputState {
    pub pointer_pos: Vec2i,
    pub modifiers: KeyMod ,
    current_pixels_per_point: f32,
}

impl EguiInputState {
    pub fn new(egui_input: RawInput) -> Self {
        EguiInputState {
            pointer_pos: Pos2::new(0f32, 0f32),
            clipboard: init_clipboard(),
            egui_input,
            modifiers: Modifiers::default(),
        }
    }
}

pub fn handle_event(event: glfw::WindowEvent, state: &mut EguiInputState) {
    use glfw::WindowEvent::*;

    match event {
        ContentScale(pixels_per_point, _) => {
            state.egui_input.pixels_per_point = Some(pixels_per_point);
            state.current_pixels_per_point = pixels_per_point;
        },

        FramebufferSize(width, height) => {
            state.egui_input.screen_rect = Some(epaint::emath::Rect::from_min_size(
                Pos2::new(0f32, 0f32),
                egui::vec2(width as f32, height as f32) / state.current_pixels_per_point,
            ))
        }

        MouseButton (mouse_btn, glfw::Action::Press, _) => state.egui_input.events.push(egui::Event::PointerButton {
            pos: state.pointer_pos,
            button: match mouse_btn {
                glfw::MouseButtonLeft => egui::PointerButton::Primary,
                glfw::MouseButtonRight => egui::PointerButton::Secondary,
                glfw::MouseButtonMiddle => egui::PointerButton::Middle,
                _ => unreachable!(),
            },
            pressed: true,
            modifiers: state.modifiers,
        }),

        MouseButton (mouse_btn, glfw::Action::Release, _) => state.egui_input.events.push(egui::Event::PointerButton {
            pos: state.pointer_pos,
            button: match mouse_btn {
                glfw::MouseButtonLeft => egui::PointerButton::Primary,
                glfw::MouseButtonRight => egui::PointerButton::Secondary,
                glfw::MouseButtonMiddle => egui::PointerButton::Middle,
                _ => unreachable!(),
            },
            pressed: false,
            modifiers: state.modifiers,
        }),

        CursorPos(x, y) => {
            state.pointer_pos = pos2(
                x as f32 / state.current_pixels_per_point,
                y as f32 / state.current_pixels_per_point,
            );
            state
                .egui_input
                .events
                .push(egui::Event::PointerMoved(state.pointer_pos))
        }

        Key(keycode, _scancode, glfw::Action::Release, keymod) => {
            use glfw::Modifiers as Mod;
            if let Some(key) = translate_virtual_key_code(keycode) {
                state.modifiers = Modifiers {
                    alt: (keymod & Mod::Alt == Mod::Alt),
                    ctrl: (keymod & Mod::Control == Mod::Control),
                    shift: (keymod & Mod::Shift == Mod::Shift),

                    // TODO: GLFW doesn't seem to support the mac command key
                    // mac_cmd: keymod & Mod::LGUIMOD == Mod::LGUIMOD,
                    command: (keymod & Mod::Control == Mod::Control),

                    ..Default::default()
                };

                state.egui_input.events.push(Event::Key {
                    key,
                    pressed: false,
                    modifiers: state.modifiers,
                });
            }
        }

        Key(keycode, _scancode, glfw::Action::Press | glfw::Action::Repeat, keymod) => {
            use glfw::Modifiers as Mod;
            if let Some(key) = translate_virtual_key_code(keycode) {
                state.modifiers = Modifiers {
                    alt: (keymod & Mod::Alt == Mod::Alt),
                    ctrl: (keymod & Mod::Control == Mod::Control),
                    shift: (keymod & Mod::Shift == Mod::Shift),

                    // TODO: GLFW doesn't seem to support the mac command key
                    // mac_cmd: keymod & Mod::LGUIMOD == Mod::LGUIMOD,
                    command: (keymod & Mod::Control == Mod::Control),

                    ..Default::default()
                };

                if state.modifiers.command && key == egui::Key::X {
                    state.egui_input.events.push(egui::Event::Cut);
                } else if state.modifiers.command && key == egui::Key::C {
                    state.egui_input.events.push(egui::Event::Copy);
                } else if state.modifiers.command && key == egui::Key::V {
                    if let Some(clipboard_ctx) = state.clipboard.as_mut() {
                        state.egui_input.events.push(egui::Event::Text(clipboard_ctx.get_contents().unwrap_or("".to_string())));
                    }
                } else {
                    state.egui_input.events.push(Event::Key {
                        key,
                        pressed: true,
                        modifiers: state.modifiers,
                    });
                }
            }
        }

        Char(c) => {
            state.egui_input.events.push(Event::Text(c.to_string()));
        }

        Scroll (x, y) => {
            state.egui_input.events.push(egui::Event::Scroll(vec2(x as f32, y as f32)));
        }

        _ => {}
    }
}

pub fn translate_virtual_key_code(key: glfw::Key) -> Option<egui::Key> {

    Some(match key {
        glfw::Key::Left  => egui::Key::ArrowLeft,
        glfw::Key::Up    => egui::Key::ArrowUp,
        glfw::Key::Right => egui::Key::ArrowRight,
        glfw::Key::Down  => egui::Key::ArrowDown,

        glfw::Key::Escape    => egui::Key::Escape,
        glfw::Key::Tab       => egui::Key::Tab,
        glfw::Key::Backspace => egui::Key::Backspace,
        glfw::Key::Space     => egui::Key::Space,

        glfw::Key::Enter     => egui::Key::Enter,

        glfw::Key::Insert    => egui::Key::Insert,
        glfw::Key::Home      => egui::Key::Home,
        glfw::Key::Delete    => egui::Key::Delete,
        glfw::Key::End       => egui::Key::End,
        glfw::Key::PageDown  => egui::Key::PageDown,
        glfw::Key::PageUp    => egui::Key::PageUp,


        glfw::Key::A => egui::Key::A,
        glfw::Key::B => egui::Key::B,
        glfw::Key::C => egui::Key::C,
        glfw::Key::D => egui::Key::D,
        glfw::Key::E => egui::Key::E,
        glfw::Key::F => egui::Key::F,
        glfw::Key::G => egui::Key::G,
        glfw::Key::H => egui::Key::H,
        glfw::Key::I => egui::Key::I,
        glfw::Key::J => egui::Key::J,
        glfw::Key::K => egui::Key::K,
        glfw::Key::L => egui::Key::L,
        glfw::Key::M => egui::Key::M,
        glfw::Key::N => egui::Key::N,
        glfw::Key::O => egui::Key::O,
        glfw::Key::P => egui::Key::P,
        glfw::Key::Q => egui::Key::Q,
        glfw::Key::R => egui::Key::R,
        glfw::Key::S => egui::Key::S,
        glfw::Key::T => egui::Key::T,
        glfw::Key::U => egui::Key::U,
        glfw::Key::V => egui::Key::V,
        glfw::Key::W => egui::Key::W,
        glfw::Key::X => egui::Key::X,
        glfw::Key::Y => egui::Key::Y,
        glfw::Key::Z => egui::Key::Z,

        _ => {
            return None;
        }
    })
}

pub fn translate_cursor(cursor_icon: egui::CursorIcon) -> glfw::StandardCursor {
    match cursor_icon {
        CursorIcon::Default => glfw::StandardCursor::Arrow,
        CursorIcon::PointingHand => glfw::StandardCursor::Hand,
        CursorIcon::ResizeHorizontal => glfw::StandardCursor::HResize,
        CursorIcon::ResizeVertical => glfw::StandardCursor::VResize,
        // TODO: GLFW doesnt have these specific resize cursors, so we'll just use the HResize and VResize ones instead
        CursorIcon::ResizeNeSw => glfw::StandardCursor::HResize,
        CursorIcon::ResizeNwSe => glfw::StandardCursor::VResize,
        CursorIcon::Text => glfw::StandardCursor::IBeam,
        CursorIcon::Crosshair => glfw::StandardCursor::Crosshair,
        // TODO: Same for these
        CursorIcon::NotAllowed | CursorIcon::NoDrop => glfw::StandardCursor::Arrow,
        CursorIcon::Wait => glfw::StandardCursor::Arrow,
        CursorIcon::Grab | CursorIcon::Grabbing => glfw::StandardCursor::Hand,

        _ => glfw::StandardCursor::Arrow,
    }
}

pub fn init_clipboard() -> Option<ClipboardContext> {
    match ClipboardContext::new() {
        Ok(clipboard) => Some(clipboard),
        Err(err) => {
            eprintln!("Failed to initialize clipboard: {}", err);
            None
        }
    }
}

pub fn copy_to_clipboard(egui_state: &mut EguiInputState, copy_text: String) {
    if let Some(clipboard) = egui_state.clipboard.as_mut() {
        let result = clipboard.set_contents(copy_text);
        if result.is_err() {
            dbg!("Unable to set clipboard content.");
        }
    }
}
