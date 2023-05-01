//  MOD.rs
//    by Lut99
// 
//  Created:
//    01 May 2023, 18:54:46
//  Last edited:
//    01 May 2023, 19:19:46
//  Auto updated?
//    Yes
// 
//  Description:
//!   The `objects` module defines the objects to which we can render. It
//!   is structured object-oriented _like_, but because we use our
//!   ECS-like [`crate::hitlist::HitList`] and we never turn the objects
//!   into dynamic trait instances, we won't have the downsides of virtual
//!   function pointers.
// 

// Define the submodules
pub mod spec;
pub mod utils;
pub mod group;
pub mod sphere;

// Bring some of this into this namespace
pub use spec::*;
pub use group::ObjectGroup;
pub use sphere::Sphere;


/***** AUXILLARY *****/
/// Defines an enum containing all possible objects.
#[derive(Clone, Debug, serde::Deserialize, enum_debug::EnumDebug, serde::Serialize)]
pub enum Object {
    /// A group of objects we can consider as one.
    ObjectGroup(ObjectGroup),

    /// A perfect sphere.
    Sphere(Sphere),
}

impl BoundingBoxable for Object {
    #[inline]
    fn aabb(&self) -> crate::math::AABB {
        use Object::*;
        match self {
            ObjectGroup(g) => g.aabb(),

            Sphere(s) => s.aabb(),
        }
    }
}
impl Hittable for Object {
    #[inline]
    fn hit(&self, ray: crate::math::Ray) -> Option<HitRecord> {
        use Object::*;
        match self {
            ObjectGroup(_) => { unreachable!(); },

            Sphere(s) => s.hit(ray),
        }
    }
}
