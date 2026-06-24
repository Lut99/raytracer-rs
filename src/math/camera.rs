//  CAMERA.rs
//    by Lut99
//
//  Created:
//    28 Apr 2023, 10:33:16
//  Last edited:
//    02 May 2023, 18:17:51
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the [`Camera`] class, which we can use to control the camera
//!   in a render scene.
//

use std::f64::consts::PI;

use super::ray::Ray;
use super::vec3::Vec3;
use crate::math::vec3::cross3;


/***** HELPER FUNCTION *****/
/// Turns degrees into radians, crazy.
#[inline]
const fn degrees_to_radians(degrees: f64) -> f64 { degrees * PI / 180.0 }





/***** LIBRARY *****/
/// The Camera struct defines a camera and controls for managing it.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    /// The dimensions to render to.
    dims: (u32, u32),
    /// The origin where rays are shot from.
    origin: Vec3,
    /// The lower-left corner of the triangle to shoot through.
    lower_left: Vec3,
    /// The direction of the camera's triangle rightward.
    horizontal: Vec3,
    /// The direction of the camera's triangle downward.
    vertical: Vec3,
}

// Constructors
impl Camera {
    /// Constructor for the Camera that initializes at the origin (0, 0, 0), looking forward, with the given settings.
    ///
    /// # Arguments
    /// - `dims`: Defines the width x height of the resulting image, in pixels.
    /// - `vfov`: Defines the vertical field-of-view (fov) for the camera.
    /// - `lookfrom`: Defines the point where we are looking _from_.
    /// - `lookat`: Defines the point where we are looking _to_.
    /// - `up`: Defines the vertex pointing to the up.
    ///
    /// # Returns
    /// A new Camera instance derived from the given properties.
    pub fn new(dims: (u32, u32), vfov: f64, lookfrom: Vec3, lookat: Vec3, up: Vec3) -> Self {
        // Compute the origin
        let origin: Vec3 = lookfrom;

        // Compute the viewport dimensions
        let aspect_ratio: f64 = dims.0 as f64 / dims.1 as f64;
        let focal_length: f64 = (lookfrom - lookat).length();
        let theta = degrees_to_radians(vfov);
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * focal_length;
        let viewport_width = viewport_height * aspect_ratio;

        // Compute the viewport's vectors
        let w = (lookfrom - lookat).unit();
        let u = cross3(up, w).unit();
        let v = cross3(w, u);
        let horizontal: Vec3 = viewport_width * u;
        let vertical: Vec3 = viewport_height * v;

        // Compute the lower left corner position of the vector (such that we can add the horizontal and vertical vectors)
        let lower_left: Vec3 = origin - (focal_length * w) - horizontal / 2.0 - vertical / 2.0;

        // Use that to create ourselves
        Self { dims, origin, lower_left, horizontal, vertical }
    }
}

// Camera
impl Camera {
    /// Returns a ray casted through the given virtual coordinates of the viewport.
    ///
    /// # Arguments
    /// - `u`: The virtual X-coordinate, ranging [0.0, 1.0].
    /// - `v`: The virtual Y-coordinate, ranging [0.0, 1.0].
    ///
    /// # Returns
    /// A new [`Ray`], casted from this camera's `origin` through the viewport spanned by it.
    #[inline]
    pub fn cast(&self, u: f64, v: f64) -> Ray { Ray::new(self.origin, self.lower_left + u * self.horizontal + v * self.vertical - self.origin) }

    /// Returns the dimensions we're rendering to, as a width x height pair.
    #[inline]
    pub const fn dims(&self) -> (u32, u32) { self.dims }
}
