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
pub const fn degrees_to_radians(degrees: f64) -> f64 { degrees * PI / 180.0 }

/// Samples a random point in a unit disk.
#[inline]
pub fn random_in_unit_disk() -> Vec3 {
    // NOTE: Terrible, but hard to do better?
    loop {
        let vec = Vec3::new(2.0 * fastrand::f64_inclusive() - 1.0, 2.0 * fastrand::f64_inclusive() - 1.0, 0.0);
        if vec.length2() < 1.0 {
            return vec;
        }
    }
}





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
    /// The amount of defocus to use. Set to 0 to disable.
    defocus_angle: f64,
    /// The amount of defocus to render in the horizontal direction.
    defocus_u: Vec3,
    /// The amount of defocus to render in the vertical direction.
    defocus_v: Vec3,
    /// The shutter time of the Camera, in microseconds.
    ///
    /// Use a shutter time of `1` to disable motion blur.
    shutter_time: u64,
}

// Constructors
impl Camera {
    /// Constructor for the Camera that initializes at the origin (0, 0, 0), looking forward, with
    /// the given settings.
    ///
    /// # Arguments
    /// - `dims`: Defines the width x height of the resulting image, in pixels.
    /// - `vfov`: Defines the vertical field-of-view (fov) for the camera.
    /// - `defocus_angle`: The amount of defocus to use. Set to 0 to disable.
    /// - `focus_dist`: The distance between us and the focal point where the camera is sharp.
    /// - `shutter_time_us`:  The shutter time of the Camera, in microseconds. Use a shutter time
    ///   of `1` to disable motion blur.
    /// - `lookfrom`: Defines the point where we are looking _from_.
    /// - `lookat`: Defines the point where we are looking _to_.
    /// - `up`: Defines the vertex pointing to the up.
    ///
    /// # Returns
    /// A new Camera instance derived from the given properties.
    #[track_caller]
    pub fn new(
        dims: (u32, u32),
        vfov: f64,
        defocus_angle: f64,
        mut focus_dist: f64,
        shutter_time_us: u64,
        lookfrom: Vec3,
        lookat: Vec3,
        up: Vec3,
    ) -> Self {
        // Compute the origin
        let origin: Vec3 = lookfrom;

        // Update the focal dist if it is bogus
        if focus_dist == 0.0 {
            focus_dist = (lookfrom - lookat).length();
        }

        // Compute the viewport dimensions
        let aspect_ratio: f64 = dims.0 as f64 / dims.1 as f64;
        let theta = degrees_to_radians(vfov);
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * focus_dist;
        let viewport_width = viewport_height * aspect_ratio;

        // Compute the viewport's vectors
        let w = (lookfrom - lookat).unit();
        let u = cross3(up, w).unit();
        let v = cross3(w, u);
        let horizontal: Vec3 = viewport_width * u;
        let vertical: Vec3 = viewport_height * v;

        // Compute the lower left corner position of the vector (such that we can add the horizontal and vertical vectors)
        let lower_left: Vec3 = origin - (focus_dist * w) - horizontal / 2.0 - vertical / 2.0;

        // Calculate the defocus disk
        let defocus_radius = focus_dist * (degrees_to_radians(defocus_angle / 2.0)).tan();
        let defocus_u = u * defocus_radius;
        let defocus_v = v * defocus_radius;

        // Use that to create ourselves
        Self {
            dims,
            origin,
            lower_left,
            horizontal,
            vertical,
            defocus_angle,
            defocus_u,
            defocus_v,
            shutter_time: if shutter_time_us > 0 { shutter_time_us } else { panic!("Shutter time cannot be 0") },
        }
    }
}

// Camera
impl Camera {
    /// Returns a ray casted through the given virtual coordinates of the viewport.
    ///
    /// # Arguments
    /// - `u`: The virtual X-coordinate, ranging [0.0, 1.0].
    /// - `v`: The virtual Y-coordinate, ranging [0.0, 1.0].
    /// - `t`: The current time in microseconds. Note that rays may be casted a tiny amount in the
    ///   future to emulate shutter time.
    ///
    /// # Returns
    /// A new [`Ray`], casted from this camera's `origin` through the viewport spanned by it.
    #[inline]
    pub fn cast(&self, u: f64, v: f64, t: u64) -> Ray {
        let ray_origin: Vec3 = if self.defocus_angle <= 0.0 { self.origin } else { self.defocus_disk_sample() };
        Ray::with_time(ray_origin, self.lower_left + u * self.horizontal + v * self.vertical - ray_origin, fastrand::u64(t..t + self.shutter_time))
    }

    /// Samples a random point on the "defocus disk"
    ///
    /// This is a small disk around the camera's center (aligned with the viewport) that we will
    /// sample from to simulate focal point blurring.
    #[inline]
    fn defocus_disk_sample(&self) -> Vec3 {
        let p = random_in_unit_disk();
        self.origin + (p.x * self.defocus_u) + (p.y * self.defocus_v)
    }

    /// Returns the dimensions we're rendering to, as a width x height pair.
    #[inline]
    pub const fn dims(&self) -> (u32, u32) { self.dims }
}
