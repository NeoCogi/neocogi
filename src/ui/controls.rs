use std::ops::Add;
use std::ptr::slice_from_raw_parts;
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
use super::*;

pub trait ControlProvider {
    fn text(&mut self, text: &str);
    fn label(&mut self, text: &str);
    fn button(&mut self, label: &str, icon: Option<usize>, opt: WidgetOption) -> ResourceState;
    fn checkbox(&mut self, label: &str, state: &mut bool) -> ResourceState;
    fn textbox_raw(
        &mut self,
        buf: &mut String,
        id: Id,
        r: Recti,
        opt: WidgetOption,
    ) -> ResourceState;
    fn textbox_ex(&mut self, buf: &mut String, opt: WidgetOption) -> ResourceState;

    fn slider_ex(
        &mut self,
        value: &mut Real,
        low: Real,
        high: Real,
        step: Real,
        precision: usize,
        opt: WidgetOption,
    ) -> ResourceState;

    fn number_ex(
        &mut self,
        value: &mut Real,
        step: Real,
        precision: usize,
        opt: WidgetOption,
    ) -> ResourceState;
}

impl<P: Default, R: RendererBackEnd<P>> Context<P, R> {
    fn number_textbox(
        &mut self,
        precision: usize,
        value: &mut Real,
        r: Recti,
        id: Id,
    ) -> ResourceState {
        if self.mouse_pressed.is_left() && self.key_down.is_shift() && self.hover == Some(id) {
            self.number_edit = Some(id);
            self.number_edit_buf.clear();
            self.number_edit_buf.append_real(precision, *value);
        }

        if self.number_edit == Some(id) {
            let mut temp = self.number_edit_buf.clone();
            let res: ResourceState = self.textbox_raw(&mut temp, id, r, WidgetOption::NONE);
            self.number_edit_buf = temp;
            if res.is_submitted() || self.focus != Some(id) {
                match Real::from_decimal(self.number_edit_buf.as_str()) {
                    Ok(v) => {
                        *value = v as Real;
                        self.number_edit = None;
                    }
                    _ => (),
                }
            } else {
                return ResourceState::ACTIVE;
            }
        }
        return ResourceState::NONE;
    }
}

impl<P: Default, R: RendererBackEnd<P>> ControlProvider for Context<P, R> {
    fn text(&mut self, text: &str) {
        let font = self.style.normal_font;
        let color = self.style.colors[ControlColor::Text as usize];
        let style = self.style;
        self.column(|ctx| {
            let h = ctx.renderer.get_font_height(font) as i32;
            ctx.rows_with_line_config(&[-1], h, |ctx| {
                let mut r = ctx.layout_stack.next_cell(&ctx.style);
                for line in text.lines() {
                    let mut rx = r.x;
                    let words = line.split_inclusive(' ');
                    for w in words {
                        // TODO: split w when its width > w into many lines
                        let tw = ctx.get_text_width(font, w);
                        if tw + rx < r.x + r.width {
                            ctx.draw_text(font, w, vec2(rx, r.y), color);
                            rx += tw;
                        } else {
                            r = ctx.layout_stack.next_cell(&ctx.style);
                            rx = r.x;
                        }
                    }
                    r = ctx.layout_stack.next_cell(&ctx.style);
                }
            });
        });
    }

    fn label(&mut self, text: &str) {
        let layout = self.layout_stack.next_cell(&self.style);
        self.draw_control_text(
            self.style.normal_font,
            text,
            layout,
            ControlColor::Text,
            WidgetOption::NONE,
        );
    }

    fn button(&mut self, label: &str, icon: Option<usize>, opt: WidgetOption) -> ResourceState {
        let mut res = ResourceState::NONE;
        let id: Id = if label.len() > 0 {
            self.get_id_from_str(label)
        } else {
            self.get_id_u32(icon.unwrap() as u32)
        };
        let r = self.layout_stack.next_cell(&self.style);
        self.update_control(id, r, opt);
        if self.mouse_pressed.is_left() && self.focus == Some(id) {
            res |= ResourceState::SUBMIT;
        }
        self.draw_control_frame(id, r, ControlColor::Button, opt);
        if label.len() > 0 {
            self.draw_control_text(self.style.normal_font, label, r, ControlColor::Text, opt);
        }
        if icon.is_some() {
            self.draw_icon(
                icon.unwrap(),
                r,
                self.style.colors[ControlColor::Text as usize],
            );
        }
        return res;
    }

    fn checkbox(&mut self, label: &str, state: &mut bool) -> ResourceState {
        let mut res = ResourceState::NONE;
        let id: Id = self.get_id_from_ptr(state);
        let mut r = self.layout_stack.next_cell(&self.style);
        let box_0 = Rect::new(r.x, r.y, r.height, r.height);
        self.update_control(id, r, WidgetOption::NONE);
        if self.mouse_pressed.is_left() && self.focus == Some(id) {
            res |= ResourceState::CHANGE;
            *state = *state == false;
        }
        self.draw_control_frame(id, box_0, ControlColor::Base, WidgetOption::NONE);
        if *state {
            self.draw_icon(CHECK, box_0, self.style.colors[ControlColor::Text as usize]);
        }
        r = Rect::new(r.x + box_0.width, r.y, r.width - box_0.width, r.height);
        self.draw_control_text(
            self.style.normal_font,
            label,
            r,
            ControlColor::Text,
            WidgetOption::NONE,
        );
        return res;
    }

    fn textbox_raw(
        &mut self,
        buf: &mut String,
        id: Id,
        r: Recti,
        opt: WidgetOption,
    ) -> ResourceState {
        let mut res = ResourceState::NONE;
        self.update_control(id, r, opt | WidgetOption::HOLD_FOCUS);
        if self.focus == Some(id) {
            let mut len = buf.len();

            if self.input_text.len() > 0 {
                buf.push_str(self.input_text.as_str());
                len += self.input_text.len() as usize;
                res |= ResourceState::CHANGE
            }

            if self.key_pressed.is_backspace() && len > 0 {
                // skip utf-8 continuation bytes
                buf.pop();
                res |= ResourceState::CHANGE
            }
            if self.key_pressed.is_return() {
                self.set_focus(None);
                res |= ResourceState::SUBMIT;
            }
        }
        self.draw_control_frame(id, r, ControlColor::Base, opt);
        if self.focus == Some(id) {
            let color = self.style.colors[ControlColor::Text as usize];
            let font = self.style.normal_font;
            let textw = self.get_text_width(font, buf.as_str());
            let texth = self.get_text_height(font, buf.as_str());
            let ofx = r.width - self.style.padding - textw - 1;
            let textx = r.x
                + (if ofx < self.style.padding {
                    ofx
                } else {
                    self.style.padding
                });
            let texty = r.y + (r.height - texth) / 2;
            self.push_clip_rect(r);
            self.draw_text(font, buf.as_str(), vec2(textx, texty), color);
            self.draw_rect(Rect::new(textx + textw, texty, 1, texth), color);
            self.pop_clip_rect();
        } else {
            self.draw_control_text(
                self.style.normal_font,
                buf.as_str(),
                r,
                ControlColor::Text,
                opt,
            );
        }
        return res;
    }

    fn textbox_ex(&mut self, buf: &mut String, opt: WidgetOption) -> ResourceState {
        let id = self.get_id_from_ptr(buf);
        let r = self.layout_stack.next_cell(&self.style);
        return self.textbox_raw(buf, id, r, opt);
    }

    fn slider_ex(
        &mut self,
        value: &mut Real,
        low: Real,
        high: Real,
        step: Real,
        precision: usize,
        opt: WidgetOption,
    ) -> ResourceState {
        let mut res = ResourceState::NONE;
        let last = *value;
        let mut v = last;
        let id = self.get_id_from_ptr(value);
        let base = self.layout_stack.next_cell(&self.style);
        if !self.number_textbox(precision, &mut v, base, id).is_none() {
            return res;
        }
        self.update_control(id, base, opt);
        if self.focus == Some(id) && (!self.mouse_down.is_none() | self.mouse_pressed.is_left()) {
            v = low + (self.mouse_pos.x - base.x) as Real * (high - low) / base.width as Real;
            if step != 0. {
                v = (v + step / 2 as Real) / step * step;
            }
        }
        v = if high < (if low > v { low } else { v }) {
            high
        } else if low > v {
            low
        } else {
            v
        };
        *value = v;
        if last != v {
            res |= ResourceState::CHANGE;
        }
        self.draw_control_frame(id, base, ControlColor::Base, opt);
        let w = self.style.thumb_size;
        let x = ((v - low) * (base.width - w) as Real / (high - low)) as i32;
        let thumb = Rect::new(base.x + x, base.y, w, base.height);
        self.draw_control_frame(id, thumb, ControlColor::Button, opt);
        self.slider_buff.clear();
        self.slider_buff.append_real(precision, *value);
        let txt_ptr = self.slider_buff.as_str().as_ptr();
        let txt_slice = slice_from_raw_parts(txt_ptr, self.slider_buff.as_str().len());
        let txt = unsafe { std::str::from_utf8(&*txt_slice) }.unwrap();
        self.draw_control_text(self.style.normal_font, txt, base, ControlColor::Text, opt);
        return res;
    }

    fn number_ex(
        &mut self,
        value: &mut Real,
        step: Real,
        precision: usize,
        opt: WidgetOption,
    ) -> ResourceState {
        let mut res = ResourceState::NONE;
        let id: Id = self.get_id_from_ptr(value);
        let base = self.layout_stack.next_cell(&self.style);
        let last: Real = *value;
        if !self.number_textbox(precision, value, base, id).is_none() {
            return res;
        }
        self.update_control(id, base, opt);
        if self.focus == Some(id) && self.mouse_down.is_left() {
            *value += self.mouse_delta.x as Real * step;
        }
        if *value != last {
            res |= ResourceState::CHANGE;
        }
        self.draw_control_frame(id, base, ControlColor::Base, opt);
        self.slider_buff.clear();
        self.slider_buff.append_real(precision, *value);
        let txt_ptr = self.slider_buff.as_str().as_ptr();
        let txt_slice = slice_from_raw_parts(txt_ptr, self.slider_buff.as_str().len());
        let txt = unsafe { std::str::from_utf8(&*txt_slice) }.unwrap();
        self.draw_control_text(self.style.normal_font, txt, base, ControlColor::Text, opt);
        return res;
    }
}
