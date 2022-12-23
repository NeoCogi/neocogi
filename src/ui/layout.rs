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

#[derive(Default, Copy, Clone)]
pub struct Layout {
    pub body: Recti,
    pub next: Recti,
    pub position: Vec2i,
    pub size: Vec2i,
    pub max: Vec2i,
    pub widths: [i32; 16],
    pub items: usize,
    pub item_index: usize,
    pub next_row: i32,
    pub next_type: LayoutPosition,
    pub indent: i32,
}

#[derive(PartialEq, Copy, Clone)]
#[repr(u32)]
pub enum LayoutPosition {
    Absolute = 2,
    Relative = 1,
    None = 0,
}

impl Default for LayoutPosition {
    fn default() -> Self {
        LayoutPosition::None
    }
}

#[derive(Default, Clone)]
pub struct LayoutStack {
    stack: FixedVec<Layout, 16>,
    last_rect: Recti,
}

impl LayoutStack {
    pub fn push(&mut self, body: Recti, scroll: Vec2i) {
        let mut layout: Layout = Layout {
            body: Recti {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            next: Recti {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            position: Vec2i { x: 0, y: 0 },
            size: Vec2i { x: 0, y: 0 },
            max: Vec2i { x: 0, y: 0 },
            widths: [0; 16],
            items: 0,
            item_index: 0,
            next_row: 0,
            next_type: LayoutPosition::None,
            indent: 0,
        };
        layout.body = Rect::new(
            body.x - scroll.x,
            body.y - scroll.y,
            body.width,
            body.height,
        );
        layout.max = vec2(-0x1000000, -0x1000000);
        self.stack.push(layout);
        self.row(&[0], 0);
    }

    pub fn top(&self) -> &Layout {
        return self.stack.top().unwrap();
    }

    pub fn top_mut(&mut self) -> &mut Layout {
        return self.stack.top_mut().unwrap();
    }

    pub fn pop(&mut self) {
        self.stack.pop()
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn begin_column(&mut self, style: &Style) {
        let layout = self.next(style);
        self.push(layout, vec2(0, 0));
    }

    pub fn end_column(&mut self) {
        let b = self.top().clone();
        self.stack.pop();

        // inherit position/next_row/max from child layout if they are greater
        let a = self.top_mut();
        a.position.x = i32::max(a.position.x, b.position.x + b.body.x - a.body.x);
        a.next_row = i32::max(a.next_row, b.next_row + b.body.y - a.body.y);
        a.max.x = i32::max(a.max.x, b.max.x);
        a.max.y = i32::max(a.max.y, b.max.y);
    }

    fn row_for_layout(layout: &mut Layout, widths: &[i32], height: i32) {
        layout.items = widths.len();
        assert!(widths.len() <= 16);
        for i in 0..widths.len() {
            layout.widths[i] = widths[i];
        }
        layout.position = vec2(layout.indent, layout.next_row);
        layout.size.y = height;
        layout.item_index = 0;
    }

    pub fn row(&mut self, widths: &[i32], height: i32) {
        let layout = self.top_mut();
        Self::row_for_layout(layout, widths, height);
    }

    pub fn width(&mut self, width: i32) {
        self.top_mut().size.x = width;
    }

    pub fn height(&mut self, height: i32) {
        self.top_mut().size.y = height;
    }

    pub fn set_next(&mut self, r: Recti, position: LayoutPosition) {
        let layout = self.top_mut();
        layout.next = r;
        layout.next_type = position;
    }

    pub fn next(&mut self, style: &Style) -> Recti {
        let layout = self.top_mut();
        let mut res = Recti::new(0, 0, 0, 0);
        if layout.next_type != LayoutPosition::None {
            let type_0 = layout.next_type;
            layout.next_type = LayoutPosition::None;
            res = layout.next;
            if type_0 == LayoutPosition::Absolute {
                self.last_rect = res;
                return self.last_rect;
            }
        } else {
            // handle next row
            let litems = layout.items;
            let lsize_y = layout.size.y;
            let mut undefined_widths = [0; 16];
            undefined_widths[0..litems as usize]
                .copy_from_slice(&layout.widths[0..litems as usize]);
            if layout.item_index == layout.items {
                Self::row_for_layout(layout, &undefined_widths[0..litems as usize], lsize_y);
            }

            // position
            res.x = layout.position.x;
            res.y = layout.position.y;

            // size
            res.width = if layout.items > 0 {
                layout.widths[layout.item_index as usize]
            } else {
                layout.size.x
            };
            res.height = layout.size.y;
            if res.width == 0 {
                res.width = style.size.x + style.padding * 2;
            }
            if res.height == 0 {
                res.height = style.size.y + style.padding * 2;
            }
            if res.width < 0 {
                res.width += layout.body.width - res.x + 1;
            }
            if res.height < 0 {
                res.height += layout.body.height - res.y + 1;
            }
            layout.item_index += 1;
        }

        // update position
        layout.position.x += res.width + style.spacing;
        layout.next_row = i32::max(layout.next_row, res.y + res.height + style.spacing);

        // apply body offset
        res.x += layout.body.x;
        res.y += layout.body.y;

        // update max position
        layout.max.x = i32::max(layout.max.x, res.x + res.width);
        layout.max.y = i32::max(layout.max.y, res.y + res.height);
        self.last_rect = res;
        return self.last_rect;
    }
}
