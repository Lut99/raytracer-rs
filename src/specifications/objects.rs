//  OBJECTS.rs
//    by Lut99
// 
//  Created:
//    30 Apr 2023, 10:59:17
//  Last edited:
//    30 Apr 2023, 11:24:07
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the objects known to the raytracer.
// 

use enum_debug::EnumDebug;
use serde::{Deserialize, Serialize};

use crate::math::Vec3;


/***** AUXILLARY *****/
/// Defines an enum containing all possible objects.
#[derive(Clone, Debug, Deserialize, EnumDebug, Serialize)]
pub enum Object {
    /// A group of objects we can consider as one.
    ObjectGroup(ObjectGroup),

    /// A perfect sphere.
    Sphere(Sphere),
}





/***** LIBRARY *****/
/// An ObjectGroup defines a nested list of objects, grouped together for rendering or update purposes.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ObjectGroup {
    /// The list of objects we are grouping.
    pub objects : Vec<Object>,
}



/// Defines a perfect sphere.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Sphere {
    /// The center point of the sphere.
    pub center : Vec3,
    /// The radius of the sphere.
    pub radius : f64,
}
