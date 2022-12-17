#[cfg(feature = "renderer")]
pub mod renderer;

#[cfg(feature = "ui")]
pub mod ui;

pub mod rs_math3d {
    pub use rs_math3d::*;
}

#[cfg(feature = "scene")]
pub mod scene;

#[cfg(feature = "ui")]
pub extern crate glfw;

#[cfg(feature = "editor")]
pub mod editor;

pub enum UpdateResult {
    Handled,
    Unhandled,
}
