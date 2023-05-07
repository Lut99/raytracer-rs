//  SCENE.rs
//    by Lut99
// 
//  Created:
//    23 Apr 2023, 11:40:52
//  Last edited:
//    07 May 2023, 12:43:21
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
use crate::specifications::materials::{Diffuse, NormalMap, StaticColour};


/***** AUXILLARY *****/
/// Helper trait that we use to get some specialization in on retrieving the internal object in the [`Object`] and [`Material`] enums.
pub trait IntoInner<T> {
    /// Returns ourselves as the given type if we are.
    /// 
    /// # Returns
    /// The internal object, or [`None`].
    fn into_inner(self) -> Option<T>;
}



/// Defines an abstraction over objects that makes it more intuitive for the user to pass them.
#[derive(Clone, Debug, Deserialize, EnumDebug, Serialize)]
pub enum Object {
    // Normal objects
    /// A perfect sphere.
    Sphere(Sphere<Material>),

    // Represents a group of objects.
    Group(Vec<Self>),
}

impl<T: Clone> IntoInner<Sphere<T>> for Object where Material: IntoInner<T> {
    #[inline]
    fn into_inner(self) -> Option<Sphere<T>> {
        if let Self::Sphere(s) = self {
            s.material.into_inner().map(|m| {
                Sphere {
                    center : s.center,
                    radius : s.radius,

                    material : m,
                }
            })
        } else {
            None
        }
    }
}



/// Defines an abstraction over materials that we can use to parse objects independently from sphere.
#[derive(Clone, Copy, Debug, Deserialize, EnumDebug, Serialize)]
pub enum Material {
    // Basic materials
    /// A non-lighted static colour.
    StaticColour(StaticColour),
    /// A non-lighted normal map.
    NormalMap(NormalMap),

    // Diffuse materials
    /// The basic diffuse material.
    Diffuse(Diffuse),
}

impl IntoInner<StaticColour> for Material {
    #[inline]
    fn into_inner(self) -> Option<StaticColour> { if let Self::StaticColour(c) = self { Some(c) } else { None } }
}
impl IntoInner<NormalMap> for Material {
    #[inline]
    fn into_inner(self) -> Option<NormalMap> { if let Self::NormalMap(nm) = self { Some(nm) } else { None } }
}

impl IntoInner<Diffuse> for Material {
    #[inline]
    fn into_inner(self) -> Option<Diffuse> { if let Self::Diffuse(d) = self { Some(d) } else { None } }
}





/***** LIBRARY *****/
/// The SceneFile defines the scene's file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SceneFile {
    /// The objects found in this scene.
    pub objects : Vec<Object>,
}
impl_file!(SceneFile, serde_yaml);
