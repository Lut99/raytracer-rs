//  HITRECORD.rs
//    by Lut99
//
//  Description:
//!   Auxillary struct for remembering where a [`Ray`] hit an [`Object`].
//

use crate::math::{Ray, Vec3};


/***** LIBRARY *****/
/// Defines everything we want to know about a hit.
#[derive(Clone, Copy, Debug)]
pub struct HitRecord {
    /// The (closest) point where the [`Ray`] hits an object.
    pub hit: Vec3,
    /// The (closest) point where the [`Ray`] hits an object, given as distance from the ray's origin.
    pub t: f64,
    /// The surface normal on the point we are hitting.
    pub normal: Vec3,
    /// Whether we are hitting the front face of the object or the backface.
    pub front_face: bool,
    /// An XY-coordinate pair relative to the object (useful for texture mapping).
    pub uv: (f64, f64),
}

// Constructors
impl HitRecord {
    /// Constructor for the HitRecord that compute the internal `hit`, `normal` and `front_face` from the given ray, hit distance on that ray and outward normal.
    ///
    /// # Arguments
    /// - `ray`: The [`Ray`] which hits an object.
    /// - `hit`: The physical point where we hit the object. Probably computed as [`Ray::at()`], but we leave this for the caller since they typically need this point to compute the normal.
    /// - `t`: The distance from the `ray`'s origin, along the ray, which hits the object.
    /// - `outward_normal`: The outward facing normal that we will store but tweaked so it's always in the direction of the `ray`.
    /// - `uv`: An XY-coordinate pair relative to the object (useful for texture mapping).
    ///
    /// # Returns
    /// A new `HitRecord` with the math taken care of.
    pub fn new(ray: Ray, hit: Vec3, t: f64, outward_normal: Vec3, uv: (f64, f64)) -> Self {
        // Compute the normal from the outward normal, remembering the direction
        let front_face: bool = ray.direct.dot(outward_normal) < 0.0;
        let normal: Vec3 = if front_face { outward_normal } else { -outward_normal };

        // Return ourselves
        Self { hit, t, normal, front_face, uv }
    }
}
