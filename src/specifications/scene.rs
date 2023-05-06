//  SCENE.rs
//    by Lut99
// 
//  Created:
//    23 Apr 2023, 11:40:52
//  Last edited:
//    05 May 2023, 11:44:34
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the scene file.
// 

use enum_debug::EnumDebug;
use serde::{Deserialize, Serialize};

use crate::common::file::impl_file;
use crate::specifications::objects::Sphere;
use crate::specifications::materials::{Diffuse, NormalMap};


/***** AUXILLARY *****/
/// Defines an abstraction over objects that makes it more intuitive for the user to pass them.
#[derive(Clone, Debug, Deserialize, EnumDebug, Serialize)]
pub enum Object {
    // Normal objects
    /// A perfect sphere.
    Sphere(Sphere<Material>),

    // Represents a group of objects.
    Group(Vec<Self>),
}

/// Defines an abstraction over materials that we can use to parse objects independently from sphere.
#[derive(Clone, Copy, Debug, Deserialize, EnumDebug, Serialize)]
pub enum Material {
    // Basic materials
    /// A non-lighted normal map.
    NormalMap(NormalMap),

    // Diffuse materials
    /// The basic diffuse material.
    Diffuse(Diffuse),
}





/***** LIBRARY *****/
/// The SceneFile defines the scene's file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SceneFile {
    /// The objects found in this scene.
    pub objects : Vec<Object>,
}
impl_file!(SceneFile, serde_yaml);
