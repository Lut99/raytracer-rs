//  SCENE.rs
//    by Lut99
// 
//  Created:
//    23 Apr 2023, 11:40:52
//  Last edited:
//    28 Apr 2023, 11:36:52
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the scene file.
// 

use serde::{Deserialize, Serialize};

use crate::common::file::impl_file;
use crate::math::vec3::Vec3;


/***** LIBRARY *****/
/// Defines the possible objects we can see in a scene.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Object {
    /// A sphere.
    Sphere(Sphere),
}

/// Defines what we know about a Sphere.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Sphere {
    /// The center of the sphere.
    pub center : Vec3,
    /// The radius of the sphere.
    pub radius : f64,
}



/// The SceneFile defines the scene's file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SceneFile {
    /// The objects found in this scene.
    pub objects : Vec<Object>,
}
impl_file!(SceneFile, serde_yaml);
