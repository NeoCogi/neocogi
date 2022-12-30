//
// Copyright 2022-Present (c) Raja Lehtihet & Wael El Oraiby
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
// -----------------------------------------------------------------------------
// Ported to rust from https://github.com/rxi/microui/ and the original license
//
// Copyright (c) 2020 rxi
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to
// deal in the Software without restriction, including without limitation the
// rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
// sell copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
// IN THE SOFTWARE.
//
use core::ptr;
use std::marker::PhantomData;

mod fixed_collections;
pub use fixed_collections::*;

mod atlas;
pub use atlas::*;

mod layout;
use layout::*;

pub mod system;

mod controls;
pub use controls::*;

pub use system::*;

use rs_math3d::{color4b, Color4b, Rect, Recti, Vec2i};

use bitflags::*;

pub trait RendererBackEnd<P> {
    fn get_char_width(&self, _font: FontId, c: char) -> usize;
    fn get_font_height(&self, _font: FontId) -> usize;

    fn begin_frame(&mut self, width: usize, height: usize);
    fn end_frame(&mut self) -> P;

    fn draw_rect(&mut self, rect: Recti, color: Color4b);
    fn draw_text(&mut self, text: &str, pos: Vec2i, color: Color4b);
    fn draw_icon(&mut self, id: Icon, r: Recti, color: Color4b);
    fn set_clip_rect(&mut self, rect: Recti);

    fn flush(&mut self);

    fn frame_size(&self) -> (usize, usize);
}

#[derive(Copy, Clone)]
pub struct Pool<const N: usize> {
    vec: [PoolItem; N],
}

impl<const N: usize> Pool<N> {
    pub fn alloc(&mut self, id: Id, frame: usize) -> usize {
        let mut res = None;
        let mut latest_update = frame;
        for i in 0..N {
            if self.vec[i].last_update < latest_update {
                latest_update = self.vec[i].last_update;
                res = Some(i);
            }
        }

        assert!(res.is_some());
        self.vec[res.unwrap()].id = id;
        self.update(res.unwrap(), frame);
        return res.unwrap();
    }

    pub fn get(&self, id: Id) -> Option<usize> {
        for i in 0..N {
            if self.vec[i].id == id {
                return Some(i);
            }
        }
        None
    }

    pub fn update(&mut self, idx: usize, frame: usize) {
        self.vec[idx].last_update = frame;
    }

    pub fn reset(&mut self, idx: usize) {
        self.vec[idx] = PoolItem::default();
    }
}

impl<const N: usize> Default for Pool<N> {
    fn default() -> Self {
        Self {
            vec: [PoolItem::default(); N],
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
#[repr(u32)]
pub enum Clip {
    None = 0,
    Part = 1,
    All = 2,
}

#[derive(PartialEq, Copy, Clone)]
#[repr(u32)]
pub enum ControlColor {
    Max = 14,
    ScrollThumb = 13,
    ScrollBase = 12,
    BaseFocus = 11,
    BaseHover = 10,
    Base = 9,
    ButtonFocus = 8,
    ButtonHover = 7,
    Button = 6,
    PanelBG = 5,
    TitleText = 4,
    TitleBG = 3,
    WindowBG = 2,
    Border = 1,
    Text = 0,
}

impl ControlColor {
    pub fn hover(&mut self) {
        *self = match self {
            Self::Base => Self::BaseHover,
            Self::Button => Self::ButtonHover,
            _ => *self,
        }
    }

    pub fn focus(&mut self) {
        *self = match self {
            Self::Base => Self::BaseFocus,
            Self::Button => Self::ButtonFocus,
            Self::BaseHover => Self::BaseFocus,
            Self::ButtonHover => Self::ButtonFocus,
            _ => *self,
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
#[repr(u32)]
pub enum Icon {
    Max = 5,
    Expanded = 4,
    Collapsed = 3,
    Check = 2,
    Close = 1,
    None = 0,
}

bitflags! {
    pub struct ResourceState : u32 {
        const CHANGE = 4;
        const SUBMIT = 2;
        const ACTIVE = 1;
        const NONE = 0;
    }
}

impl ResourceState {
    pub fn is_changed(&self) -> bool {
        self.intersects(Self::CHANGE)
    }
    pub fn is_submitted(&self) -> bool {
        self.intersects(Self::SUBMIT)
    }
    pub fn is_active(&self) -> bool {
        self.intersects(Self::ACTIVE)
    }
    pub fn is_none(&self) -> bool {
        self.bits == 0
    }
}

bitflags! {
    pub struct WidgetOption : u32 {
        const EXPANDED = 4096;
        const CLOSED = 2048;
        const POPUP= 1024;
        const AUTO_SIZE = 512;
        const HOLD_FOCUS = 256;
        const NO_TITLE = 128;
        const NO_CLOSE = 64;
        const NO_SCROLL = 32;
        const NO_RESIZE = 16;
        const NO_FRAME = 8;
        const NO_INTERACT = 4;
        const ALIGN_RIGHT = 2;
        const ALIGN_CENTER = 1;
        const NONE = 0;
    }
}

impl WidgetOption {
    pub fn is_expanded(&self) -> bool {
        self.intersects(WidgetOption::EXPANDED)
    }
    pub fn is_closed(&self) -> bool {
        self.intersects(WidgetOption::CLOSED)
    }
    pub fn is_popup(&self) -> bool {
        self.intersects(WidgetOption::POPUP)
    }
    pub fn is_auto_sizing(&self) -> bool {
        self.intersects(WidgetOption::AUTO_SIZE)
    }
    pub fn is_holding_focus(&self) -> bool {
        self.intersects(WidgetOption::HOLD_FOCUS)
    }
    pub fn has_no_title(&self) -> bool {
        self.intersects(WidgetOption::NO_TITLE)
    }
    pub fn has_no_close(&self) -> bool {
        self.intersects(WidgetOption::NO_CLOSE)
    }
    pub fn has_no_scroll(&self) -> bool {
        self.intersects(WidgetOption::NO_SCROLL)
    }
    pub fn is_fixed(&self) -> bool {
        self.intersects(WidgetOption::NO_RESIZE)
    }
    pub fn has_no_frame(&self) -> bool {
        self.intersects(WidgetOption::NO_FRAME)
    }
    pub fn is_not_interactive(&self) -> bool {
        self.intersects(WidgetOption::NO_INTERACT)
    }
    pub fn is_aligned_right(&self) -> bool {
        self.intersects(WidgetOption::ALIGN_RIGHT)
    }
    pub fn is_aligned_center(&self) -> bool {
        self.intersects(WidgetOption::ALIGN_CENTER)
    }
    pub fn is_none(&self) -> bool {
        self.bits == 0
    }
}

bitflags! {
    pub struct MouseButton : u32 {
        const MIDDLE = 4;
        const RIGHT = 2;
        const LEFT = 1;
        const NONE = 0;
    }
}

impl MouseButton {
    pub fn is_middle(&self) -> bool {
        self.intersects(Self::MIDDLE)
    }
    pub fn is_right(&self) -> bool {
        self.intersects(Self::RIGHT)
    }
    pub fn is_left(&self) -> bool {
        self.intersects(Self::LEFT)
    }
    pub fn is_none(&self) -> bool {
        self.bits == 0
    }
}

bitflags! {
    pub struct KeyModifier : u32 {
        const RETURN = 16;
        const BACKSPACE = 8;
        const ALT = 4;
        const CTRL = 2;
        const SHIFT = 1;
        const NONE = 0;
    }
}

impl KeyModifier {
    pub fn is_none(&self) -> bool {
        self.bits == 0
    }
    pub fn is_return(&self) -> bool {
        self.intersects(Self::RETURN)
    }
    pub fn is_backspace(&self) -> bool {
        self.intersects(Self::BACKSPACE)
    }
    pub fn is_alt(&self) -> bool {
        self.intersects(Self::ALT)
    }
    pub fn is_ctrl(&self) -> bool {
        self.intersects(Self::CTRL)
    }
    pub fn is_shift(&self) -> bool {
        self.intersects(Self::SHIFT)
    }
}

#[repr(C)]
pub struct Context<P, R: RendererBackEnd<P>> {
    pub style: Style,
    pub hover: Option<Id>,
    pub focus: Option<Id>,
    pub last_id: Option<Id>,
    pub last_zindex: i32,
    pub updated_focus: bool,
    pub frame: usize,
    pub hover_root: Option<usize>,
    pub next_hover_root: Option<usize>,
    pub scroll_target: Option<usize>,
    pub number_edit_buf: FixedString<127>,
    pub number_edit: Option<Id>,
    pub command_list: FixedVec<Command, 4096>,
    pub root_list: FixedVec<usize, 32>,
    pub container_stack: FixedVec<usize, 32>,
    pub clip_stack: FixedVec<Recti, 32>,
    pub id_stack: FixedVec<Id, 32>,
    layout_stack: LayoutStack,
    pub text_stack: FixedString<65536>,
    pub container_pool: Pool<48>,
    pub containers: [Container; 48],
    pub treenode_pool: Pool<48>,
    pub mouse_pos: Vec2i,
    pub last_mouse_pos: Vec2i,
    pub mouse_delta: Vec2i,
    pub scroll_delta: Vec2i,
    pub mouse_down: MouseButton,
    pub mouse_pressed: MouseButton,
    pub key_down: KeyModifier,
    pub key_pressed: KeyModifier,
    pub input_text: FixedString<32>,
    renderer: R,
    _unused: PhantomData<P>,
}

#[derive(Default, Copy, Clone)]
struct PoolItem {
    pub id: Id,
    pub last_update: usize,
}

#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub struct Id(u32);

#[derive(Default, Copy, Clone)]
pub struct Container {
    pub head_idx: Option<usize>,
    pub tail_idx: Option<usize>,
    pub rect: Recti,
    pub body: Recti,
    pub content_size: Vec2i,
    pub scroll: Vec2i,
    pub zindex: i32,
    pub open: bool,
}

#[derive(Copy, Clone)]
pub enum Command {
    Jump {
        dst_idx: Option<usize>,
    },
    Clip {
        rect: Recti,
    },
    Rect {
        rect: Recti,
        color: Color4b,
    },
    Text {
        font: FontId,
        pos: Vec2i,
        color: Color4b,
        str_start: usize,
        str_len: usize,
    },
    Icon {
        rect: Recti,
        id: Icon,
        color: Color4b,
    },
    Custom {
        rect: Recti,
        render_fn_id: usize,
    },
    None,
}

impl Default for Command {
    fn default() -> Self {
        Command::None
    }
}

pub trait Font {
    fn name(&self) -> &str;
    fn get_size(&self) -> usize;
    fn get_char_size(&self, c: char) -> (usize, usize);
}

#[derive(Copy, Clone)]
pub struct FontId(pub usize);

#[derive(Copy, Clone)]
pub struct Style {
    pub font: FontId,
    pub size: Vec2i,
    pub padding: i32,
    pub spacing: i32,
    pub indent: i32,
    pub title_height: i32,
    pub scrollbar_size: i32,
    pub thumb_size: i32,
    pub colors: [Color4b; 14],
}

pub type Real = f32;

static UNCLIPPED_RECT: Recti = Recti {
    x: 0,
    y: 0,
    width: 0x1000000,
    height: 0x1000000,
};

impl Default for Style {
    fn default() -> Self {
        Self {
            font: FontId(0),
            size: Vec2i { x: 68, y: 10 },
            padding: 5,
            spacing: 4,
            indent: 24,
            title_height: 24,
            scrollbar_size: 12,
            thumb_size: 8,
            colors: [
                color4b(230, 230, 230, 255),
                color4b(25, 25, 25, 255),
                color4b(50, 50, 50, 255),
                color4b(25, 25, 25, 255),
                color4b(240, 240, 240, 255),
                color4b(0, 0, 0, 0),
                color4b(75, 75, 75, 255),
                color4b(95, 95, 95, 255),
                color4b(115, 115, 115, 255),
                color4b(30, 30, 30, 255),
                color4b(35, 35, 35, 255),
                color4b(40, 40, 40, 255),
                color4b(43, 43, 43, 255),
                color4b(30, 30, 30, 255),
            ],
        }
    }
}

pub fn vec2(x: i32, y: i32) -> Vec2i {
    Vec2i { x, y }
}

pub fn color(r: u8, g: u8, b: u8, a: u8) -> Color4b {
    color4b(r, g, b, a)
}

pub fn expand_rect(r: Recti, n: i32) -> Recti {
    Rect::new(
        r.x - n,
        r.y - n,
        i32::max(0, r.width + n * 2),
        i32::max(0, r.height + n * 2),
    )
}

pub fn intersect_rects(r1: Recti, r2: Recti) -> Recti {
    let x1 = i32::max(r1.x, r2.x);
    let y1 = i32::max(r1.y, r2.y);
    let mut x2 = i32::min(r1.x + r1.width, r2.x + r2.width);
    let mut y2 = i32::min(r1.y + r1.height, r2.y + r2.height);
    if x2 < x1 {
        x2 = x1;
    }
    if y2 < y1 {
        y2 = y1;
    }
    return Rect::new(x1, y1, x2 - x1, y2 - y1);
}

pub fn rect_overlaps_vec2(r: Recti, p: Vec2i) -> bool {
    p.x >= r.x && p.x < r.x + r.width && p.y >= r.y && p.y < r.y + r.height
}

fn hash_step(h: u32, n: u32) -> u32 {
    (h ^ n).wrapping_mul(16777619 as u32)
}

fn hash_u32(hash_0: &mut Id, orig_id: u32) {
    let bytes = orig_id.to_be_bytes();
    for b in bytes {
        *hash_0 = Id(hash_step(hash_0.0, b as u32));
    }
}

fn hash_str(hash_0: &mut Id, s: &str) {
    for c in s.chars() {
        *hash_0 = Id(hash_step(hash_0.0, c as u32));
    }
}

fn hash_bytes(hash_0: &mut Id, s: &[u8]) {
    for c in s {
        *hash_0 = Id(hash_step(hash_0.0, *c as u32));
    }
}

impl<P, R: RendererBackEnd<P>> Context<P, R> {
    pub fn new(renderer: R) -> Self {
        Self {
            style: Style::default(),
            hover: None,
            focus: None,
            last_id: None,
            last_zindex: 0,
            updated_focus: false,
            frame: 0,
            hover_root: None,
            next_hover_root: None,
            scroll_target: None,
            number_edit_buf: FixedString::default(),
            number_edit: None,
            command_list: FixedVec::default(),
            root_list: FixedVec::default(),
            container_stack: FixedVec::default(),
            clip_stack: FixedVec::default(),
            id_stack: FixedVec::default(),
            layout_stack: LayoutStack::default(),
            text_stack: FixedString::default(),
            container_pool: Pool::default(),
            containers: [Container::default(); 48],
            treenode_pool: Pool::default(),
            mouse_pos: Vec2i::default(),
            last_mouse_pos: Vec2i::default(),
            mouse_delta: Vec2i::default(),
            scroll_delta: Vec2i::default(),
            mouse_down: MouseButton::NONE,
            mouse_pressed: MouseButton::NONE,
            key_down: KeyModifier::NONE,
            key_pressed: KeyModifier::NONE,
            input_text: FixedString::default(),
            renderer,
            _unused: PhantomData::default(),
        }
    }

    fn draw_frame(&mut self, rect: Recti, colorid: ControlColor) {
        self.draw_rect(rect, self.style.colors[colorid as usize]);
        if colorid == ControlColor::ScrollBase
            || colorid == ControlColor::ScrollThumb
            || colorid == ControlColor::TitleBG
        {
            return;
        }
        if self.style.colors[ControlColor::Border as usize].w != 0 {
            // alpha
            self.draw_box(
                expand_rect(rect, 1),
                self.style.colors[ControlColor::Border as usize],
            );
        }
    }

    fn begin(&mut self, width: usize, height: usize) {
        self.root_list.clear();
        self.text_stack.clear();
        self.scroll_target = None;
        self.hover_root = self.next_hover_root;
        self.next_hover_root = None;
        self.mouse_delta.x = self.mouse_pos.x - self.last_mouse_pos.x;
        self.mouse_delta.y = self.mouse_pos.y - self.last_mouse_pos.y;
        self.command_list.clear();
        self.frame += 1;
        self.renderer.begin_frame(width, height);
    }

    fn end(&mut self) -> P {
        assert_eq!(self.container_stack.len(), 0);
        assert_eq!(self.clip_stack.len(), 0);
        assert_eq!(self.id_stack.len(), 0);
        assert_eq!(self.layout_stack.len(), 0);
        if !self.scroll_target.is_none() {
            self.containers[self.scroll_target.unwrap()].scroll.x += self.scroll_delta.x;
            self.containers[self.scroll_target.unwrap()].scroll.y += self.scroll_delta.y;
        }
        if !self.updated_focus {
            self.focus = None;
        }
        self.updated_focus = false;
        if !self.mouse_pressed.is_none()
            && !self.next_hover_root.is_none()
            && self.containers[self.next_hover_root.unwrap()].zindex < self.last_zindex
            && self.containers[self.next_hover_root.unwrap()].zindex >= 0
        {
            self.bring_to_front(self.next_hover_root.unwrap());
        }
        self.key_pressed = KeyModifier::NONE;
        self.input_text.clear();
        self.mouse_pressed = MouseButton::NONE;
        self.scroll_delta = vec2(0, 0);
        self.last_mouse_pos = self.mouse_pos;
        let n = self.root_list.len();
        let containers = &self.containers;
        quick_sort_by(self.root_list.as_slice_mut(), |a, b| {
            containers[*a].zindex.cmp(&containers[*b].zindex)
        });

        for i in 0..n {
            if i == 0 {
                // root container!
                // if this is the first container then make the first command jump to it.
                // otherwise set the previous container's tail to jump to this one

                let cmd = &mut self.command_list[0];
                assert!(match cmd {
                    Command::Jump { .. } => true,
                    _ => false,
                });
                let dst_idx = self.containers[self.root_list[i as usize]]
                    .head_idx
                    .unwrap()
                    + 1;
                *cmd = Command::Jump {
                    dst_idx: Some(dst_idx),
                };
                assert!(dst_idx < self.command_list.len());
            } else {
                let prev = &self.containers[self.root_list[i - 1]];
                self.command_list[prev.tail_idx.unwrap()] = Command::Jump {
                    dst_idx: Some(
                        self.containers[self.root_list[i as usize]]
                            .head_idx
                            .unwrap()
                            + 1,
                    ),
                };
            }
            if i == n - 1 {
                assert!(
                    self.containers[self.root_list[i as usize]]
                        .tail_idx
                        .unwrap()
                        < self.command_list.len()
                );
                assert!(
                    match self.command_list[self.containers[self.root_list[i as usize]]
                        .tail_idx
                        .unwrap()]
                    {
                        Command::Jump { .. } => true,
                        _ => false,
                    }
                );
                self.command_list[self.containers[self.root_list[i as usize]]
                    .tail_idx
                    .unwrap()] = Command::Jump {
                    dst_idx: Some(self.command_list.len()),
                };
                // the snake eats its tail
            }
        }

        self.paint();
        self.renderer.end_frame()
    }

    pub fn set_focus(&mut self, id: Option<Id>) {
        self.focus = id;
        self.updated_focus = true;
    }

    pub fn get_id_u32(&mut self, orig_id: u32) -> Id {
        let mut res: Id = match self.id_stack.top() {
            Some(id) => *id,
            None => Id(2166136261),
        };
        hash_u32(&mut res, orig_id);
        self.last_id = Some(res);
        return res;
    }

    pub fn get_id_from_ptr<T: ?Sized>(&mut self, orig_id: &T) -> Id {
        let mut res: Id = match self.id_stack.top() {
            Some(id) => *id,
            None => Id(2166136261),
        };
        let ptr = orig_id as *const T as *const u8 as usize;
        let bytes = ptr.to_le_bytes();
        hash_bytes(&mut res, &bytes);
        self.last_id = Some(res);
        return res;
    }

    pub fn get_id_from_str(&mut self, s: &str) -> Id {
        let mut res: Id = match self.id_stack.top() {
            Some(id) => *id,
            None => Id(2166136261),
        };
        hash_str(&mut res, s);
        self.last_id = Some(res);
        return res;
    }

    pub fn push_id_from_ptr<T>(&mut self, orig_id: &T) {
        let id = self.get_id_from_ptr(orig_id);
        self.id_stack.push(id);
    }

    pub fn push_id_from_str(&mut self, s: &str) {
        let id = self.get_id_from_str(s);
        self.id_stack.push(id);
    }

    pub fn pop_id(&mut self) {
        self.id_stack.pop();
    }

    pub fn push_clip_rect(&mut self, rect: Recti) {
        let last = self.get_clip_rect();
        self.clip_stack.push(intersect_rects(rect, last));
    }

    pub fn pop_clip_rect(&mut self) {
        self.clip_stack.pop();
    }

    pub fn get_clip_rect(&mut self) -> Recti {
        *self.clip_stack.top().unwrap()
    }

    pub fn check_clip(&mut self, r: Recti) -> Clip {
        let cr = self.get_clip_rect();
        if r.x > cr.x + cr.width
            || r.x + r.width < cr.x
            || r.y > cr.y + cr.height
            || r.y + r.height < cr.y
        {
            return Clip::All;
        }
        if r.x >= cr.x
            && r.x + r.width <= cr.x + cr.width
            && r.y >= cr.y
            && r.y + r.height <= cr.y + cr.height
        {
            return Clip::None;
        }
        return Clip::Part;
    }

    fn pop_container(&mut self) {
        let cnt = self.get_current_container();
        let layout = *self.layout_stack.top();

        self.containers[cnt].content_size.x = layout.max.x - layout.body.x;
        self.containers[cnt].content_size.y = layout.max.y - layout.body.y;

        self.container_stack.pop();
        self.layout_stack.pop();
        self.pop_id();
    }

    pub fn get_current_container(&self) -> usize {
        *self.container_stack.top().unwrap()
    }

    pub fn get_current_container_rect(&self) -> Recti {
        self.containers[*self.container_stack.top().unwrap()].rect
    }

    pub fn set_current_container_rect(&mut self, rect: &Recti) {
        self.containers[*self.container_stack.top().unwrap()].rect = *rect;
    }

    pub fn get_current_container_scroll(&self) -> Vec2i {
        self.containers[*self.container_stack.top().unwrap()].scroll
    }

    pub fn set_current_container_scroll(&mut self, scroll: &Vec2i) {
        self.containers[*self.container_stack.top().unwrap()].scroll = *scroll;
    }

    pub fn get_current_container_content_size(&self) -> Vec2i {
        self.containers[*self.container_stack.top().unwrap()].content_size
    }

    pub fn get_current_container_body(&self) -> Recti {
        self.containers[*self.container_stack.top().unwrap()].body
    }

    fn get_container_index_intern(&mut self, id: Id, opt: WidgetOption) -> Option<usize> {
        let idx = self.container_pool.get(id);
        if idx.is_some() {
            if self.containers[idx.unwrap()].open || !opt.is_closed() {
                self.container_pool.update(idx.unwrap(), self.frame);
            }
            return idx;
        }
        if opt.is_closed() {
            return None;
        }
        let idx = self.container_pool.alloc(id, self.frame);
        self.containers[idx] = Container::default();
        self.containers[idx].head_idx = None;
        self.containers[idx].tail_idx = None;
        self.containers[idx].open = true;
        self.bring_to_front(idx);
        Some(idx)
    }

    fn get_container_index(&mut self, name: &str) -> Option<usize> {
        let id = self.get_id_from_str(name);
        self.get_container_index_intern(id, WidgetOption::NONE)
    }

    pub fn bring_to_front(&mut self, cnt: usize) {
        self.last_zindex += 1;
        self.containers[cnt].zindex = self.last_zindex;
    }

    pub fn input_mousemove(&mut self, x: i32, y: i32) {
        self.mouse_pos = vec2(x, y);
    }

    pub fn input_mousedown(&mut self, x: i32, y: i32, btn: MouseButton) {
        self.input_mousemove(x, y);
        self.mouse_down |= btn;
        self.mouse_pressed |= btn;
    }

    pub fn input_mouseup(&mut self, x: i32, y: i32, btn: MouseButton) {
        self.input_mousemove(x, y);
        self.mouse_down &= !btn;
    }

    pub fn input_scroll(&mut self, x: i32, y: i32) {
        self.scroll_delta.x += x;
        self.scroll_delta.y += y * (self.renderer.get_font_height(FontId(0)) as i32);
    }

    pub fn input_keydown(&mut self, key: KeyModifier) {
        self.key_pressed |= key;
        self.key_down |= key;
    }

    pub fn input_keyup(&mut self, key: KeyModifier) {
        self.key_down &= !key;
    }

    pub fn input_text(&mut self, text: &str) {
        self.input_text += text;
    }

    pub fn push_command(&mut self, cmd: Command) -> (&mut Command, usize) {
        self.command_list.push(cmd)
    }

    pub fn push_text(&mut self, str: &str) -> usize {
        let str_start = self.text_stack.len();
        for c in str.chars() {
            self.text_stack.push(c);
        }
        return str_start;
    }

    ///
    /// returns the next command to execute and the next index to use
    ///
    pub fn mu_next_command(&mut self, mut cmd_id: usize) -> Option<(Command, usize)> {
        if cmd_id >= self.command_list.len() {
            cmd_id = 0
        }

        while cmd_id != self.command_list.len() {
            match self.command_list[cmd_id] {
                Command::Jump { dst_idx } => cmd_id = dst_idx.unwrap(),
                _ => return Some((self.command_list[cmd_id], cmd_id + 1)),
            }
        }
        None
    }

    fn push_jump(&mut self, dst_idx: Option<usize>) -> usize {
        let (_, pos) = self.push_command(Command::Jump { dst_idx });
        pos
    }

    pub fn set_clip(&mut self, rect: Recti) {
        self.push_command(Command::Clip { rect });
    }

    pub fn draw_rect(&mut self, mut rect: Recti, color: Color4b) {
        rect = intersect_rects(rect, self.get_clip_rect());
        if rect.width > 0 && rect.height > 0 {
            self.push_command(Command::Rect { rect, color });
        }
    }

    pub fn draw_box(&mut self, r: Recti, color: Color4b) {
        self.draw_rect(Rect::new(r.x + 1, r.y, r.width - 2, 1), color);
        self.draw_rect(
            Rect::new(r.x + 1, r.y + r.height - 1, r.width - 2, 1),
            color,
        );
        self.draw_rect(Rect::new(r.x, r.y, 1, r.height), color);
        self.draw_rect(Rect::new(r.x + r.width - 1, r.y, 1, r.height), color);
    }

    pub fn draw_text(&mut self, font: FontId, str: &str, pos: Vec2i, color: Color4b) {
        let rect = Rect::new(
            pos.x,
            pos.y,
            self.get_text_width(font, str),
            self.get_text_height(font, str),
        );
        let clipped = self.check_clip(rect);
        match clipped {
            Clip::All => return,
            Clip::Part => {
                let clip = self.get_clip_rect();
                self.set_clip(clip)
            }
            _ => (),
        }

        let str_start = self.push_text(str);
        self.push_command(Command::Text {
            str_start,
            str_len: str.len(),
            pos,
            color,
            font,
        });
        if clipped != Clip::None {
            self.set_clip(UNCLIPPED_RECT);
        }
    }

    pub fn draw_icon(&mut self, id: Icon, rect: Recti, color: Color4b) {
        let clipped = self.check_clip(rect);
        match clipped {
            Clip::All => return,
            Clip::Part => {
                let clip = self.get_clip_rect();
                self.set_clip(clip)
            }
            _ => (),
        }
        self.push_command(Command::Icon { id, rect, color });
        if clipped != Clip::None {
            self.set_clip(UNCLIPPED_RECT);
        }
    }

    fn in_hover_root(&mut self) -> bool {
        match self.hover_root {
            Some(hover_root) => {
                let len = self.container_stack.len();
                for i in 0..len {
                    if self.container_stack[len - i - 1] == hover_root {
                        return true;
                    }
                    if self.containers[self.container_stack[len - i - 1]]
                        .head_idx
                        .is_some()
                    {
                        break;
                    }
                }
                false
            }
            None => false,
        }
    }

    pub fn draw_control_frame(
        &mut self,
        id: Id,
        rect: Recti,
        mut colorid: ControlColor,
        opt: WidgetOption,
    ) {
        if opt.has_no_frame() {
            return;
        }

        if self.focus == Some(id) {
            colorid.focus()
        } else if self.hover == Some(id) {
            colorid.hover()
        }
        self.draw_frame(rect, colorid);
    }

    pub fn draw_control_text(
        &mut self,
        str: &str,
        rect: Recti,
        colorid: ControlColor,
        opt: WidgetOption,
    ) {
        let mut pos: Vec2i = Vec2i { x: 0, y: 0 };
        let font = self.style.font;
        let tw = self.get_text_width(font, str);
        self.push_clip_rect(rect);
        pos.y = rect.y + (rect.height - self.get_text_height(font, str)) / 2;
        if opt.is_aligned_center() {
            pos.x = rect.x + (rect.width - tw) / 2;
        } else if opt.is_aligned_right() {
            pos.x = rect.x + rect.width - tw - self.style.padding;
        } else {
            pos.x = rect.x + self.style.padding;
        }
        self.draw_text(font, str, pos, self.style.colors[colorid as usize]);
        self.pop_clip_rect();
    }

    pub fn mouse_over(&mut self, rect: Recti) -> bool {
        rect_overlaps_vec2(rect, self.mouse_pos)
            && rect_overlaps_vec2(self.get_clip_rect(), self.mouse_pos)
            && self.in_hover_root()
    }

    pub fn update_control(&mut self, id: Id, rect: Recti, opt: WidgetOption) {
        let mouseover = self.mouse_over(rect);
        if self.focus == Some(id) {
            self.updated_focus = true;
        }
        if opt.is_not_interactive() {
            return;
        }
        if mouseover && self.mouse_down.is_none() {
            self.hover = Some(id);
        }
        if self.focus == Some(id) {
            if !self.mouse_pressed.is_none() && !mouseover {
                self.set_focus(None);
            }
            if self.mouse_down.is_none() && !opt.is_holding_focus() {
                self.set_focus(None);
            }
        }
        if self.hover == Some(id) {
            if !self.mouse_pressed.is_none() {
                self.set_focus(Some(id));
            } else if !mouseover {
                self.hover = None;
            }
        }
    }

    pub fn get_text_width(&self, font: FontId, text: &str) -> i32 {
        let mut res = 0;
        let mut acc = 0;
        for c in text.chars() {
            if c == '\n' {
                res = usize::max(res, acc);
                acc = 0;
            }
            acc += self.renderer.get_char_width(font, c);
        }
        res = usize::max(res, acc);
        res as i32
    }

    pub fn get_text_height(&self, font: FontId, text: &str) -> i32 {
        let font_height = self.renderer.get_font_height(font);
        let lc = text.lines().count();
        (lc * font_height) as i32
    }

    fn header_internal(
        &mut self,
        label: &str,
        is_treenode: bool,
        opt: WidgetOption,
    ) -> ResourceState {
        let id: Id = self.get_id_from_str(label);
        let idx = self.treenode_pool.get(id);
        let mut expanded = 0;
        self.rows_with_line_config(&[-1], 0, |ctx| {
            let mut active = idx.is_some() as i32;
            expanded = if opt.is_expanded() {
                (active == 0) as i32
            } else {
                active
            };
            let mut r = ctx.layout_stack.next_cell(&ctx.style);
            ctx.update_control(id, r, WidgetOption::NONE);
            active ^= (ctx.mouse_pressed.is_left() && ctx.focus == Some(id)) as i32;
            if idx.is_some() {
                if active != 0 {
                    ctx.treenode_pool.update(idx.unwrap(), ctx.frame);
                } else {
                    ctx.treenode_pool.reset(idx.unwrap());
                }
            } else if active != 0 {
                ctx.treenode_pool.alloc(id, ctx.frame);
            }

            if is_treenode {
                if ctx.hover == Some(id) {
                    ctx.draw_frame(r, ControlColor::ButtonHover);
                }
            } else {
                ctx.draw_control_frame(id, r, ControlColor::Button, WidgetOption::NONE);
            }
            ctx.draw_icon(
                if expanded != 0 {
                    Icon::Expanded
                } else {
                    Icon::Collapsed
                },
                Rect::new(r.x, r.y, r.height, r.height),
                ctx.style.colors[ControlColor::Text as usize],
            );
            r.x += r.height - ctx.style.padding;
            r.width -= r.height - ctx.style.padding;
            ctx.draw_control_text(label, r, ControlColor::Text, WidgetOption::NONE);
        });
        return if expanded != 0 {
            ResourceState::ACTIVE
        } else {
            ResourceState::NONE
        };
    }

    fn header_ex(&mut self, label: &str, opt: WidgetOption) -> ResourceState {
        self.header_internal(label, false, opt)
    }

    fn begin_treenode_ex(&mut self, label: &str, opt: WidgetOption) -> ResourceState {
        let res = self.header_internal(label, true, opt);
        if res.is_active() && self.last_id.is_some() {
            self.layout_stack.top_mut().indent += self.style.indent;
            self.id_stack.push(self.last_id.unwrap());
        }
        return res;
    }

    fn end_treenode(&mut self) {
        self.layout_stack.top_mut().indent -= self.style.indent;
        self.pop_id();
    }

    fn clamp(x: i32, a: i32, b: i32) -> i32 {
        i32::min(b, i32::max(a, x))
    }

    fn scrollbars(&mut self, cnt_id: usize, body: &mut Recti) {
        let sz = self.style.scrollbar_size;
        let mut cs: Vec2i = self.containers[cnt_id].content_size;
        cs.x += self.style.padding * 2;
        cs.y += self.style.padding * 2;
        self.push_clip_rect(body.clone());
        if cs.y > self.containers[cnt_id].body.height {
            body.width -= sz;
        }
        if cs.x > self.containers[cnt_id].body.width {
            body.height -= sz;
        }
        let body = *body;
        let maxscroll = cs.y - body.height;
        if maxscroll > 0 && body.height > 0 {
            let id: Id = self.get_id_from_str("!scrollbary");
            let mut base = body;
            base.x = body.x + body.width;
            base.width = self.style.scrollbar_size;
            self.update_control(id, base, WidgetOption::NONE);
            if self.focus == Some(id) && self.mouse_down.is_left() {
                self.containers[cnt_id].scroll.y += self.mouse_delta.y * cs.y / base.height;
            }
            self.containers[cnt_id].scroll.y =
                Self::clamp(self.containers[cnt_id].scroll.y, 0, maxscroll);

            self.draw_frame(base, ControlColor::ScrollBase);
            let mut thumb = base;
            thumb.height = if self.style.thumb_size > base.height * body.height / cs.y {
                self.style.thumb_size
            } else {
                base.height * body.height / cs.y
            };
            thumb.y += self.containers[cnt_id].scroll.y * (base.height - thumb.height) / maxscroll;
            self.draw_frame(thumb, ControlColor::ScrollThumb);
            if self.mouse_over(body) {
                self.scroll_target = Some(cnt_id);
            }
        } else {
            self.containers[cnt_id].scroll.y = 0;
        }
        let maxscroll_0 = cs.x - body.width;
        if maxscroll_0 > 0 && body.width > 0 {
            let id_0: Id = self.get_id_from_str("!scrollbarx");
            let mut base_0 = body;
            base_0.y = body.y + body.height;
            base_0.height = self.style.scrollbar_size;
            self.update_control(id_0, base_0, WidgetOption::NONE);
            if self.focus == Some(id_0) && self.mouse_down.is_left() {
                self.containers[cnt_id].scroll.x += self.mouse_delta.x * cs.x / base_0.width;
            }
            self.containers[cnt_id].scroll.x =
                Self::clamp(self.containers[cnt_id].scroll.x, 0, maxscroll_0);

            self.draw_frame(base_0, ControlColor::ScrollBase);
            let mut thumb_0 = base_0;
            thumb_0.width = if self.style.thumb_size > base_0.width * body.width / cs.x {
                self.style.thumb_size
            } else {
                base_0.width * body.width / cs.x
            };
            thumb_0.x +=
                self.containers[cnt_id].scroll.x * (base_0.width - thumb_0.width) / maxscroll_0;
            self.draw_frame(thumb_0, ControlColor::ScrollThumb);
            if self.mouse_over(body) {
                self.scroll_target = Some(cnt_id);
            }
        } else {
            self.containers[cnt_id].scroll.x = 0;
        }
        self.pop_clip_rect();
    }

    fn push_container_body(&mut self, cnt_idx: usize, body: Recti, opt: WidgetOption) {
        let mut body = body;
        if !opt.has_no_scroll() {
            self.scrollbars(cnt_idx, &mut body);
        }

        let new_body = expand_rect(body, -self.style.padding);
        self.layout_stack.push_rect_scroll(
            new_body,
            self.containers[cnt_idx].scroll,
            if opt.is_auto_sizing() { true } else { false },
        );
        self.containers[cnt_idx].body = body;
    }

    fn begin_root_container(&mut self, cnt: usize) {
        self.container_stack.push(cnt);

        self.root_list.push(cnt);
        self.containers[cnt].head_idx = Some(self.push_jump(None));
        if rect_overlaps_vec2(self.containers[cnt].rect, self.mouse_pos)
            && (self.next_hover_root.is_none()
                || self.containers[cnt].zindex
                    > self.containers[self.next_hover_root.unwrap()].zindex)
        {
            self.next_hover_root = Some(cnt);
        }
        self.clip_stack.push(UNCLIPPED_RECT);
    }

    fn end_root_container(&mut self) {
        let cnt = self.get_current_container();
        self.containers[cnt].tail_idx = Some(self.push_jump(None));
        self.command_list[self.containers[cnt].head_idx.unwrap()] = Command::Jump {
            dst_idx: Some(self.command_list.len()),
        };

        self.pop_clip_rect();
        self.pop_container();
    }

    fn begin_window(&mut self, title: &str, mut r: Recti, opt: WidgetOption) -> ResourceState {
        let id = self.get_id_from_str(title);
        let cnt_id = self.get_container_index_intern(id, opt);
        if cnt_id.is_none() || !self.containers[cnt_id.unwrap()].open {
            return ResourceState::NONE;
        }
        self.id_stack.push(id);

        if self.containers[cnt_id.unwrap()].rect.width == 0 {
            self.containers[cnt_id.unwrap()].rect = r;
        }
        self.begin_root_container(cnt_id.unwrap());

        let mut body = self.containers[cnt_id.unwrap()].rect;
        r = body;
        if !opt.has_no_frame() {
            self.draw_frame(r, ControlColor::WindowBG);
        }
        if !opt.has_no_title() {
            let mut tr = r;
            tr.height = self.style.title_height;
            self.draw_frame(tr, ControlColor::TitleBG);

            // TODO: Is this necessary?
            if !opt.has_no_title() {
                let id = self.get_id_from_str("!title");
                self.update_control(id, tr, opt);
                self.draw_control_text(title, tr, ControlColor::TitleText, opt);
                if Some(id) == self.focus && self.mouse_down.is_left() {
                    self.containers[cnt_id.unwrap()].rect.x += self.mouse_delta.x;
                    self.containers[cnt_id.unwrap()].rect.y += self.mouse_delta.y;
                }
                body.y += tr.height;
                body.height -= tr.height;
            }
            if !opt.has_no_close() {
                let id = self.get_id_from_str("!close");
                let r = Rect::new(tr.x + tr.width - tr.height, tr.y, tr.height, tr.height);
                tr.width -= r.width;
                self.draw_icon(
                    Icon::Close,
                    r,
                    self.style.colors[ControlColor::TitleText as usize],
                );
                self.update_control(id, r, opt);
                if self.mouse_pressed.is_left() && Some(id) == self.focus {
                    self.containers[cnt_id.unwrap()].open = false;
                }
            }
        }

        self.push_container_body(cnt_id.unwrap(), body, opt);
        if !opt.is_auto_sizing() {
            let sz = self.style.title_height;
            let id_2 = self.get_id_from_str("!resize");
            let r_0 = Recti::new(r.x + r.width - sz, r.y + r.height - sz, sz, sz);
            self.update_control(id_2, r_0, opt);
            if Some(id_2) == self.focus && self.mouse_down.is_left() {
                self.containers[cnt_id.unwrap()].rect.width =
                    if 96 > self.containers[cnt_id.unwrap()].rect.width + self.mouse_delta.x {
                        96
                    } else {
                        self.containers[cnt_id.unwrap()].rect.width + self.mouse_delta.x
                    };
                self.containers[cnt_id.unwrap()].rect.height =
                    if 64 > self.containers[cnt_id.unwrap()].rect.height + self.mouse_delta.y {
                        64
                    } else {
                        self.containers[cnt_id.unwrap()].rect.height + self.mouse_delta.y
                    };
            }
        }
        if opt.is_auto_sizing() {
            let r = self.layout_stack.top().body;
            let container = &mut self.containers[cnt_id.unwrap()];
            container.rect.width = container.content_size.x + (container.rect.width - r.width);
            container.rect.height = container.content_size.y + (container.rect.height - r.height);
        }

        if opt.is_popup() && !self.mouse_pressed.is_none() && self.hover_root != cnt_id {
            self.containers[cnt_id.unwrap()].open = false;
        }
        self.push_clip_rect(self.containers[cnt_id.unwrap()].body);

        return ResourceState::ACTIVE;
    }

    fn end_window(&mut self) {
        self.pop_clip_rect();
        self.end_root_container();
    }

    pub fn open_popup(&mut self, name: &str) {
        let cnt = self.get_container_index(name);
        self.next_hover_root = cnt;
        self.hover_root = self.next_hover_root;
        self.containers[cnt.unwrap()].rect = Rect::new(self.mouse_pos.x, self.mouse_pos.y, 1, 1);
        self.containers[cnt.unwrap()].open = true;
        self.bring_to_front(cnt.unwrap());
    }

    fn begin_popup(&mut self, name: &str) -> ResourceState {
        let opt = WidgetOption::POPUP
            | WidgetOption::AUTO_SIZE
            | WidgetOption::NO_RESIZE
            | WidgetOption::NO_SCROLL
            | WidgetOption::NO_TITLE
            | WidgetOption::CLOSED;
        return self.begin_window(name, Rect::new(0, 0, 0, 0), opt);
    }

    fn end_popup(&mut self) {
        self.end_window();
    }

    fn begin_panel_ex(&mut self, name: &str, opt: WidgetOption) {
        self.push_id_from_str(name);
        let cnt_id = self.get_container_index_intern(self.last_id.unwrap(), opt);
        let rect = self.layout_stack.next_cell(&self.style);
        self.containers[cnt_id.unwrap()].rect = rect;
        if !opt.has_no_frame() {
            self.draw_frame(rect, ControlColor::PanelBG);
        }

        self.container_stack.push(cnt_id.unwrap());
        self.push_container_body(cnt_id.unwrap(), rect, opt);
        self.push_clip_rect(self.containers[cnt_id.unwrap()].body);
    }

    fn end_panel(&mut self) {
        self.pop_clip_rect();
        self.pop_container();
    }

    fn paint(&mut self) {
        let mut cmd_id = 0;
        loop {
            match self.mu_next_command(cmd_id) {
                Some((command, id)) => {
                    match command {
                        Command::Text {
                            str_start,
                            str_len,
                            pos,
                            color,
                            ..
                        } => {
                            let str = &self.text_stack[str_start..str_start + str_len];
                            self.renderer.draw_text(str, pos, color);
                        }
                        Command::Rect { rect, color } => {
                            self.renderer.draw_rect(rect, color);
                        }
                        Command::Icon { id, rect, color } => {
                            self.renderer.draw_icon(id, rect, color);
                        }
                        Command::Clip { rect } => {
                            self.renderer.set_clip_rect(rect);
                        }
                        _ => {}
                    }
                    cmd_id = id;
                }
                None => break,
            }
        }

        self.renderer.flush();
    }

    ////////////////////////////////////////////////////////////////////////////
    // Queries
    ////////////////////////////////////////////////////////////////////////////
    pub fn frame_size(&self) -> (usize, usize) {
        self.renderer.frame_size()
    }

    pub fn current_rect(&self) -> Recti {
        self.layout_stack.last_rect()
    }

    ////////////////////////////////////////////////////////////////////////////
    // LAMBDA based context
    ////////////////////////////////////////////////////////////////////////////

    pub fn popup<F: FnOnce(&mut Self)>(&mut self, name: &str, f: F) -> ResourceState {
        let res = self.begin_popup(name);
        if !res.is_none() {
            f(self);
            self.end_popup();
        }
        res
    }

    pub fn treenode<F: FnOnce(&mut Self)>(
        &mut self,
        label: &str,
        opt: WidgetOption,
        f: F,
    ) -> ResourceState {
        let res = self.begin_treenode_ex(label, opt);
        if !res.is_none() {
            f(self);
            self.end_treenode()
        }
        res
    }

    pub fn window<F: FnOnce(&mut Self)>(
        &mut self,
        title: &str,
        r: Recti,
        opt: WidgetOption,
        f: F,
    ) -> ResourceState {
        let res = self.begin_window(title, r, opt);
        if !res.is_none() {
            f(self);
            self.end_window();
        }
        res
    }

    pub fn panel<F: FnOnce(&mut Self)>(&mut self, name: &str, opt: WidgetOption, f: F) {
        self.begin_panel_ex(name, opt);
        f(self);
        self.end_panel();
    }

    pub fn frame<F: FnOnce(&mut Self)>(&mut self, width: usize, height: usize, f: F) -> P {
        self.begin(width, height);
        f(self);
        self.end()
    }

    pub fn column<F: FnOnce(&mut Self)>(&mut self, f: F) {
        self.layout_stack.begin_column(&self.style);
        f(self);
        self.layout_stack.end_column()
    }

    pub fn rows_with_line_config<F: FnOnce(&mut Self)>(
        &mut self,
        widths: &[i32],
        height: i32,
        f: F,
    ) {
        self.layout_stack.row_config(widths, height);
        f(self);
    }

    pub fn next_cell(&mut self) -> Recti {
        self.layout_stack.next_cell(&self.style)
    }

    pub fn header<F: FnOnce(&mut Self)>(
        &mut self,
        label: &str,
        opt: WidgetOption,
        f: F,
    ) -> ResourceState {
        let res = self.header_internal(label, false, opt);
        if res.is_active() && self.last_id.is_some() {
            f(self);
        }
        res
    }
}
