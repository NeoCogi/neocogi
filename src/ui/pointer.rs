use rs_math3d::vector::FloatVector;

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
use crate::rs_math3d::*;

#[derive(Debug, Copy, Clone)]
pub enum ButtonState {
    None,
    Pressed(f32),
    Released,
    Scroll(f32),
}

#[derive(Debug, Copy, Clone)]
pub enum Event {
    None,
    Click(Vec2f, f32),
    Drag(Vec2f, f32, Vec2f, f32),
    Move(Vec2f, Vec2f),
    Scroll(f32),
}

#[derive(Debug, Copy, Clone)]
pub struct State {
    curr_pos    : Vec2f,
    curr_state  : ButtonState,

    prev_pos    : Vec2f,
    prev_state  : ButtonState,
}

impl State {
    pub fn new() -> Self {
        Self {
            curr_pos    : Vec2f::new(-1.0, -1.0),
            curr_state  : ButtonState::None,
            prev_pos    : Vec2f::new(-1.0, -1.0),
            prev_state  : ButtonState::None,
        }
    }

    pub fn event(&self) -> Event {
        let diff = (self.curr_pos - self.prev_pos).length();
        match (&self.prev_state, &self.curr_state, diff) {
            (ButtonState::Released,     ButtonState::Released, x) if x == 0.0 => Event::None,
            (ButtonState::Released,     ButtonState::Released, _)       => Event::Move(self.prev_pos, self.curr_pos),
            (ButtonState::Released,     ButtonState::Pressed(p), _)     => Event::Click(self.curr_pos, *p),
            (ButtonState::Pressed(pp),  ButtonState::Pressed(cp), _)    => Event::Drag(self.prev_pos, *pp, self.curr_pos, *cp),
            (ButtonState::Pressed(_),   ButtonState::Released, _)       => Event::None,
            (_,                         ButtonState::Scroll(s), _)      => Event::Scroll(*s),
            (_,                         _, _)                           => Event::None,
        }
    }

    pub fn update(&mut self, pos: Vec2f, st: ButtonState) {
        self.prev_pos   = self.curr_pos;
        self.prev_state = self.curr_state;
        self.curr_pos   = pos;
        self.curr_state = st;
    }

    pub fn reset_button_state(&mut self) {
        self.curr_state = ButtonState::None;
    }
}