
#[cfg(feature="renderer")]
pub mod renderer;

#[cfg(feature="ui")]
pub mod ui;

#[cfg(feature="ui")]
pub mod egui {
    pub use egui::*;
}

pub mod rs_math3d {
    pub use rs_math3d::*;
}

#[cfg(feature="scene")]
pub mod scene;

pub extern crate glfw;

pub enum UpdateResult {
    Handled,
    Unhandled,
}
