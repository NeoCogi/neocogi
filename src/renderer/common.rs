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
use rs_math3d::*;
use std::sync::*;

pub enum ResourceType {
    DeviceBuffer,
    Texture,
    RenderTarget,
    Shader,
    Pipeline,
    FrameBuffer,
}

#[repr(C)]
pub struct Resource<Desc> {
    res_type: ResourceType,
    res_id: usize,
    desc: Desc,
    depends_on: Option<DriverPtrInternal>, // resources depend on drivers or other resources
}

impl<Desc> Resource<Desc> {
    pub(crate) fn new(
        res_type: ResourceType,
        res_id: usize,
        desc: Desc,
        depends_on: Option<DriverPtrInternal>,
    ) -> Self {
        Self {
            res_type: res_type,
            res_id: res_id,
            desc: desc,
            depends_on: depends_on,
        }
    }
    pub(crate) fn res_id(&self) -> usize {
        self.res_id
    }
    pub fn desc(&self) -> &Desc {
        &self.desc
    }
}

impl<Desc> Drop for Resource<Desc> {
    fn drop(&mut self) {
        match &mut self.depends_on {
            Some(driver) => driver
                .lock()
                .as_deref_mut()
                .unwrap()
                .delete_resource(&self.res_type, self.res_id),
            _ => panic!("No driver!"),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Attributes
////////////////////////////////////////////////////////////////////////////////

pub trait AttributeDataTypeGetter {
    fn get_attribute_type() -> VertexFormat;
}

// i8 type
impl AttributeDataTypeGetter for i8 {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::SByte
    }
}

impl AttributeDataTypeGetter for Vector2<i8> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::SByte2
    }
}

impl AttributeDataTypeGetter for Vector3<i8> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::SByte3
    }
}

impl AttributeDataTypeGetter for Vector4<i8> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::SByte4
    }
}

// u8 type
impl AttributeDataTypeGetter for u8 {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Byte
    }
}

impl AttributeDataTypeGetter for Vector2<u8> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Byte2
    }
}

impl AttributeDataTypeGetter for Vector3<u8> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Byte3
    }
}

impl AttributeDataTypeGetter for Vector4<u8> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Byte4
    }
}

// s16 type
impl AttributeDataTypeGetter for i16 {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Short
    }
}

impl AttributeDataTypeGetter for Vector2<i16> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Short2
    }
}

impl AttributeDataTypeGetter for Vector3<i16> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Short3
    }
}

impl AttributeDataTypeGetter for Vector4<i16> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Short4
    }
}

// f32 type
impl AttributeDataTypeGetter for f32 {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Float
    }
}

impl AttributeDataTypeGetter for Vector2<f32> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Float2
    }
}

impl AttributeDataTypeGetter for Vector3<f32> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Float3
    }
}

impl AttributeDataTypeGetter for Vector4<f32> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Float4
    }
}

// i32 type
impl AttributeDataTypeGetter for i32 {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Int
    }
}

impl AttributeDataTypeGetter for Vector2<i32> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Int2
    }
}

impl AttributeDataTypeGetter for Vector3<i32> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Int3
    }
}

impl AttributeDataTypeGetter for Vector4<i32> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Int4
    }
}

// u32 type
impl AttributeDataTypeGetter for u32 {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::UInt
    }
}

impl AttributeDataTypeGetter for Vector2<u32> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::UInt2
    }
}

impl AttributeDataTypeGetter for Vector3<u32> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::UInt3
    }
}

impl AttributeDataTypeGetter for Vector4<u32> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::UInt4
    }
}

// matrix type
impl AttributeDataTypeGetter for Matrix2<f32> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Float2x2
    }
}

impl AttributeDataTypeGetter for Matrix3<f32> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Float3x3
    }
}

impl AttributeDataTypeGetter for Matrix4<f32> {
    fn get_attribute_type() -> VertexFormat {
        VertexFormat::Float4x4
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Uniforms
////////////////////////////////////////////////////////////////////////////////

pub trait UniformDataTypeGetter {
    fn get_uniform_type() -> UniformDataType;
}

impl UniformDataTypeGetter for u32 {
    fn get_uniform_type() -> UniformDataType {
        UniformDataType::UInt
    }
}

impl UniformDataTypeGetter for Vector2<u32> {
    fn get_uniform_type() -> UniformDataType {
        UniformDataType::UInt2
    }
}

impl UniformDataTypeGetter for Vector3<u32> {
    fn get_uniform_type() -> UniformDataType {
        UniformDataType::UInt3
    }
}

impl UniformDataTypeGetter for Vector4<u32> {
    fn get_uniform_type() -> UniformDataType {
        UniformDataType::UInt4
    }
}

impl UniformDataTypeGetter for i32 {
    fn get_uniform_type() -> UniformDataType {
        UniformDataType::Int
    }
}

impl UniformDataTypeGetter for Vector2<i32> {
    fn get_uniform_type() -> UniformDataType {
        UniformDataType::Int2
    }
}

impl UniformDataTypeGetter for Vector3<i32> {
    fn get_uniform_type() -> UniformDataType {
        UniformDataType::Int3
    }
}

impl UniformDataTypeGetter for Vector4<i32> {
    fn get_uniform_type() -> UniformDataType {
        UniformDataType::Int4
    }
}

impl UniformDataTypeGetter for f32 {
    fn get_uniform_type() -> UniformDataType {
        UniformDataType::Float
    }
}

impl UniformDataTypeGetter for Vector2<f32> {
    fn get_uniform_type() -> UniformDataType {
        UniformDataType::Float2
    }
}

impl UniformDataTypeGetter for Vector3<f32> {
    fn get_uniform_type() -> UniformDataType {
        UniformDataType::Float3
    }
}

impl UniformDataTypeGetter for Vector4<f32> {
    fn get_uniform_type() -> UniformDataType {
        UniformDataType::Float4
    }
}

impl UniformDataTypeGetter for Matrix2<f32> {
    fn get_uniform_type() -> UniformDataType {
        UniformDataType::Float2x2
    }
}

impl UniformDataTypeGetter for Matrix3<f32> {
    fn get_uniform_type() -> UniformDataType {
        UniformDataType::Float3x3
    }
}

impl UniformDataTypeGetter for Matrix4<f32> {
    fn get_uniform_type() -> UniformDataType {
        UniformDataType::Float4x4
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Macros
////////////////////////////////////////////////////////////////////////////////
#[macro_export]
macro_rules! offset_of {
    ($Struct:path, $field:ident) => {{
        // Using a separate function to minimize unhygienic hazards
        // (e.g. unsafety of #[repr(packed)] field borrows).
        // Uncomment `const` when `const fn`s can juggle pointers.
        /*const*/
        fn offset() -> usize {
            let u = std::mem::MaybeUninit::<$Struct>::uninit();
            // Use pattern-matching to avoid accidentally going through Deref.
            let &$Struct { $field: ref f, .. } = unsafe { &*u.as_ptr() };
            let o = (f as *const _ as usize).wrapping_sub(&u as *const _ as usize);
            // Triple check that we are within `u` still.
            assert!((0..=std::mem::size_of_val(&u)).contains(&o));
            o
        }
        offset()
    }};
}

#[macro_export]
macro_rules! render_data {
    () => {};
    (vertex $name:ident { $($field_name:ident: $field_type:ty,)* }) => {
        #[repr(C)]
        #[derive(Debug, Copy, Clone)]
        struct $name {
            $($field_name: $field_type,)*
        }

        impl $crate::renderer::VertexTrait for $name {
            // This is purely an example—not a good one.
            fn get_attribute_descriptors() -> Vec<$crate::renderer::VertexAttributeDesc> {
                vec![$($crate::renderer::VertexAttributeDesc::new(stringify!($field_name).to_string(), <$field_type>::get_attribute_type(), $crate::offset_of!($name, $field_name))),*]
            }

            fn get_attribute_names() -> Vec<String> {
                vec![$(stringify!($field_name).to_string()),*]
            }

            fn stride() -> usize {
                core::mem::size_of::<Self>()
            }
        }
    };

    (vertex $name:ident { $($field_name:ident: $field_type:ty,)* } $($e:tt)*) => {
        $crate::render_data! { vertex $name {
            $($field_name: $field_type,)*
        } }
        $crate::render_data! { $($e)* }
    };

    (pub vertex $name:ident { $($field_name:ident: $field_type:ty,)* }) => {
        #[repr(C)]
        #[derive(Debug, Copy, Clone)]
        pub struct $name {
            $($field_name: $field_type,)*
        }

        impl $crate::renderer::VertexTrait for $name {
            // This is purely an example—not a good one.
            fn get_attribute_descriptors() -> Vec<$crate::VertexAttributeDesc> {
                vec![$($crate::VertexAttributeDesc::new(stringify!($field_name).to_string(), <$field_type>::get_attribute_type(), $crate::offset_of!($name, $field_name))),*]
            }

            fn get_attribute_names() -> Vec<String> {
                vec![$(stringify!($field_name).to_string()),*]
            }

            fn stride() -> usize {
                core::mem::size_of::<Self>()
            }
        }
    };

    (pub vertex $name:ident { $($field_name:ident: $field_type:ty,)* } $($e:tt)*) => {
        $crate::render_data! { pub vertex $name {
            $($field_name: $field_type,)*
        } }
        $crate::render_data! { $($e)* }
    };

    (uniforms $name:ident {
        $($field_name:ident: $field_type:ty,)*
    }) => {
        #[repr(C)]
        #[derive(Debug, Copy, Clone)]
        struct $name {
            $($field_name: $field_type,)*
        }

        impl $crate::renderer::UniformBlockTrait for $name {
            // This is purely an example—not a good one.
            fn get_uniform_descriptors() -> Vec<$crate::renderer::UniformDataDesc> {
                vec![$($crate::renderer::UniformDataDesc::new(stringify!($field_name).to_string(), <$field_type>::get_uniform_type(), 1, $crate::offset_of!($name, $field_name))),*]
            }

            fn get_uniform_names() -> Vec<String> {
                vec![$(stringify!($field_name).to_string()),*]
            }
        }
    };

    (uniforms $name:ident {
        $($field_name:ident: $field_type:ty,)*
    } $($e:tt)*) => {
        $crate::render_data! { uniforms $name {
            $($field_name: $field_type,)*
        } }
        $crate::render_data! { $($e)* }
    };
}

////////////////////////////////////////////////////////////////////////////////
/// VertexAttributeDesc
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum VertexFormat {
    Byte,
    Byte2,
    Byte3,
    Byte4,

    SByte,
    SByte2,
    SByte3,
    SByte4,

    Short,
    Short2,
    Short3,
    Short4,

    Int,
    Int2,
    Int3,
    Int4,

    UInt,
    UInt2,
    UInt3,
    UInt4,

    Float,
    Float2,
    Float3,
    Float4,

    Float2x2,
    Float3x3,
    Float4x4,
}

#[derive(Clone)]
pub struct VertexAttributeDesc {
    name: String,
    format: VertexFormat,
    offset: usize,
}

impl VertexAttributeDesc {
    pub fn new(name: String, format: VertexFormat, offset: usize) -> Self {
        Self {
            name: name,
            format: format,
            offset: offset,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn format(&self) -> VertexFormat {
        self.format.clone()
    }
    pub fn offset(&self) -> usize {
        self.offset
    }
}

pub trait VertexTrait {
    fn get_attribute_descriptors() -> Vec<VertexAttributeDesc>;
    fn get_attribute_names() -> Vec<String>;
    fn stride() -> usize;
}

////////////////////////////////////////////////////////////////////////////////
/// UniformBlock
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum UniformDataType {
    UInt,
    UInt2,
    UInt3,
    UInt4,
    Int,
    Int2,
    Int3,
    Int4,
    Float,
    Float2,
    Float3,
    Float4,
    Float2x2,
    Float3x3,
    Float4x4,
}

#[derive(Clone)]
pub struct UniformDesc {
    name: String,
    format: UniformDataType,
    count: usize,
}

impl UniformDesc {
    pub fn new(name: String, format: UniformDataType, count: usize) -> Self {
        Self {
            name: name,
            format: format,
            count: count,
        }
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn format(&self) -> UniformDataType {
        self.format.clone()
    }
    pub fn count(&self) -> usize {
        self.count
    }
}

#[derive(Clone)]
pub struct UniformDataDesc {
    pub desc: UniformDesc,
    pub offset: usize,
}

impl UniformDataDesc {
    pub fn new(name: String, format: UniformDataType, count: usize, offset: usize) -> Self {
        Self {
            desc: UniformDesc::new(name, format, count),
            offset: offset,
        }
    }
    pub fn offset(&self) -> usize {
        self.offset
    }
    pub fn desc(&self) -> &UniformDesc {
        &self.desc
    }
}

pub trait UniformBlockTrait {
    fn get_uniform_descriptors() -> Vec<UniformDataDesc>;
    fn get_uniform_names() -> Vec<String>;
}

////////////////////////////////////////////////////////////////////////////////
/// Buffers
////////////////////////////////////////////////////////////////////////////////

pub trait Payload: Send + Sync {
    fn ptr(&self) -> *const u8;
    fn size(&self) -> usize;
}

impl<T: Send + Sync> Payload for Vec<T> {
    fn ptr(&self) -> *const u8 {
        self.as_ptr() as *const u8
    }
    fn size(&self) -> usize {
        ::core::mem::size_of::<T>() * self.len()
    }
}

pub struct GenPayload<T: Sized + Send + Sync> {
    t: T,
}

impl<T: Sized + Send + Sync> Payload for GenPayload<T> {
    fn ptr(&self) -> *const u8 {
        &self.t as *const T as *const u8
    }
    fn size(&self) -> usize {
        ::core::mem::size_of::<T>()
    }
}

impl<T: Sized + Send + Sync> GenPayload<T> {
    pub fn from(t: T) -> Self {
        Self { t }
    }
}

// impl<T: Send + Sync> Payload for &[T] {
//     fn ptr(&self) -> *const u8 { self.as_ptr() as *const u8 }
//     fn size(&self) -> usize { ::core::mem::size_of::<T>() * self.len() }
// }

pub enum Usage {
    Static(Arc<dyn Payload>),
    Dynamic(usize),
    Streamed(usize),
}

impl Usage {
    // pub fn new_static<T>(data: &Vec<T>) -> Usage {
    //     let p = data.as_ptr();
    //     let nd = p as *const u8;
    //     Self::Static(nd, data.len() * std::mem::size_of::<T>())
    // }

    pub fn new_dynamic<T>(len: usize) -> Usage {
        Self::Dynamic(len * std::mem::size_of::<T>())
    }

    pub fn new_streamed<T>(len: usize) -> Usage {
        Self::Dynamic(len * std::mem::size_of::<T>())
    }

    pub fn size(&self) -> usize {
        match self {
            Usage::Static(b) => b.size(),
            Usage::Dynamic(s) => *s,
            Usage::Streamed(s) => *s,
        }
    }
}

pub struct DeviceBufferMapping {
    pub ptr: *mut u8,
    pub offset: usize,
    pub size: usize,
    pub buff: DeviceBufferPtr,
}

pub enum DeviceBufferDesc {
    Vertex(Usage),
    Index(Usage),
    Pixel(Usage),
}

impl DeviceBufferDesc {
    pub fn size(&self) -> usize {
        match self {
            Self::Vertex(u) | Self::Index(u) | Self::Pixel(u) => u.size(),
        }
    }
}

pub type DeviceBuffer = Resource<DeviceBufferDesc>;
pub type DeviceBufferPtr = Arc<DeviceBuffer>;

////////////////////////////////////////////////////////////////////////////////
/// ImageDesc
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy)]
pub enum WrapMode {
    Repeat,
    ClampToEdge,
    ClampToBorder,
    MirroredRepeat,
}

#[derive(Clone)]
pub struct PixelChannel {
    pub size: usize,
    pub wrap: WrapMode,
}

impl PixelChannel {
    pub fn default(size: usize) -> Self {
        Self {
            size: size,
            wrap: WrapMode::Repeat,
        }
    }

    pub fn resize(mut self, size: usize) -> Self {
        self.size = size;
        self
    }

    pub fn with_wrap(mut self, wrap: WrapMode) -> Self {
        self.wrap = wrap;
        self
    }
}

#[derive(Clone)]
pub enum SamplerType {
    Sampler2D(PixelChannel, PixelChannel),
}

#[derive(Clone, Debug)]
pub enum Filter {
    Nearest,
    Linear,
    NearestMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapNearest,
    LinearMipmapLinear,
}

#[derive(Clone, Debug)]
pub enum PixelFormat {
    RGB8U,
    RGBA8U,
    R8U,
    RGB32U,
    RGBA32U,
    R32U,

    RGB32F,
    RGBA32F,
    R32F,

    D16,
    D32,
    D24S8,
    D32S8,

    RGB8(MinMagFilter),
    RGBA8(MinMagFilter),
    R8(MinMagFilter),
}

#[derive(Clone, Debug)]
pub struct MinMagFilter {
    pub min_filter: Filter,
    pub mag_filter: Filter,
}

impl MinMagFilter {
    pub fn default() -> Self {
        Self {
            min_filter: Filter::Nearest,
            mag_filter: Filter::Nearest,
        }
    }

    pub fn with_min_filter(mut self, filter: Filter) -> Self {
        self.min_filter = filter;
        self
    }

    pub fn with_mag_filter(mut self, filter: Filter) -> Self {
        self.mag_filter = filter;
        self
    }
}

#[derive(Debug)]
pub enum OrigSurfaceType {
    UInt,
    Float,
}

pub enum OrigSurfaceClass {
    Color,
    Depth,
}

impl PixelFormat {
    pub fn to_orig_surface_type(&self) -> OrigSurfaceType {
        match self {
            PixelFormat::RGB8U => OrigSurfaceType::UInt,
            PixelFormat::RGBA8U => OrigSurfaceType::UInt,
            PixelFormat::R8U => OrigSurfaceType::UInt,
            PixelFormat::RGB32U => OrigSurfaceType::UInt,
            PixelFormat::RGBA32U => OrigSurfaceType::UInt,
            PixelFormat::R32U => OrigSurfaceType::UInt,

            PixelFormat::RGB32F => OrigSurfaceType::Float,
            PixelFormat::RGBA32F => OrigSurfaceType::Float,
            PixelFormat::R32F => OrigSurfaceType::Float,
            PixelFormat::D16 => OrigSurfaceType::Float,
            PixelFormat::D32 => OrigSurfaceType::Float,
            PixelFormat::D24S8 => OrigSurfaceType::Float,
            PixelFormat::D32S8 => OrigSurfaceType::Float,
            PixelFormat::RGB8(_) => OrigSurfaceType::Float,
            PixelFormat::RGBA8(_) => OrigSurfaceType::Float,
            PixelFormat::R8(_) => OrigSurfaceType::Float,
        }
    }
}

#[derive(Clone)]
pub struct SamplerDesc {
    pub image_type: SamplerType,
    pub mip_maps: usize,
    pub pixel_format: PixelFormat,
}

impl SamplerDesc {
    pub fn default(width: usize, height: usize) -> Self {
        Self {
            image_type: SamplerType::Sampler2D(
                PixelChannel::default(width),
                PixelChannel::default(height),
            ),
            mip_maps: 0,
            pixel_format: PixelFormat::RGBA8U,
        }
    }

    pub fn with_wrap_mode(mut self, wrap: WrapMode) -> Self {
        let image_type = match self.image_type {
            SamplerType::Sampler2D(mut w, mut h) => {
                w.wrap = wrap;
                h.wrap = wrap;
                SamplerType::Sampler2D(w, h)
            }
        };
        self.image_type = image_type;
        self
    }

    pub fn with_pixel_format(mut self, pf: PixelFormat) -> Self {
        self.pixel_format = pf;
        self
    }

    pub fn with_mip_maps(mut self, levels: usize) -> Self {
        self.mip_maps = levels;
        self
    }

    pub fn width(&self) -> usize {
        match self.image_type {
            SamplerType::Sampler2D(PixelChannel { size, wrap: _ }, _) => size,
        }
    }

    pub fn height(&self) -> usize {
        match self.image_type {
            SamplerType::Sampler2D(_, PixelChannel { size, wrap: _ }) => size,
        }
    }
}

pub struct TextureDesc {
    pub sampler_desc: SamplerDesc,
    pub payload: Option<Arc<dyn Payload>>,
}

pub struct RenderTargetDesc {
    pub sampler_desc: SamplerDesc,
    pub sample_count: usize,
}

pub type Texture = Resource<TextureDesc>;
pub type TexturePtr = Arc<Texture>;

pub type RenderTarget = Resource<RenderTargetDesc>;
pub type RenderTargetPtr = Arc<RenderTarget>;

////////////////////////////////////////////////////////////////////////////////
/// ShaderDesc
////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
pub struct ShaderDesc {
    pub vertex_shader: String,
    pub pixel_shader: String,

    pub vertex_attributes: Vec<Vec<String>>,
    pub vertex_uniforms: Vec<String>,
    pub vertex_surfaces: Vec<String>,

    pub pixel_uniforms: Vec<String>,
    pub pixel_surfaces: Vec<String>,
}

unsafe impl Send for ShaderDesc {}
unsafe impl Sync for ShaderDesc {}

pub type Shader = Resource<ShaderDesc>;
pub type ShaderPtr = Arc<Shader>;

////////////////////////////////////////////////////////////////////////////////
/// Binding
////////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Eq, PartialEq)]
pub enum IndexType {
    None,
    UInt16,
    UInt32,
}

pub trait IndexTypeTrait: Send + Sync {
    fn to_index_type() -> IndexType;
}

impl IndexTypeTrait for u16 {
    fn to_index_type() -> IndexType {
        IndexType::UInt16
    }
}

impl IndexTypeTrait for u32 {
    fn to_index_type() -> IndexType {
        IndexType::UInt32
    }
}

#[derive(Clone)]
pub struct Bindings {
    pub vertex_buffers: Vec<DeviceBufferPtr>,
    pub index_buffer: Option<DeviceBufferPtr>,

    pub vertex_images: Vec<TexturePtr>,
    pub pixel_images: Vec<TexturePtr>,
}

////////////////////////////////////////////////////////////////////////////////
/// PipelineDesc
////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
pub enum PrimitiveType {
    Points,
    Lines,
    LineStrip,
    Triangles,
    TriangleStrip,
}

#[derive(Clone)]
pub enum CullMode {
    Winding,
    None,
}

#[derive(Clone)]
pub enum FaceWinding {
    CCW,
    CW,
}

#[derive(Clone)]
pub struct VertexBufferLayout {
    pub buffer_id: usize,
    pub vertex_attributes: Vec<VertexAttributeDesc>,
    pub stride: usize,
    pub divisor: usize,
}

#[derive(Clone)]
pub enum BlendFactor {
    Zero,
    One,

    SrcColor,
    OneMinusSrcColor,
    SrcAlpha,
    OneMinusSrcAlpha,

    DstColor,
    OneMinusDstColor,
    DstAlpha,
    OneMinusDstAlpha,

    SrcAlphaSaturate,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
}

#[derive(Clone)]
pub struct Blend {
    pub src_factor_rgb: BlendFactor,
    pub src_factor_alpha: BlendFactor,

    pub dst_factor_rgb: BlendFactor,
    pub dst_factor_alpha: BlendFactor,
}

impl Blend {
    pub fn default() -> Self {
        Self {
            src_factor_rgb: BlendFactor::One,
            src_factor_alpha: BlendFactor::One,

            dst_factor_rgb: BlendFactor::OneMinusSrcAlpha,
            dst_factor_alpha: BlendFactor::OneMinusSrcAlpha,
        }
    }
}

#[derive(Clone)]
pub enum BlendOp {
    None,
    Add(Blend),
    Subtract(Blend),
    ReverseSubtract(Blend),
}

#[derive(Clone)]
pub enum PolygonOffset {
    None,
    FactorUnits(f32, f32),
}

#[derive(Clone)]
pub struct PipelineDesc {
    pub primitive_type: PrimitiveType,
    pub shader: ShaderPtr,

    // layout
    pub buffer_layouts: Vec<VertexBufferLayout>,

    //
    pub uniform_descs: Vec<UniformDataDesc>,
    pub index_type: IndexType,

    pub face_winding: FaceWinding,
    pub cull_mode: CullMode,

    pub depth_write: bool,
    pub depth_test: bool,

    pub blend: BlendOp,
    pub polygon_offset: PolygonOffset,
}

unsafe impl Send for PipelineDesc {}
unsafe impl Sync for PipelineDesc {}

pub type Pipeline = Resource<PipelineDesc>;
pub type PipelinePtr = Arc<Pipeline>;

////////////////////////////////////////////////////////////////////////////////
/// Pass
////////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Copy)]
pub enum ColorPassAction {
    Clear(Color4b),
    Previous,
}

#[derive(Clone, Copy)]
pub enum DepthPassAction {
    Clear(f32),
    Previous,
}

#[derive(Clone)]
pub enum SurfaceAttachment {
    Texture(TexturePtr),
    RenderTarget(RenderTargetPtr),
}

impl SurfaceAttachment {
    pub fn pixel_format(&self) -> PixelFormat {
        match self {
            SurfaceAttachment::Texture(tex) => tex.desc.sampler_desc.pixel_format.clone(),
            SurfaceAttachment::RenderTarget(rt) => rt.desc.sampler_desc.pixel_format.clone(),
        }
    }
}

#[derive(Clone)]
pub struct FrameBufferDesc {
    pub color_attachements: [Option<SurfaceAttachment>; 4],
    pub depth_stencil_attachement: SurfaceAttachment,
}

unsafe impl Send for FrameBufferDesc {}
unsafe impl Sync for FrameBufferDesc {}

pub type FrameBuffer = Resource<FrameBufferDesc>;
pub type FrameBufferPtr = Arc<FrameBuffer>;

pub(crate) struct DrawCommand {
    pub pipe: PipelinePtr,
    pub bindings: Bindings,
    pub uniforms: Arc<dyn Payload>,
    pub prim_count: u32,
    pub instance_count: u32,
}

pub(crate) struct UpdateDeviceBufferCommand {
    pub buffer: DeviceBufferPtr,
    pub offset: usize,
    pub payload: Arc<dyn Payload>,
}

pub(crate) struct UpdateTextureCommand {
    pub tex: TexturePtr,
    pub payload: Arc<dyn Payload>,
}

pub(crate) enum RenderPassCommand {
    Viewport(i32, i32, u32, u32),
    Scissor(i32, i32, u32, u32),
    Draw(DrawCommand),
    UpdateDeviceBuffer(UpdateDeviceBufferCommand),
    UpdateTexture(UpdateTextureCommand),
}

pub struct Pass {
    pub width: usize,
    pub height: usize,
    pub frame_buffer: Option<FrameBufferPtr>,
    pub color_actions: [ColorPassAction; 4],
    pub depth_action: DepthPassAction,
    pub queue: PassCommandQueue,
}

#[derive(Default)]
pub struct PassCommandQueue {
    pub(crate) commands: Vec<RenderPassCommand>,
}

impl PassCommandQueue {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn set_viewport(&mut self, x: i32, y: i32, w: u32, h: u32) {
        self.commands.push(RenderPassCommand::Viewport(x, y, w, h));
    }

    pub fn set_scissor(&mut self, x: i32, y: i32, w: u32, h: u32) {
        self.commands.push(RenderPassCommand::Scissor(x, y, w, h))
    }

    pub fn draw(
        &mut self,
        pipe: &PipelinePtr,
        bindings: &Bindings,
        uniforms: Arc<dyn Payload>,
        prim_count: u32,
        instance_count: u32,
    ) {
        self.commands.push(RenderPassCommand::Draw(DrawCommand {
            pipe: pipe.clone(),
            bindings: bindings.clone(),
            uniforms,
            prim_count,
            instance_count,
        }));
    }

    pub fn update_device_buffer(
        &mut self,
        dev_buf: &mut DeviceBufferPtr,
        offset: usize,
        pl: Arc<dyn Payload>,
    ) {
        self.commands.push(RenderPassCommand::UpdateDeviceBuffer(
            UpdateDeviceBufferCommand {
                buffer: dev_buf.clone(),
                offset,
                payload: pl,
            },
        ));
    }

    pub fn update_texture(&mut self, tex: &mut TexturePtr, pl: Arc<dyn Payload>) {
        self.commands
            .push(RenderPassCommand::UpdateTexture(UpdateTextureCommand {
                tex: tex.clone(),
                payload: pl,
            }));
    }

    pub fn drain(&mut self) {
        self.commands.clear();
    }

    pub fn append(&mut self, mut other: PassCommandQueue) {
        self.commands.append(&mut other.commands);
    }
}

impl Pass {
    pub fn new(
        width: usize,
        height: usize,
        frame_buffer: Option<FrameBufferPtr>,
        color_actions: [ColorPassAction; 4],
        depth_action: DepthPassAction,
    ) -> Self {
        Self {
            width,
            height,
            frame_buffer,
            color_actions,
            depth_action,
            queue: PassCommandQueue {
                commands: Vec::new(),
            },
        }
    }

    pub fn clone_with_no_commands(&self) -> Self {
        Self {
            queue: PassCommandQueue {
                commands: Vec::new(),
            },
            frame_buffer: self.frame_buffer.clone(),
            ..*self
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Readback surface
////////////////////////////////////////////////////////////////////////////////
pub enum ReadbackPayload {
    RGB32U(Vec<Vector3<u32>>),
    RGBA32U(Vec<Vector4<u32>>),
    R32U(Vec<u32>),

    RGB32F(Vec<Vec3f>),
    RGBA32F(Vec<Vec4f>),
    R32F(Vec<f32>),

    Depth(Vec<f32>),
}

pub enum ReadbackError {
    NoReadbackFromRenderTarget,
    RectOutOfBound,
}

pub enum ReadbackResult {
    Ok(ReadbackPayload),
    Error(ReadbackError),
}

////////////////////////////////////////////////////////////////////////////////
/// Capabilities
////////////////////////////////////////////////////////////////////////////////
#[derive(Copy, Clone)]
pub struct DriverCaps {
    pub max_2d_surface_dimension: Dimensioni,
}

////////////////////////////////////////////////////////////////////////////////
/// Driver
////////////////////////////////////////////////////////////////////////////////
pub trait Driver {
    fn get_caps(&self) -> DriverCaps;
    fn create_device_buffer(&mut self, desc: DeviceBufferDesc) -> Option<DeviceBufferPtr>;
    fn create_texture(&mut self, desc: TextureDesc) -> Option<TexturePtr>;
    fn create_render_target(&mut self, desc: RenderTargetDesc) -> Option<RenderTargetPtr>;
    fn create_shader(&mut self, desc: ShaderDesc) -> Option<ShaderPtr>;
    fn create_pipeline(&mut self, desc: PipelineDesc) -> Option<PipelinePtr>;
    fn create_frame_buffer(&mut self, desc: FrameBufferDesc) -> Option<FrameBufferPtr>;

    fn delete_resource(&mut self, resource_type: &ResourceType, res_id: usize);

    fn render_pass(&mut self, pass: &mut Pass);

    fn read_back(
        &mut self,
        surface: &TexturePtr,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<ReadbackPayload>;
}

//
// There is a very important reason for having this abstraction: Rust mutexes are not reentrant!!!
// This would break mutability borrowing rules: many lock.muts would have multiple owners of mutability.
// to remedy this, we need to abstract the lock and the automatic unlock within the same code block.
// While we are using Driver trait, this should be transparent from the compiler point of view since, we
// are not using DriverPtr in dynamic way
//
pub(crate) type DriverPtrInternal = Arc<Mutex<dyn Driver>>;

#[derive(Clone)]
pub struct DriverPtr {
    driver: DriverPtrInternal,
}

unsafe impl Send for DriverPtr {}
unsafe impl Sync for DriverPtr {}

impl DriverPtr {
    pub fn from(driver: DriverPtrInternal) -> Self {
        Self { driver }
    }
}

impl Driver for DriverPtr {
    fn get_caps(&self) -> DriverCaps {
        self.driver.lock().as_deref_mut().unwrap().get_caps()
    }

    fn create_device_buffer(&mut self, desc: DeviceBufferDesc) -> Option<DeviceBufferPtr> {
        self.driver
            .lock()
            .as_deref_mut()
            .unwrap()
            .create_device_buffer(desc)
    }

    fn create_texture(&mut self, desc: TextureDesc) -> Option<TexturePtr> {
        self.driver
            .lock()
            .as_deref_mut()
            .unwrap()
            .create_texture(desc)
    }

    fn create_render_target(&mut self, desc: RenderTargetDesc) -> Option<RenderTargetPtr> {
        self.driver
            .lock()
            .as_deref_mut()
            .unwrap()
            .create_render_target(desc)
    }

    fn create_shader(&mut self, desc: ShaderDesc) -> Option<ShaderPtr> {
        self.driver
            .lock()
            .as_deref_mut()
            .unwrap()
            .create_shader(desc)
    }

    fn create_pipeline(&mut self, desc: PipelineDesc) -> Option<PipelinePtr> {
        self.driver
            .lock()
            .as_deref_mut()
            .unwrap()
            .create_pipeline(desc)
    }

    fn create_frame_buffer(&mut self, desc: FrameBufferDesc) -> Option<FrameBufferPtr> {
        self.driver
            .lock()
            .as_deref_mut()
            .unwrap()
            .create_frame_buffer(desc)
    }

    fn delete_resource(&mut self, resource_type: &ResourceType, res_id: usize) {
        self.driver
            .lock()
            .as_deref_mut()
            .unwrap()
            .delete_resource(resource_type, res_id)
    }

    fn render_pass(&mut self, pass: &mut Pass) {
        self.driver.lock().as_deref_mut().unwrap().render_pass(pass)
    }

    fn read_back(
        &mut self,
        surface: &TexturePtr,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<ReadbackPayload> {
        self.driver
            .lock()
            .as_deref_mut()
            .unwrap()
            .read_back(surface, x, y, w, h)
    }
}
