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
use std::marker::PhantomData;

mod atlas_data;
pub use atlas_data::*;

mod num_appender;
pub use num_appender::*;

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
    fn draw_text(&mut self, font: FontId, text: &str, pos: Vec2i, color: Color4b);
    fn draw_icon(&mut self, id: usize, r: Recti, color: Color4b);
    fn add_render_pass_commands(&mut self, commands: P);

    fn set_clip_rect(&mut self, rect: Recti);

    fn flush(&mut self);

    fn frame_size(&self) -> (usize, usize);

    fn set_atlas(atlas: &Atlas);
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
        Self { vec: [PoolItem::default(); N] }
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
        const SET_SIZE = 4096;
        const EXPANDED = 2048;
        const CLOSED = 1024;
        const POPUP= 512;
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
    pub fn is_setting_size(&self) -> bool {
        self.intersects(WidgetOption::SET_SIZE)
    }
    pub fn is_expanded(&self) -> bool {
        self.intersects(WidgetOption::EXPANDED)
    }
    pub fn is_closed(&self) -> bool {
        self.intersects(WidgetOption::CLOSED)
    }
    pub fn is_popup(&self) -> bool {
        self.intersects(WidgetOption::POPUP)
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
pub struct Context<P: Default, R: RendererBackEnd<P>> {
    pub hover: Option<Id>,
    pub focus: Option<Id>,
    pub last_id: Option<Id>,
    pub last_zindex: i32,
    pub updated_focus: bool,
    pub frame: usize,
    pub hover_root: Option<ContRef>,
    pub next_hover_root: Option<ContRef>,
    pub scroll_target: Option<ContRef>,
    pub number_edit_buf: String,
    pub number_edit: Option<Id>,
    pub root_list: Vec<ContRef>,
    pub container_stack: Vec<ContRef>,
    pub clip_stack: Vec<Recti>,
    pub id_stack: Vec<Id>,
    text_stack: String,
    container_pool: Pool<48>,
    containers: [Container<P>; 48],
    treenode_pool: Pool<48>,
    pub mouse_pos: Vec2i,
    pub last_mouse_pos: Vec2i,
    pub mouse_delta: Vec2i,
    pub scroll_delta: Vec2i,
    pub mouse_down: MouseButton,
    pub mouse_pressed: MouseButton,
    pub key_down: KeyModifier,
    pub key_pressed: KeyModifier,
    pub input_text: String,
    slider_buff: String,
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

#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub struct ContRef(usize);

#[derive(Default)]
pub struct Container<P: Default> {
    pub rect: Recti,
    pub body: Recti,
    pub content_size: Vec2i,
    pub scroll: Vec2i,
    pub zindex: i32,
    pub open: bool,
    pub commands: Vec<Command<P>>,

    layout_stack: LayoutStack,
}

impl<P: Default> Container<P> {
    pub fn next_cell(&mut self, style: &Style) -> Recti {
        self.layout_stack.next_cell(style)
    }

    pub fn layout(&self) -> &Layout {
        self.layout_stack.top()
    }

    pub fn layout_mut(&mut self) -> &mut Layout {
        self.layout_stack.top_mut()
    }

    pub fn push_rect_scroll(&mut self, body: Recti) {
        self.layout_stack.push_rect_scroll(body, self.scroll)
    }

    pub fn begin_column(&mut self, style: &Style) {
        self.layout_stack.begin_column(style)
    }

    pub fn end_column(&mut self) {
        self.layout_stack.end_column()
    }

    pub fn row_config(&mut self, widths: &[i32], height: i32) {
        self.layout_stack.row_config(widths, height)
    }

    pub fn end_row(&mut self) {
        self.layout_stack.end_row();
    }

    pub fn default_cell_height(&self, style: &Style) -> i32 {
        self.layout_stack.default_cell_height(style)
    }
}

pub enum Command<P: Default> {
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
        id: usize,
        color: Color4b,
    },
    DirectRenderPassCommands {
        pass: P,
    },
    None,
}

impl<P: Default> Default for Command<P> {
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

#[derive(Clone)]
pub struct Style {
    pub bold_font: FontId,
    pub normal_font: FontId,
    pub console_font: FontId,
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
            bold_font: FontId(BOLD),
            normal_font: FontId(NORMAL),
            console_font: FontId(CONSOLE),
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
    Rect::new(r.x - n, r.y - n, i32::max(0, r.width + n * 2), i32::max(0, r.height + n * 2))
}

// pub fn intersect_rects(r1: Recti, r2: Recti) -> Recti {
//     let x1 = i32::max(r1.x, r2.x);
//     let y1 = i32::max(r1.y, r2.y);
//     let mut x2 = i32::min(r1.x + r1.width, r2.x + r2.width);
//     let mut y2 = i32::min(r1.y + r1.height, r2.y + r2.height);
//     if x2 < x1 {
//         x2 = x1;
//     }
//     if y2 < y1 {
//         y2 = y1;
//     }
//     return Rect::new(x1, y1, x2 - x1, y2 - y1);
// }

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

impl<P: Default, R: RendererBackEnd<P>> Context<P, R> {
    pub fn new(renderer: R) -> Self {
        Self {
            hover: None,
            focus: None,
            last_id: None,
            last_zindex: 0,
            updated_focus: false,
            frame: 0,
            hover_root: None,
            next_hover_root: None,
            scroll_target: None,
            number_edit_buf: String::default(),
            number_edit: None,
            root_list: Vec::default(),
            container_stack: Vec::default(),
            clip_stack: Vec::default(),
            id_stack: Vec::default(),
            text_stack: String::default(),
            container_pool: Pool::default(),
            containers: [(); 48].map(|_| Container::default()),
            treenode_pool: Pool::default(),
            mouse_pos: Vec2i::default(),
            last_mouse_pos: Vec2i::default(),
            mouse_delta: Vec2i::default(),
            scroll_delta: Vec2i::default(),
            mouse_down: MouseButton::NONE,
            mouse_pressed: MouseButton::NONE,
            key_down: KeyModifier::NONE,
            key_pressed: KeyModifier::NONE,
            input_text: String::default(),
            slider_buff: String::new(),
            renderer,
            _unused: PhantomData::default(),
        }
    }

    fn draw_frame(&mut self, style: &Style, rect: Recti, colorid: ControlColor) {
        self.draw_rect(rect, style.colors[colorid as usize]);
        if colorid == ControlColor::ScrollBase || colorid == ControlColor::ScrollThumb || colorid == ControlColor::TitleBG {
            return;
        }
        if style.colors[ControlColor::Border as usize].w != 0 {
            // alpha
            self.draw_box(expand_rect(rect, 1), style.colors[ControlColor::Border as usize]);
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
        for container in &mut self.containers {
            container.commands.clear();
        }
        self.frame += 1;
        self.renderer.begin_frame(width, height);
    }

    fn end(&mut self) -> P {
        assert_eq!(self.container_stack.len(), 0);
        assert_eq!(self.clip_stack.len(), 0);
        assert_eq!(self.id_stack.len(), 0);
        match self.scroll_target {
            Some(cnt_ref) => {
                let mut container = &mut self.containers[cnt_ref.0];
                container.scroll.x += self.scroll_delta.x;
                container.scroll.y += self.scroll_delta.y;
            }
            None => (),
        }

        if !self.updated_focus {
            self.focus = None;
        }
        self.updated_focus = false;
        match self.next_hover_root {
            Some(cnt_ref)
                if !self.mouse_pressed.is_none() && self.containers[cnt_ref.0].zindex < self.last_zindex && self.containers[cnt_ref.0].zindex >= 0 =>
            {
                self.bring_to_front(cnt_ref);
            }
            _ => (),
        }
        self.key_pressed = KeyModifier::NONE;
        self.input_text.clear();
        self.slider_buff.clear();
        self.mouse_pressed = MouseButton::NONE;
        self.scroll_delta = vec2(0, 0);
        self.last_mouse_pos = self.mouse_pos;
        let containers = &self.containers;
        self.root_list.sort_by(|a, b| containers[a.0].zindex.cmp(&containers[b.0].zindex));

        self.paint();
        self.renderer.end_frame()
    }

    pub fn set_focus(&mut self, id: Option<Id>) {
        self.focus = id;
        self.updated_focus = true;
    }

    pub fn get_id_u32(&mut self, orig_id: u32) -> Id {
        let mut res: Id = match self.id_stack.last() {
            Some(id) => *id,
            None => Id(2166136261),
        };
        hash_u32(&mut res, orig_id);
        self.last_id = Some(res);
        return res;
    }

    pub fn get_id_from_ptr<T: ?Sized>(&mut self, orig_id: &T) -> Id {
        let mut res: Id = match self.id_stack.last() {
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
        let mut res: Id = match self.id_stack.last() {
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

    #[must_use]
    pub fn push_clip_rect(&mut self, rect: Recti) -> Option<Recti> {
        let last = self.get_clip_rect();
        match rect.intersect(&last) {
            Some(isect) => {
                self.clip_stack.push(isect);
                Some(isect)
            }
            _ => None,
        }
    }

    pub fn pop_clip_rect(&mut self) {
        self.clip_stack.pop();
    }

    pub fn get_clip_rect(&mut self) -> Recti {
        *self.clip_stack.last().unwrap()
    }

    pub fn check_clip(&mut self, r: Recti) -> Clip {
        let cr = self.get_clip_rect();
        if r.x > cr.x + cr.width || r.x + r.width < cr.x || r.y > cr.y + cr.height || r.y + r.height < cr.y {
            return Clip::All;
        }
        if r.x >= cr.x && r.x + r.width <= cr.x + cr.width && r.y >= cr.y && r.y + r.height <= cr.y + cr.height {
            return Clip::None;
        }
        return Clip::Part;
    }

    fn pop_container(&mut self) {
        let cnt = self.get_current_container();
        let layout = self.containers[cnt.0].layout();
        let cx = layout.max.x - layout.body.x;
        let cy = layout.max.y - layout.body.y;
        self.containers[cnt.0].content_size.x = cx;
        self.containers[cnt.0].content_size.y = cy;

        self.container_stack.pop();
        self.pop_id();
    }

    pub fn get_current_container(&self) -> ContRef {
        *self.container_stack.last().unwrap()
    }

    pub fn get_current_container_rect(&self) -> Recti {
        self.containers[self.container_stack.last().unwrap().0].rect
    }

    pub fn set_current_container_rect(&mut self, rect: &Recti) {
        self.containers[self.container_stack.last().unwrap().0].rect = *rect;
    }

    pub fn get_current_container_scroll(&self) -> Vec2i {
        self.containers[self.container_stack.last().unwrap().0].scroll
    }

    pub fn set_current_container_scroll(&mut self, scroll: &Vec2i) {
        self.containers[self.container_stack.last().unwrap().0].scroll = *scroll;
    }

    pub fn get_current_container_content_size(&self) -> Vec2i {
        self.containers[self.container_stack.last().unwrap().0].content_size
    }

    pub fn get_current_container_body(&self) -> Recti {
        self.containers[self.container_stack.last().unwrap().0].body
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
        self.containers[idx].open = true;
        self.bring_to_front(ContRef(idx));
        Some(idx)
    }

    fn get_container_index(&mut self, name: &str) -> Option<usize> {
        let id = self.get_id_from_str(name);
        self.get_container_index_intern(id, WidgetOption::NONE)
    }

    pub fn bring_to_front(&mut self, cnt: ContRef) {
        // TODO: only increment by 1 once we have proper container nesting
        self.last_zindex += 128;
        self.containers[cnt.0].zindex = self.last_zindex;
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

    pub fn push_command(&mut self, cmd: Command<P>) -> &mut Command<P> {
        let cnt = *self.container_stack.last().unwrap();
        let id = self.containers[cnt.0].commands.len();

        self.containers[cnt.0].commands.push(cmd);
        self.containers[cnt.0].commands.get_mut(id).unwrap()
    }

    pub fn push_text(&mut self, str: &str) -> usize {
        let str_start = self.text_stack.len();
        for c in str.chars() {
            self.text_stack.push(c);
        }
        return str_start;
    }

    pub fn set_clip(&mut self, rect: Recti) {
        self.push_command(Command::Clip { rect });
    }

    pub fn draw_rect(&mut self, rect: Recti, color: Color4b) {
        let rect = rect.intersect(&self.get_clip_rect()); //intersect_rects(rect, self.get_clip_rect());
        match rect {
            Some(rect) => {
                self.push_command(Command::Rect { rect, color });
            }
            _ => (),
        };
    }

    pub fn draw_box(&mut self, r: Recti, color: Color4b) {
        self.draw_rect(Rect::new(r.x + 1, r.y, r.width - 2, 1), color);
        self.draw_rect(Rect::new(r.x + 1, r.y + r.height - 1, r.width - 2, 1), color);
        self.draw_rect(Rect::new(r.x, r.y, 1, r.height), color);
        self.draw_rect(Rect::new(r.x + r.width - 1, r.y, 1, r.height), color);
    }

    pub fn draw_text(&mut self, font: FontId, str: &str, pos: Vec2i, color: Color4b) {
        let rect = Rect::new(pos.x, pos.y, self.get_text_width(font, str), self.get_text_height(font, str));
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

    pub fn draw_icon(&mut self, id: usize, rect: Recti, color: Color4b) {
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
                }
                false
            }
            None => false,
        }
    }

    pub fn default_cell_height(&self, style: &Style) -> i32 {
        self.containers[self.container_stack.last().unwrap().0].default_cell_height(style)
    }

    pub fn draw_control_frame(&mut self, style: &Style, id: Id, rect: Recti, mut colorid: ControlColor, opt: WidgetOption) {
        if opt.has_no_frame() {
            return;
        }

        if self.focus == Some(id) {
            colorid.focus()
        } else if self.hover == Some(id) {
            colorid.hover()
        }
        self.draw_frame(style, rect, colorid);
    }

    pub fn draw_control_text(&mut self, style: &Style, font: FontId, str: &str, rect: Recti, colorid: ControlColor, opt: WidgetOption) {
        let mut pos: Vec2i = Vec2i { x: 0, y: 0 };
        let tw = self.get_text_width(font, str);
        match self.push_clip_rect(rect) {
            Some(_) => {
                pos.y = rect.y + (rect.height - self.get_text_height(font, str)) / 2;
                if opt.is_aligned_center() {
                    pos.x = rect.x + (rect.width - tw) / 2;
                } else if opt.is_aligned_right() {
                    pos.x = rect.x + rect.width - tw - style.padding;
                } else {
                    pos.x = rect.x + style.padding;
                }
                self.draw_text(font, str, pos, style.colors[colorid as usize]);
                self.pop_clip_rect();
            }
            None => (),
        }
    }

    pub fn mouse_over(&mut self, rect: Recti) -> bool {
        rect_overlaps_vec2(rect, self.mouse_pos) && rect_overlaps_vec2(self.get_clip_rect(), self.mouse_pos) && self.in_hover_root()
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
            //acc += self.renderer.get_char_width(font, c);
            if (c as usize) < 127 {
                let chr = usize::min(c as usize, 127);
                let entry = &ATLAS.fonts[font.0].1.entries[chr - 32];
                acc += entry.advance.x as usize;
            }
        }
        res = usize::max(res, acc);
        res as i32
    }

    pub fn get_text_height(&self, font: FontId, text: &str) -> i32 {
        let font_height = self.renderer.get_font_height(font);
        let lc = text.lines().count();
        (lc * font_height) as i32
    }

    fn header_internal(&mut self, style: &Style, font: FontId, label: &str, is_treenode: bool, opt: WidgetOption) -> ResourceState {
        let id: Id = self.get_id_from_str(label);
        let idx = self.treenode_pool.get(id);
        let mut expanded = 0;

        self.rows_with_line_config(style, &[-1], 0, |ctx, style| {
            let mut active = idx.is_some() as i32;
            expanded = if opt.is_expanded() { (active == 0) as i32 } else { active };
            let mut r = ctx.next_cell(style);
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
                    ctx.draw_frame(style, r, ControlColor::ButtonHover);
                }
            } else {
                ctx.draw_control_frame(style, id, r, ControlColor::Button, WidgetOption::NONE);
            }
            ctx.draw_icon(
                if expanded != 0 { MINUS } else { PLUS },
                Rect::new(r.x, r.y, r.height, r.height),
                style.colors[ControlColor::Text as usize],
            );
            r.x += r.height - style.padding;
            r.width -= r.height - style.padding;
            ctx.draw_control_text(style, font, label, r, ControlColor::Text, WidgetOption::NONE);
        });
        return if expanded != 0 { ResourceState::ACTIVE } else { ResourceState::NONE };
    }

    fn header_ex(&mut self, style: &Style, font: FontId, label: &str, opt: WidgetOption) -> ResourceState {
        self.header_internal(style, font, label, false, opt)
    }

    fn begin_treenode_ex(&mut self, style: &Style, label: &str, opt: WidgetOption) -> ResourceState {
        let res = self.header_internal(style, style.normal_font, label, true, opt);
        if res.is_active() && self.last_id.is_some() {
            self.layout_mut().indent += style.indent;
            self.id_stack.push(self.last_id.unwrap());
        }
        return res;
    }

    fn end_treenode(&mut self, style: &Style) {
        self.layout_mut().indent -= style.indent;
        self.pop_id();
    }

    fn clamp(x: i32, a: i32, b: i32) -> i32 {
        i32::min(b, i32::max(a, x))
    }

    fn scrollbars(&mut self, style: &Style, cnt_id: usize, body: &mut Recti) {
        let sz = style.scrollbar_size;
        let mut cs: Vec2i = self.containers[cnt_id].content_size;
        cs.x += style.padding * 2;
        cs.y += style.padding * 2;
        if self.push_clip_rect(body.clone()).is_none() {
            return;
        }
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
            base.width = style.scrollbar_size;
            self.update_control(id, base, WidgetOption::NONE);
            if self.focus == Some(id) && self.mouse_down.is_left() {
                self.containers[cnt_id].scroll.y += self.mouse_delta.y * cs.y / base.height;
            }
            self.containers[cnt_id].scroll.y = Self::clamp(self.containers[cnt_id].scroll.y, 0, maxscroll);

            self.draw_frame(style, base, ControlColor::ScrollBase);
            let mut thumb = base;
            thumb.height = if style.thumb_size > base.height * body.height / cs.y {
                style.thumb_size
            } else {
                base.height * body.height / cs.y
            };
            thumb.y += self.containers[cnt_id].scroll.y * (base.height - thumb.height) / maxscroll;
            self.draw_frame(style, thumb, ControlColor::ScrollThumb);
            if self.mouse_over(body) {
                self.scroll_target = Some(ContRef(cnt_id));
            }
        } else {
            self.containers[cnt_id].scroll.y = 0;
        }
        let maxscroll_0 = cs.x - body.width;
        if maxscroll_0 > 0 && body.width > 0 {
            let id_0: Id = self.get_id_from_str("!scrollbarx");
            let mut base_0 = body;
            base_0.y = body.y + body.height;
            base_0.height = style.scrollbar_size;
            self.update_control(id_0, base_0, WidgetOption::NONE);
            if self.focus == Some(id_0) && self.mouse_down.is_left() {
                self.containers[cnt_id].scroll.x += self.mouse_delta.x * cs.x / base_0.width;
            }
            self.containers[cnt_id].scroll.x = Self::clamp(self.containers[cnt_id].scroll.x, 0, maxscroll_0);

            self.draw_frame(style, base_0, ControlColor::ScrollBase);
            let mut thumb_0 = base_0;
            thumb_0.width = if style.thumb_size > base_0.width * body.width / cs.x {
                style.thumb_size
            } else {
                base_0.width * body.width / cs.x
            };
            thumb_0.x += self.containers[cnt_id].scroll.x * (base_0.width - thumb_0.width) / maxscroll_0;
            self.draw_frame(style, thumb_0, ControlColor::ScrollThumb);
            if self.mouse_over(body) {
                self.scroll_target = Some(ContRef(cnt_id));
            }
        } else {
            self.containers[cnt_id].scroll.x = 0;
        }
        self.pop_clip_rect();
    }

    fn push_container_body(&mut self, style: &Style, cnt_idx: usize, body: Recti, opt: WidgetOption) {
        let mut body = body;
        if !opt.has_no_scroll() {
            self.scrollbars(style, cnt_idx, &mut body);
        }

        let new_body = expand_rect(body, -style.padding);
        self.containers[cnt_idx].push_rect_scroll(new_body);
        self.containers[cnt_idx].body = body;
    }

    fn begin_root_container(&mut self, cnt: ContRef) {
        self.container_stack.push(cnt);

        self.root_list.push(cnt);
        self.containers[cnt.0].commands.clear();
        if rect_overlaps_vec2(self.containers[cnt.0].rect, self.mouse_pos)
            && (self.next_hover_root.is_none() || self.containers[cnt.0].zindex > self.containers[self.next_hover_root.unwrap().0].zindex)
        {
            self.next_hover_root = Some(cnt);
        }
        self.clip_stack.push(UNCLIPPED_RECT);
    }

    fn end_root_container(&mut self) {
        self.pop_clip_rect();
        self.pop_container();
    }

    fn begin_window(&mut self, style: &Style, title: &str, mut r: Recti, opt: WidgetOption) -> ResourceState {
        let id = self.get_id_from_str(title);
        let cnt_id = self.get_container_index_intern(id, opt);
        if cnt_id.is_none() || !self.containers[cnt_id.unwrap()].open {
            return ResourceState::NONE;
        }
        self.id_stack.push(id);

        let container = &mut self.containers[cnt_id.unwrap()];
        if opt.is_setting_size() {
            container.rect.width = r.width;
            container.rect.height = r.height;
        }

        if container.rect.width == 0 {
            container.rect.width = r.width;
        }
        if container.rect.height == 0 {
            container.rect.height = r.height;
        }
        if container.rect.x == 0 {
            container.rect.x = r.x;
        }
        if container.rect.y == 0 {
            container.rect.y = r.y;
        }

        self.begin_root_container(ContRef(cnt_id.unwrap()));

        let mut body = self.containers[cnt_id.unwrap()].rect;
        r = body;
        if !opt.has_no_frame() {
            self.draw_frame(style, r, ControlColor::WindowBG);
        }
        if !opt.has_no_title() {
            let mut tr = r;
            tr.height = style.title_height;
            self.draw_frame(style, tr, ControlColor::TitleBG);

            // TODO: Is this necessary?
            if !opt.has_no_title() {
                let id = self.get_id_from_str("!title");
                self.update_control(id, tr, opt);
                self.draw_control_text(style, style.bold_font, title, tr, ControlColor::TitleText, opt);
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
                self.draw_icon(CLOSE, r, style.colors[ControlColor::TitleText as usize]);
                self.update_control(id, r, opt);
                if self.mouse_pressed.is_left() && Some(id) == self.focus {
                    self.containers[cnt_id.unwrap()].open = false;
                }
            }
        }

        self.push_container_body(style, cnt_id.unwrap(), body, opt);

        let sz = style.title_height;
        let id_2 = self.get_id_from_str("!resize");
        let r_0 = Recti::new(r.x + r.width - sz, r.y + r.height - sz, sz, sz);
        self.update_control(id_2, r_0, opt);
        if Some(id_2) == self.focus && self.mouse_down.is_left() {
            self.containers[cnt_id.unwrap()].rect.width = if 96 > self.containers[cnt_id.unwrap()].rect.width + self.mouse_delta.x {
                96
            } else {
                self.containers[cnt_id.unwrap()].rect.width + self.mouse_delta.x
            };
            self.containers[cnt_id.unwrap()].rect.height = if 64 > self.containers[cnt_id.unwrap()].rect.height + self.mouse_delta.y {
                64
            } else {
                self.containers[cnt_id.unwrap()].rect.height + self.mouse_delta.y
            };
        }

        if opt.is_popup() && !self.mouse_pressed.is_none() && self.hover_root != cnt_id.map(|x| ContRef(x)) {
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
        self.next_hover_root = cnt.map(|x| ContRef(x));
        self.hover_root = self.next_hover_root;
        self.containers[cnt.unwrap()].rect = Rect::new(self.mouse_pos.x, self.mouse_pos.y, 0, 0);
        self.containers[cnt.unwrap()].open = true;
        self.bring_to_front(ContRef(cnt.unwrap()));
    }

    fn begin_popup(&mut self, style: &Style, name: &str, rect: Recti) -> ResourceState {
        let opt = WidgetOption::POPUP | WidgetOption::NO_RESIZE | WidgetOption::NO_SCROLL | WidgetOption::NO_TITLE | WidgetOption::CLOSED;
        return self.begin_window(style, name, rect, opt);
    }

    fn end_popup(&mut self) {
        self.end_window();
    }

    fn begin_panel_ex(&mut self, style: &Style, name: &str, opt: WidgetOption) {
        self.push_id_from_str(name);
        let cnt_id = self.get_container_index_intern(self.last_id.unwrap(), opt);
        let rect = self.next_cell(style);
        self.root_list.push(ContRef(cnt_id.unwrap()));
        self.containers[cnt_id.unwrap()].rect = rect;

        //
        // TODO: quick and dirty hack to solve the problem of nested container zindex ordering
        //      Fix this when we have proper container nesting
        //
        let zindex = self.containers[self.container_stack.last().unwrap().0].zindex + 1;
        self.containers[cnt_id.unwrap()].rect = rect;
        self.containers[cnt_id.unwrap()].zindex = zindex;

        if !opt.has_no_frame() {
            self.draw_frame(style, rect, ControlColor::PanelBG);
        }

        self.container_stack.push(ContRef(cnt_id.unwrap()));
        self.push_container_body(style, cnt_id.unwrap(), rect, opt);
        self.push_clip_rect(self.containers[cnt_id.unwrap()].body);
    }

    fn end_panel(&mut self) {
        self.pop_clip_rect();
        self.pop_container();
    }

    fn paint(&mut self) {
        for cnt in &self.root_list {
            let container = &mut self.containers[cnt.0];
            let mut commands = Vec::new();
            std::mem::swap(&mut commands, &mut container.commands);
            for command in commands {
                match command {
                    Command::Text {
                        str_start,
                        str_len,
                        pos,
                        color,
                        font,
                    } => {
                        let str = &self.text_stack[str_start..str_start + str_len];
                        self.renderer.draw_text(font, str, pos, color);
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
                    Command::DirectRenderPassCommands { pass } => {
                        self.renderer.add_render_pass_commands(pass);
                    }
                    Command::None => (),
                }
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
        self.containers[self.container_stack.last().unwrap().0].layout_stack.last_rect()
    }

    ////////////////////////////////////////////////////////////////////////////
    // LAMBDA based context
    ////////////////////////////////////////////////////////////////////////////

    pub fn popup<Res, F: FnOnce(&mut Self, &Style) -> Res>(&mut self, style: &Style, name: &str, rect: Recti, f: F) -> (ResourceState, Option<Res>) {
        let res = self.begin_popup(style, name, rect);
        if !res.is_none() {
            let r = f(self, style);
            self.end_popup();
            return (res, Some(r));
        }
        (res, None)
    }

    pub fn treenode<Res, F: FnOnce(&mut Self, &Style) -> Res>(&mut self, style: &Style, label: &str, opt: WidgetOption, f: F) -> (ResourceState, Option<Res>) {
        let res = self.begin_treenode_ex(style, label, opt);
        if !res.is_none() {
            let r = f(self, style);
            self.end_treenode(style);
            return (res, Some(r));
        }
        (res, None)
    }

    pub fn window<Res, F: FnOnce(&mut Self, &Style) -> Res>(
        &mut self,
        style: &Style,
        title: &str,
        r: Recti,
        opt: WidgetOption,
        f: F,
    ) -> (ResourceState, Option<Res>) {
        let res = self.begin_window(style, title, r, opt);
        if !res.is_none() {
            let r = f(self, style);
            self.end_window();
            return (res, Some(r));
        }
        (res, None)
    }

    pub fn panel<Res, F: FnOnce(&mut Self, &Style) -> Res>(&mut self, style: &Style, name: &str, opt: WidgetOption, f: F) -> Res {
        self.begin_panel_ex(style, name, opt);
        let r = f(self, style);
        self.end_panel();
        r
    }

    pub fn frame<Res, F: FnOnce(&mut Self) -> Res>(&mut self, width: usize, height: usize, f: F) -> (P, Res) {
        self.begin(width, height);
        let r = f(self);
        let p = self.end();
        (p, r)
    }

    pub fn column<Res, F: FnOnce(&mut Self, &Style) -> Res>(&mut self, style: &Style, f: F) -> Res {
        self.top_container_mut().begin_column(&style);
        let r = f(self, style);
        self.top_container_mut().end_column();
        r
    }

    pub fn rows_with_line_config<Res, F: FnOnce(&mut Self, &Style) -> Res>(&mut self, style: &Style, widths: &[i32], height: i32, f: F) -> Res {
        self.top_container_mut().row_config(widths, height);
        let res = f(self, style);
        self.top_container_mut().end_row();
        res
    }

    pub fn next_cell(&mut self, style: &Style) -> Recti {
        self.top_container_mut().next_cell(style)
    }

    pub fn layout_mut(&mut self) -> &mut Layout {
        self.top_container_mut().layout_mut()
    }

    pub fn top_container_mut(&mut self) -> &mut Container<P> {
        &mut self.containers[self.container_stack.last().unwrap().0]
    }

    pub fn header<Res, F: FnOnce(&mut Self, &Style) -> Res>(&mut self, style: &Style, label: &str, opt: WidgetOption, f: F) -> (ResourceState, Option<Res>) {
        let res = self.header_internal(style, style.bold_font, label, false, opt);
        if res.is_active() && self.last_id.is_some() {
            return (res, Some(f(self, style)));
        }
        (res, None)
    }

    pub fn render_custom<F: FnOnce(&mut P, &Recti)>(&mut self, f: F) {
        // first flush everything
        self.renderer.flush();

        // get the viewport
        let clip = self.clip_stack.last().unwrap();
        let mut queue = P::default();
        f(&mut queue, clip);
        self.push_command(Command::DirectRenderPassCommands { pass: queue });
    }
}
