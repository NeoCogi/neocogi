mod camera;
mod view3d;
pub mod utility_mesh;

pub use camera::*;
pub use view3d::*;

use rs_math3d::*;

// transform is done as the following: position_tm * rotation_tm * scale_tm * vert_pos
pub struct Node {
    position    : Vec3f,
    rotation    : Quatf,
    scale       : Vec3f,
}
