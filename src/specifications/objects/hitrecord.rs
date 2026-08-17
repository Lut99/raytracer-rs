//  HITRECORD.rs
//    by Lut99
//
//  Description:
//!   Auxillary struct for remembering where a [`Ray`] hit an [`Object`].
//

use super::super::materials::Scattering;
use super::super::scene::Environment;
use crate::math::{Colour, Ray, Vec3};


/***** LIBRARY *****/
/// Defines all the math we want to know about a hit.
#[derive(Clone, Copy, Debug)]
pub struct HitData {
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
impl HitData {
    /// Constructor for the HitData that compute the internal `hit`, `normal` and `front_face` from the given ray, hit distance on that ray and outward normal.
    ///
    /// # Arguments
    /// - `ray`: The [`Ray`] which hits an object.
    /// - `hit`: The physical point where we hit the object. Probably computed as [`Ray::at()`], but we leave this for the caller since they typically need this point to compute the normal.
    /// - `t`: The distance from the `ray`'s origin, along the ray, which hits the object.
    /// - `outward_normal`: The outward facing normal that we will store but tweaked so it's always in the direction of the `ray`.
    /// - `uv`: An XY-coordinate pair relative to the object (useful for texture mapping).
    ///
    /// # Returns
    /// A new `HitData` with the math taken care of.
    #[inline]
    pub fn new(ray: Ray, hit: Vec3, t: f64, outward_normal: Vec3, uv: (f64, f64)) -> Self {
        // Compute the normal from the outward normal, remembering the direction
        let front_face: bool = ray.direct.dot(outward_normal) < 0.0;
        let normal: Vec3 = if front_face { outward_normal } else { -outward_normal };

        // Return ourselves
        Self { hit, t, normal, front_face, uv }
    }
}



/// Defines everything we want to know about a hit.
#[derive(Clone, Copy, Debug)]
pub struct HitRecord<M> {
    /// The material that we hit.
    pub mat:  M,
    /// The data about where we hit it.
    pub data: HitData,
}

// Constructors
impl<M> HitRecord<M> {
    /// Constructor for the HitRecord that compute the internal `hit`, `normal` and `front_face` from the given ray, hit distance on that ray and outward normal.
    ///
    /// # Arguments
    /// - `ray`: The [`Ray`] which hits an object.
    /// - `hit`: The physical point where we hit the object. Probably computed as [`Ray::at()`], but we leave this for the caller since they typically need this point to compute the normal.
    /// - `t`: The distance from the `ray`'s origin, along the ray, which hits the object.
    /// - `outward_normal`: The outward facing normal that we will store but tweaked so it's always in the direction of the `ray`.
    /// - `uv`: An XY-coordinate pair relative to the object (useful for texture mapping).
    /// - `mat`: The material that we hit.
    ///
    /// # Returns
    /// A new `HitRecord` with the math taken care of.
    #[inline]
    pub fn new(ray: Ray, hit: Vec3, t: f64, outward_normal: Vec3, uv: (f64, f64), mat: M) -> Self {
        Self { mat, data: HitData::new(ray, hit, t, outward_normal, uv) }
    }
}

// Raytracer
impl<M: Scattering> HitRecord<M> {
    /// Emits from the internal material using the internal [`HitData`].
    ///
    /// # Returns
    /// A [`Colour`] of the light being emitted. Is black if this emits nothing.
    #[inline]
    pub fn emitted(&self) -> Colour { self.mat.emitted(self.data.uv, self.data.hit) }

    /// Scatters the internal material using the internal [`HitData`].
    ///
    /// # Arguments
    /// - `ray`: The [`Ray`] that we hit the object with.
    /// - `env`: Some [`Environment`] describing global properties of the scene.
    ///
    /// # Returns
    /// A next [`Ray`] after the object's bounce, if any, and an attenuated [`Colour`] for this
    /// material.
    #[inline]
    pub fn scatter(&self, ray: Ray, env: &Environment) -> (Option<Ray>, Colour) { self.mat.scatter(ray, &self.data, env) }
}
