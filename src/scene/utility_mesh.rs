use crate::renderer::*;
use crate::rs_math3d::*;
use crate::*;
use std::ffi::c_void;
use std::ops::*;

static VERTEX_SHADER: &'static str = "
#version 300 es
in          vec4    position;
in lowp     vec4    color;

uniform     mat4    pvm;

out lowp    vec4    v_color;

void main() {
    gl_Position     = pvm * vec4(position.xyz, 1.0);
    v_color         = color;
}";

static PIXEL_SHADER: &'static str = "
#version 300 es
precision mediump float;

in lowp     vec4   v_color;

layout(location = 0) out lowp vec4    color_buffer;

void main() {
    color_buffer    = v_color;
}";

render_data! {
    vertex Vertex {
        position: Vec3f,
        color   : Color4b,
    }

    uniforms Uniforms {
        pvm     : Mat4f,
    }
}

impl std::ops::Mul<Vertex> for Mat4f {
    type Output = Vertex;
    fn mul(self, rhs: Vertex) -> Self::Output {
        Vertex {
            position: transform_vec3(&self, &rhs.position),
            color: rhs.color,
        }
    }
}

impl std::ops::Add<Vertex> for Vertex {
    type Output = Vertex;
    fn add(self, rhs: Vertex) -> Self::Output {
        Vertex {
            position: self.position + rhs.position,
            color: rhs.color,
        }
    }
}

impl std::ops::Sub<Vertex> for Vertex {
    type Output = Vertex;
    fn sub(self, rhs: Vertex) -> Self::Output {
        Vertex {
            position: self.position - rhs.position,
            color: rhs.color,
        }
    }
}

impl std::ops::Mul<f32> for Vertex {
    type Output = Vertex;
    fn mul(self, rhs: f32) -> Self::Output {
        Vertex {
            position: self.position * rhs,
            color: self.color,
        }
    }
}

impl std::ops::Mul<Vertex> for f32 {
    type Output = Vertex;
    fn mul(self, rhs: Vertex) -> Self::Output {
        Vertex {
            position: rhs.position * self,
            color: rhs.color,
        }
    }
}

impl std::ops::Div<f32> for Vertex {
    type Output = Vertex;
    fn div(self, rhs: f32) -> Self::Output {
        Vertex {
            position: self.position / rhs,
            color: self.color,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Segment {
    verts: [Vertex; 2],
}

impl Segment {
    pub fn new(start: &Vec3f, end: &Vec3f, color: &Color4b) -> Self {
        Self {
            verts: [
                Vertex {
                    position: *start,
                    color: *color,
                },
                Vertex {
                    position: *end,
                    color: *color,
                },
            ],
        }
    }

    pub fn start(&self) -> &Vec3f {
        &self.verts[0].position
    }
    pub fn end(&self) -> &Vec3f {
        &self.verts[1].position
    }

    pub fn with_color(mut self, color: &Color4b) -> Self {
        self.verts[0].color = *color;
        self.verts[1].color = *color;
        self
    }
}

impl Index<usize> for Segment {
    type Output = Vec3f;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.verts[idx].position
    }
}

impl std::ops::Mul<Segment> for Mat4f {
    type Output = Segment;
    fn mul(self, rhs: Segment) -> Self::Output {
        let v0 = self.clone() * rhs.verts[0];
        let v1 = self * rhs.verts[1];
        Segment::new(&v0.position, &v1.position, &rhs.verts[0].color)
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Triangle {
    verts: [Vertex; 3],
}

impl Triangle {
    pub fn new(v0: &Vec3f, v1: &Vec3f, v2: &Vec3f, color: &Color4b) -> Self {
        Self {
            verts: [
                Vertex {
                    position: *v0,
                    color: *color,
                },
                Vertex {
                    position: *v1,
                    color: *color,
                },
                Vertex {
                    position: *v2,
                    color: *color,
                },
            ],
        }
    }

    pub fn v0(&self) -> &Vec3f {
        &self.verts[0].position
    }
    pub fn v1(&self) -> &Vec3f {
        &self.verts[1].position
    }
    pub fn v2(&self) -> &Vec3f {
        &self.verts[2].position
    }
    pub fn with_color(mut self, color: &Color4b) -> Self {
        self.verts[0].color = *color;
        self.verts[1].color = *color;
        self.verts[2].color = *color;
        self
    }
}

impl Index<usize> for Triangle {
    type Output = Vec3f;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.verts[idx].position
    }
}

impl std::ops::Mul<Triangle> for Mat4f {
    type Output = Triangle;
    fn mul(self, rhs: Triangle) -> Self::Output {
        let v0 = self.clone() * rhs.verts[0];
        let v1 = self.clone() * rhs.verts[1];
        let v2 = self.clone() * rhs.verts[2];
        Triangle::new(
            &v0.position,
            &v1.position,
            &v2.position,
            &rhs.verts[0].color,
        )
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Quad {
    verts: [Vertex; 6],
}

impl Quad {
    pub fn new(v0: &Vec3f, v1: &Vec3f, v2: &Vec3f, v3: &Vec3f, color: &Color4b) -> Self {
        Self {
            verts: [
                Vertex {
                    position: *v0,
                    color: *color,
                },
                Vertex {
                    position: *v1,
                    color: *color,
                },
                Vertex {
                    position: *v2,
                    color: *color,
                },
                Vertex {
                    position: *v2,
                    color: *color,
                },
                Vertex {
                    position: *v3,
                    color: *color,
                },
                Vertex {
                    position: *v0,
                    color: *color,
                },
            ],
        }
    }

    pub fn v0(&self) -> &Vec3f {
        &self.verts[0].position
    }
    pub fn v1(&self) -> &Vec3f {
        &self.verts[1].position
    }
    pub fn v2(&self) -> &Vec3f {
        &self.verts[2].position
    }
    pub fn v3(&self) -> &Vec3f {
        &self.verts[4].position
    }

    pub fn with_color(mut self, color: &Color4b) -> Self {
        self.verts[0].color = *color;
        self.verts[1].color = *color;
        self.verts[2].color = *color;
        self.verts[3].color = *color;
        self.verts[4].color = *color;
        self.verts[5].color = *color;
        self
    }
}

impl Index<usize> for Quad {
    type Output = Vec3f;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.verts[idx].position
    }
}

impl std::ops::Mul<Quad> for Mat4f {
    type Output = Quad;
    fn mul(self, rhs: Quad) -> Self::Output {
        let v0 = self.clone() * rhs.verts[0];
        let v1 = self.clone() * rhs.verts[1];
        let v2 = self.clone() * rhs.verts[2];
        let v3 = self.clone() * rhs.verts[4];
        Quad::new(
            &v0.position,
            &v1.position,
            &v2.position,
            &v3.position,
            &rhs.verts[0].color,
        )
    }
}

#[derive(Clone)]
pub enum UMNode {
    Segments(Vec<Segment>),
    Tris(Vec<Triangle>),
    Quads(Vec<Quad>),
    Assembly(Vec<UMNode>),
}

impl std::ops::Mul<UMNode> for Mat4f {
    type Output = UMNode;
    fn mul(self, rhs: UMNode) -> Self::Output {
        match rhs {
            UMNode::Segments(arr) => {
                let a = arr.into_iter().map(|e| self * e).collect();
                UMNode::Segments(a)
            }
            UMNode::Tris(arr) => {
                let a = arr.into_iter().map(|e| self * e).collect();
                UMNode::Tris(a)
            }
            UMNode::Quads(arr) => {
                let a = arr.into_iter().map(|e| self * e).collect();
                UMNode::Quads(a)
            }
            UMNode::Assembly(arr) => {
                let a = arr.into_iter().map(|e| self * e).collect();
                UMNode::Assembly(a)
            }
        }
    }
}

impl UMNode {
    pub fn intersect_ray(&self, ray: &Ray3f) -> Option<Vec3f> {
        match self {
            UMNode::Segments(_) => None,
            UMNode::Tris(tris) => {
                for t in tris {
                    let t3 = Tri3::new([*t.v0(), *t.v1(), *t.v2()]);
                    match ray.intersection(&t3) {
                        Some((_, p)) => return Some(p),
                        _ => (),
                    }
                }
                None
            }

            UMNode::Quads(quads) => {
                for q in quads {
                    let t0 = Tri3::new([*q.v0(), *q.v1(), *q.v2()]);
                    match ray.intersection(&t0) {
                        Some((_, p)) => return Some(p),
                        _ => (),
                    };
                    let t1 = Tri3::new([*q.v2(), *q.v3(), *q.v0()]);
                    match ray.intersection(&t1) {
                        Some((_, p)) => return Some(p),
                        _ => (),
                    };
                }
                None
            }
            UMNode::Assembly(nodes) => {
                for n in nodes {
                    if let Some(p) = n.intersect_ray(ray) {
                        return Some(p);
                    }
                }
                None
            }
        }
    }

    pub fn circle(center: &Vec3f, normal: &Vec3f, color: &Color4b, seg_count: usize) -> Self {
        let step = 2.0 * std::f32::consts::PI / (seg_count as f32);
        let scale = normal.length();

        let [_, y_axis, x_axis] = basis_from_unit(&normal);

        let mut segs = Vec::new();
        for i in 0..seg_count {
            let angle = (i as f32) * step;
            let c = f32::cos(angle);
            let s = f32::sin(angle);

            let p0 = (x_axis * c + y_axis * s) * scale + *center;

            let angle = ((i + 1) as f32) * step;
            let c = f32::cos(angle);
            let s = f32::sin(angle);

            let p1 = (x_axis * c + y_axis * s) * scale + *center;

            segs.push(Segment::new(&p0, &p1, color));
        }

        Self::Segments(segs)
    }

    fn disk_tris(
        center: &Vec3f,
        normal: &Vec3f,
        color: &Color4b,
        seg_count: usize,
        tris: &mut Vec<Triangle>,
    ) {
        let step = 2.0 * std::f32::consts::PI / (seg_count as f32);
        let scale = normal.length();
        let [_, y_axis, x_axis] = basis_from_unit(&normal);

        for i in 0..seg_count {
            let angle = (i as f32) * step;
            let c = f32::cos(angle);
            let s = f32::sin(angle);

            let p0 = (x_axis * c + y_axis * s) * scale + *center;

            let angle = ((i + 1) as f32) * step;
            let c = f32::cos(angle);
            let s = f32::sin(angle);

            let p1 = (x_axis * c + y_axis * s) * scale + *center;

            tris.push(Triangle::new(center, &p0, &p1, color));
        }
    }

    pub fn disk(center: &Vec3f, normal: &Vec3f, color: &Color4b, seg_count: usize) -> Self {
        let mut tris = Vec::new();
        Self::disk_tris(center, normal, color, seg_count, &mut tris);

        Self::Tris(tris)
    }

    pub fn cone(
        center: &Vec3f,
        normal: &Vec3f,
        height: f32,
        color: &Color4b,
        seg_count: usize,
    ) -> Self {
        let scale = normal.length();

        let mut tris = Vec::new();
        Self::disk_tris(center, &-*normal, color, seg_count, &mut tris);

        let [_, y_axis, x_axis] = basis_from_unit(&normal);
        let step = 2.0 * std::f32::consts::PI / (seg_count as f32);

        for i in 0..seg_count {
            let angle = (i as f32) * step;
            let c = f32::cos(angle);
            let s = f32::sin(angle);

            let p0 = (x_axis * c + y_axis * s) * scale + *center;

            let angle = ((i + 1) as f32) * step;
            let c = f32::cos(angle);
            let s = f32::sin(angle);

            let p1 = (x_axis * c + y_axis * s) * scale + *center;

            tris.push(Triangle::new(
                &(*center + Vec3f::normalize(normal) * height),
                &p0,
                &p1,
                color,
            ));
        }

        Self::Tris(tris)
    }

    //     v0                  v1
    //      +--------+--------+
    //      |        ^        |
    //      |        | y_axis |
    //      |       c|        |
    //      +--------+------->+ x_axis
    //      |        |        |
    //      |        |        |
    //      |        |        |
    //   v3 +--------+--------+ v2

    fn plane_quad(
        center: &Vec3f,
        x_axis: &Vec3f,
        y_axis: &Vec3f,
        color: &Color4b,
        quads: &mut Vec<Quad>,
    ) {
        let v0 = *center - *x_axis + *y_axis;
        let v1 = *center + *x_axis + *y_axis;
        let v2 = *center + *x_axis - *y_axis;
        let v3 = *center - *x_axis - *y_axis;

        // CCW direction
        quads.push(Quad::new(&v0, &v1, &v2, &v3, color));
    }

    pub fn plane(center: &Vec3f, x_axis: &Vec3f, y_axis: &Vec3f, color: &Color4b) -> Self {
        let mut quads = Vec::new();
        Self::plane_quad(center, &(-*x_axis), y_axis, color, &mut quads);
        Self::Quads(quads)
    }

    pub fn cube(min: &Vec3f, max: &Vec3f, color: &Color4b) -> Self {
        let center = (*min + *max) * 0.5;
        let extent = *max - center;
        let x_axis = Vec3f::new(extent.x, 0.0, 0.0);
        let y_axis = Vec3f::new(0.0, extent.y, 0.0);
        let z_axis = Vec3f::new(0.0, 0.0, extent.z);

        let mut quads = Vec::new();

        // Z-axis
        Self::plane_quad(&(center + z_axis), &(-x_axis), &y_axis, color, &mut quads);
        Self::plane_quad(&(center - z_axis), &x_axis, &y_axis, color, &mut quads);

        // X-axis
        Self::plane_quad(&(center - x_axis), &(-z_axis), &y_axis, color, &mut quads);
        Self::plane_quad(&(center + x_axis), &z_axis, &y_axis, color, &mut quads);

        // Y-axis
        Self::plane_quad(&(center + y_axis), &(-z_axis), &x_axis, color, &mut quads);
        Self::plane_quad(&(center - y_axis), &z_axis, &x_axis, color, &mut quads);

        Self::Quads(quads)
    }

    pub fn cube_basis(axis: &[Vec3f; 3], min: &Vec3f, max: &Vec3f, color: &Color4b) -> Self {
        let center = (*min + *max) * 0.5;
        let extent = *max - center;
        let x_axis = extent.x * axis[0];
        let y_axis = extent.y * axis[1];
        let z_axis = extent.z * axis[2];

        let mut quads = Vec::new();

        // Z-axis
        Self::plane_quad(&(center + z_axis), &(-x_axis), &y_axis, color, &mut quads);
        Self::plane_quad(&(center - z_axis), &x_axis, &y_axis, color, &mut quads);

        // X-axis
        Self::plane_quad(&(center - x_axis), &(-z_axis), &y_axis, color, &mut quads);
        Self::plane_quad(&(center + x_axis), &z_axis, &y_axis, color, &mut quads);

        // Y-axis
        Self::plane_quad(&(center + y_axis), &(-z_axis), &x_axis, color, &mut quads);
        Self::plane_quad(&(center - y_axis), &z_axis, &x_axis, color, &mut quads);

        Self::Quads(quads)
    }

    pub fn arrow_cone(start: &Vec3f, end: &Vec3f, cone_pct: f32, color: &Color4b) -> Self {
        let seg = *end - *start;
        let tip_normal = seg * cone_pct * 0.5;
        let tip_start = *start + seg * (1.0 - cone_pct);
        let tris = Self::cone(&tip_start, &tip_normal, seg.length() * cone_pct, color, 8);

        let lines = vec![Segment::new(start, &tip_start, color)];

        Self::Assembly(vec![Self::Segments(lines), tris])
    }

    pub fn arrow_box(axis: &[Vec3f; 3], start: &Vec3f, end: &Vec3f, color: &Color4b) -> Self {
        let seg = *end - *start;
        let tip_len = (seg * 0.1).length();
        let tip_extent = Vec3f::new(tip_len, tip_len, tip_len);
        let tip_end = *end;
        let tip_start = *start + seg * 0.8;

        let min = tip_start + seg * 0.1 - tip_extent;
        let max = tip_end - seg * 0.1 + tip_extent;

        let tris = Self::cube_basis(axis, &min, &max, color);

        let lines = vec![Segment::new(start, &tip_start, color)];

        Self::Assembly(vec![Self::Segments(lines), tris])
    }

    pub fn arrow_sphere(start: &Vec3f, end: &Vec3f, cone_pct: f32, color: &Color4b) -> Self {
        let seg = *end - *start;
        let tip_start = *start + seg * 0.8;
        let tris = Self::sphere(&tip_start, cone_pct, 3, color);

        let lines = vec![Segment::new(start, &tip_start, color)];

        Self::Assembly(vec![Self::Segments(lines), tris])
    }

    pub fn basis_cone(center: &Vec3f, x_axis: &Vec3f, y_axis: &Vec3f, z_axis: &Vec3f) -> Self {
        let x = Self::arrow_cone(center, x_axis, 0.4, &color4b(0x7F, 0x00, 0x00, 0xFF));
        let y = Self::arrow_cone(center, y_axis, 0.4, &color4b(0x00, 0x7F, 0x00, 0xFF));
        let z = Self::arrow_cone(center, z_axis, 0.4, &color4b(0x00, 0x00, 0x7F, 0xFF));

        Self::Assembly(vec![x, y, z])
    }

    pub fn basis_box(center: &Vec3f, x_axis: &Vec3f, y_axis: &Vec3f, z_axis: &Vec3f) -> Self {
        let axis = [*x_axis, *y_axis, *z_axis];
        let x = Self::arrow_box(&axis, center, x_axis, &color4b(0x7F, 0x00, 0x00, 0xFF));
        let y = Self::arrow_box(&axis, center, y_axis, &color4b(0x00, 0x7F, 0x00, 0xFF));
        let z = Self::arrow_box(&axis, center, z_axis, &color4b(0x00, 0x00, 0x7F, 0xFF));

        Self::Assembly(vec![x, y, z])
    }

    fn subdivide_quad(q: &Quad) -> [Quad; 4] {
        //     v0       v01        v1
        //      +--------+--------+
        //      |        ^        |
        //      |        | y_axis |
        //      |       c|        | v12
        //  v30 +--------+------->+ x_axis
        //      |        |        |
        //      |        |        |
        //      |        |        |
        //   v3 +--------+--------+ v2
        //              v23
        let v0 = q.v0();
        let v1 = q.v1();
        let v2 = q.v2();
        let v3 = q.v3();

        let vc = (*v0 + *v1 + *v2 + *v3) / 4.0;
        let v01 = (*v0 + *v1) / 2.0;
        let v12 = (*v1 + *v2) / 2.0;
        let v23 = (*v2 + *v3) / 2.0;
        let v30 = (*v3 + *v0) / 2.0;

        let color = q.verts[0].color;
        let q0 = Quad::new(v0, &v01, &vc, &v30, &color);
        let q1 = Quad::new(v1, &v12, &vc, &v01, &color);
        let q2 = Quad::new(v2, &v23, &vc, &v12, &color);
        let q3 = Quad::new(v3, &v30, &vc, &v23, &color);
        [q0, q1, q2, q3]
    }

    fn project_quad_to_sphere(q: &Quad, center: &Vec3f, radius: f32) -> Quad {
        let v0 = q.v0();
        let v1 = q.v1();
        let v2 = q.v2();
        let v3 = q.v3();

        let vp0 = *center + (*v0 - *center).normalize() * radius;
        let vp1 = *center + (*v1 - *center).normalize() * radius;
        let vp2 = *center + (*v2 - *center).normalize() * radius;
        let vp3 = *center + (*v3 - *center).normalize() * radius;

        Quad::new(&vp0, &vp1, &vp2, &vp3, &q.verts[0].color)
    }

    pub fn sphere(center: &Vec3f, radius: f32, subdiv: usize, color: &Color4b) -> Self {
        let min = Vec3f::new(-1.0, -1.0, -1.0) + *center;
        let max = Vec3f::new(1.0, 1.0, 1.0) + *center;

        let cube = Self::cube(&min, &max, color);
        match cube {
            UMNode::Quads(quads) => {
                // This is not optimized and does a lot of allocations!
                // It's not embedded, and simplicity/clarity is above performance
                // TODO: simplify more?
                let mut sp_quads = quads;

                for _ in 0..subdiv {
                    let mut sqs = Vec::new();
                    for q in sp_quads.iter() {
                        let qs = Self::subdivide_quad(q);
                        for qq in qs.iter() {
                            sqs.push(qq.clone());
                        }
                    }

                    sp_quads = sqs;
                }

                let mut final_quads = Vec::new();

                for q in sp_quads.iter() {
                    final_quads.push(Self::project_quad_to_sphere(q, center, radius));
                }
                Self::Quads(final_quads)
            }
            _ => unreachable!(),
        }
    }

    pub fn grid_xy(center: &Vec3f, length: f32, steps: u32) -> Self {
        let mut segs = Vec::new();
        let start = -length / 2.0;
        let end = length / 2.0;

        let step_f = (end - start) / (steps as f32);
        let light_grey = color4b(0xFF, 0xFF, 0xFF, 0xFF);
        let dark_grey = color4b(0x3F, 0x3F, 0x3F, 0xFF);
        let s_x = center.clone() + Vec3f::new(start, -length / 2.0, 0.0);
        let e_x = center.clone() + Vec3f::new(start, length / 2.0, 0.0);

        let s_y = center.clone() + Vec3f::new(-length / 2.0, start, 0.0);
        let e_y = center.clone() + Vec3f::new(length / 2.0, start, 0.0);

        for x in 0..(steps + 1) {
            let color = if x % 10 != 0 { light_grey } else { dark_grey };
            let start = s_x + Vec3f::new((x as f32) * step_f, 0.0, 0.0);
            let end = e_x + Vec3f::new((x as f32) * step_f, 0.0, 0.0);
            segs.push(Segment::new(&start, &end, &color));
        }

        for y in 0..(steps + 1) {
            let color = if y % 10 != 0 { light_grey } else { dark_grey };
            let start = s_y + Vec3f::new(0.0, (y as f32) * step_f, 0.0);
            let end = e_y + Vec3f::new(0.0, (y as f32) * step_f, 0.0);
            segs.push(Segment::new(&start, &end, &color));
        }

        Self::Segments(segs)
    }

    pub fn grid_xz(center: &Vec3f, length: f32, steps: u32) -> Self {
        let mut segs = Vec::new();
        let start = -length / 2.0;
        let end = length / 2.0;

        let step_f = (end - start) / (steps as f32);
        let light_grey = color4b(0xFF, 0xFF, 0xFF, 0xFF);
        let dark_grey = color4b(0x3F, 0x3F, 0x3F, 0xFF);
        let s_x = center.clone() + Vec3f::new(start, 0.0, -length / 2.0);
        let e_x = center.clone() + Vec3f::new(start, 0.0, length / 2.0);

        let s_z = center.clone() + Vec3f::new(-length / 2.0, 0.0, start);
        let e_z = center.clone() + Vec3f::new(length / 2.0, 0.0, start);

        for x in 0..(steps + 1) {
            let color = if x % 10 != 0 { light_grey } else { dark_grey };
            let start = s_x + Vec3f::new((x as f32) * step_f, 0.0, 0.0);
            let end = e_x + Vec3f::new((x as f32) * step_f, 0.0, 0.0);
            segs.push(Segment::new(&start, &end, &color));
        }

        for z in 0..(steps + 1) {
            let color = if z % 10 != 0 { light_grey } else { dark_grey };
            let start = s_z + Vec3f::new(0.0, 0.0, (z as f32) * step_f);
            let end = e_z + Vec3f::new(0.0, 0.0, (z as f32) * step_f);
            segs.push(Segment::new(&start, &end, &color));
        }

        Self::Segments(segs)
    }
}

pub struct Renderer {
    driver: DriverPtr,
    wire_pipeline: PipelinePtr,
    solid_pipeline: PipelinePtr,

    max_verts: usize,
    vb: DeviceBufferPtr,
}

impl Renderer {
    pub fn new(driver: &mut DriverPtr, max_verts: usize) -> Self {
        let mut model_attribs = Vec::new();
        model_attribs.push(Vertex::get_attribute_names());

        let model_shader_desc = ShaderDesc {
            vertex_shader: String::from(VERTEX_SHADER),
            pixel_shader: String::from(PIXEL_SHADER),

            vertex_attributes: model_attribs,
            vertex_uniforms: vec![String::from("pvm")],
            vertex_surfaces: Vec::new(),

            pixel_uniforms: Vec::new(),
            pixel_surfaces: Vec::new(),
        };

        let model_program = driver.create_shader(model_shader_desc).unwrap();

        let vertex_layout = VertexBufferLayout {
            buffer_id: 0,
            vertex_attributes: Vertex::get_attribute_descriptors(),
            stride: Vertex::stride(),
            divisor: 0,
        };

        let solid_pipeline_desc = PipelineDesc {
            primitive_type: PrimitiveType::Triangles,
            shader: model_program.clone(),
            buffer_layouts: vec![vertex_layout.clone()],
            uniform_descs: vec![UniformDataDesc::new(
                String::from("pvm"),
                UniformDataType::Float4x4,
                1,
                0,
            )],
            index_type: IndexType::None,
            face_winding: FaceWinding::CCW,
            cull_mode: CullMode::None,
            depth_write: true,
            depth_test: true,
            blend: BlendOp::Add(Blend::default()),
        };

        let solid_pipeline = driver.create_pipeline(solid_pipeline_desc).unwrap();

        let wire_pipeline_desc = PipelineDesc {
            primitive_type: PrimitiveType::Lines,
            shader: model_program.clone(),
            buffer_layouts: vec![vertex_layout.clone()],
            uniform_descs: vec![UniformDataDesc::new(
                String::from("pvm"),
                UniformDataType::Float4x4,
                1,
                0,
            )],
            index_type: IndexType::None,
            face_winding: FaceWinding::CCW,
            cull_mode: CullMode::None,
            depth_write: true,
            depth_test: true,
            blend: BlendOp::Add(Blend::default()),
        };

        let wire_pipeline = driver.create_pipeline(wire_pipeline_desc).unwrap();

        let vb_desc = DeviceBufferDesc::Vertex(Usage::new_dynamic::<Vertex>(max_verts));
        let vb = driver.create_device_buffer(vb_desc).unwrap();

        Self {
            driver: driver.clone(),
            wire_pipeline: wire_pipeline,
            solid_pipeline: solid_pipeline,
            max_verts: max_verts,
            vb: vb,
        }
    }

    fn draw_chunks<T>(
        &mut self,
        pipeline: &PipelinePtr,
        pvm: &Mat4f,
        chunk_size: usize,
        elems: &Vec<T>,
        count_mul: usize,
    ) {
        let mut rem_elms = elems.len();
        let mut i = 0;

        while rem_elms != 0 {
            let start_chnk_idx = i * chunk_size;
            let count = usize::min(elems.len() - start_chnk_idx, chunk_size);
            let pl = &elems[start_chnk_idx..start_chnk_idx + count];

            self.driver.update_device_buffer(&mut self.vb, 0, &pl);
            let bindings = Bindings {
                vertex_buffers: vec![self.vb.clone()],
                index_buffer: None,

                vertex_images: Vec::new(),
                pixel_images: Vec::new(),
            };

            self.driver.draw(
                pipeline,
                &bindings,
                pvm as *const _ as *const c_void,
                (count * count_mul) as u32,
                1,
            );
            i += 1;
            rem_elms -= count;
        }
    }
    pub fn draw_segments(&mut self, pvm: &Mat4f, lines: &Vec<Segment>) {
        let chunk_size = self.max_verts / 2;
        let pipeline = self.wire_pipeline.clone();
        self.draw_chunks(&pipeline, pvm, chunk_size, lines, 1);
    }

    pub fn draw_tris(&mut self, pvm: &Mat4f, tris: &Vec<Triangle>) {
        let chunk_size = self.max_verts / 3;
        let pipeline = self.solid_pipeline.clone();
        self.draw_chunks(&pipeline, pvm, chunk_size, tris, 1);
    }

    pub fn draw_quads(&mut self, pvm: &Mat4f, quads: &Vec<Quad>) {
        let chunk_size = self.max_verts / 6;
        let pipeline = self.solid_pipeline.clone();
        self.draw_chunks(&pipeline, pvm, chunk_size, quads, 2);
    }

    pub fn draw_node(&mut self, pvm: &Mat4f, node: &UMNode) {
        match node {
            UMNode::Segments(segs) => self.draw_segments(pvm, segs),
            UMNode::Tris(tris) => self.draw_tris(pvm, tris),
            UMNode::Quads(quads) => self.draw_quads(pvm, quads),
            UMNode::Assembly(asms) => {
                for n in asms {
                    self.draw_node(pvm, n)
                }
            }
        }
    }
}
