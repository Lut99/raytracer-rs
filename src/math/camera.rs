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

/// Samples a random point on the "defocus disk"
///
/// This is a small disk around the camera's center (aligned with the viewport) that we will
/// sample from to simulate focal point blurring.
#[inline]
fn defocus_disk_sample(cam: &Camera) -> Vec3 {
    let p = random_in_unit_disk();
    cam.origin + (p.x * cam.defocus_u) + (p.y * cam.defocus_v)
}




/***** ITERATORS *****/
/// Returns an iterator yielding rays casted through a [`Camera`].
///
/// Unlike earlier designs, this simply features everything in one go - coordinate generation,
/// sampling and casting.
#[derive(Clone, Copy, Debug)]
pub struct Rays<'c> {
    cam:   &'c Camera,
    t_us:  u64,
    index: u64,
}

impl<'c> ExactSizeIterator for Rays<'c> {
    #[inline]
    fn len(&self) -> usize { ((self.cam.dims.0 as u64 * self.cam.dims.1 as u64 * self.cam.n_samples as u64).saturating_sub(self.index)) as usize }
}
impl<'c> Iterator for Rays<'c> {
    type Item = (u64, u32, u32, Ray);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.len() == 0 {
            return None;
        }

        // Cast the index into an (s, x, y)-pair.
        let s: u64 = self.index % self.cam.n_samples;
        let r: u64 = self.index / self.cam.n_samples;
        let x: u64 = r % self.cam.dims.0 as u64;
        let y: u64 = r / self.cam.dims.0 as u64;

        // Randomly mod the XY-pair if we're sampling
        let (x, y): (f64, f64) = if self.cam.n_samples > 1 { (x as f64 + fastrand::f64(), y as f64 + fastrand::f64()) } else { (x as f64, y as f64) };

        // Convert the pixel values to logical values
        let u: f64 = x / (self.cam.dims.0 as f64 - 1.0);
        let v: f64 = y / (self.cam.dims.1 as f64 - 1.0);
        let w: f64 = s as f64 / self.cam.n_samples as f64;

        // Cast the ray
        let ray = self.cam.cast(u, v, w, self.t_us);
        self.index += 1;
        Some((s, x as u32, y as u32, ray))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.index += n as u64;
        self.next()
    }

    #[inline]
    fn count(self) -> usize { self.len() }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len: usize = self.len();
        (len, Some(len))
    }
}





/***** LIBRARY *****/
/// The Camera struct defines a camera and controls for managing it.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    // Image properties
    /// The dimensions to render to.
    dims: (u32, u32),

    // Features
    /// The amount of samples to cast per-pixel.
    n_samples:     u64,
    /// The amount of defocus to use. Set to 0 to disable.
    defocus_angle: f64,
    /// The amount of defocus to render in the horizontal direction.
    defocus_u:     Vec3,
    /// The amount of defocus to render in the vertical direction.
    defocus_v:     Vec3,
    /// The shutter time of the Camera, in microseconds.
    ///
    /// Use a shutter time of `1` to disable motion blur.
    shutter_time:  u64,

    // Location
    /// The origin where rays are shot from.
    origin:     Vec3,
    /// The lower-left corner of the triangle to shoot through.
    lower_left: Vec3,
    /// The direction of the camera's triangle rightward.
    horizontal: Vec3,
    /// The direction of the camera's triangle downward.
    vertical:   Vec3,
}

// Constructors
impl Camera {
    /// Constructor for the Camera that initializes at the origin (0, 0, 0), looking forward, with
    /// the given settings.
    ///
    /// # Arguments
    /// - `dims`: Defines the width x height of the resulting image, in pixels.
    /// - `n_samples`: The number of sample to cast per-ray.
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
        n_samples: u64,
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
        let u = up.cross(w).unit();
        let v = w.cross(u);
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
            dims: (if dims.0 > 0 { dims.0 } else { panic!("Width cannot be 0") }, if dims.1 > 0 { dims.1 } else { panic!("Height cannot be 0") }),
            n_samples: if n_samples > 0 { n_samples } else { panic!("Number of samples cannot be 0") },
            defocus_angle,
            defocus_u,
            defocus_v,
            shutter_time: if shutter_time_us > 0 { shutter_time_us } else { panic!("Shutter time cannot be 0") },
            origin,
            lower_left,
            horizontal,
            vertical,
        }
    }
}

// Camera
impl Camera {
    /// Casts a [`Ray`] through a single (logical) pixel.
    ///
    /// # Arguments
    /// - `u`: The logical pixel index for the width, ranging `0.0` - `1.0`.
    /// - `v`: The logical pixel index for the height, ranging `0.0` - `1.0`.
    /// - `s`: The logical pixel index for the sample, ranging `0.0` - `1.0`.
    /// - `t_us`: The time at which the Ray is shot, as the number of microseconds since the start
    ///   of the scene.
    ///
    /// # Returns
    /// A [`Ray`] casted through the Camera lens.
    #[inline]
    pub fn cast(&self, u: f64, v: f64, s: f64, t_us: u64) -> Ray {
        let ray_origin: Vec3 = if self.defocus_angle <= 0.0 { self.origin } else { defocus_disk_sample(self) };
        Ray::with_time(
            ray_origin,
            self.lower_left + u * self.horizontal + v * self.vertical - ray_origin,
            t_us + (s * self.shutter_time as f64).round() as u64,
        )
    }

    /// Returns an iterator yielding [`Ray`]s casted through the Camera's lens.
    ///
    /// # Arguments
    /// - `t_us`: The time at which we're casting them, given as microseconds since the start of
    ///   the scene.
    ///
    /// # Returns
    /// A [`Rays`] iterator that yields each of them.
    #[inline]
    pub const fn rays(&self, t_us: u64) -> Rays<'_> { Rays { cam: self, t_us, index: 0 } }
}

// Properties
impl Camera {
    /// Returns the dimensions we're rendering to, as a width x height pair.
    #[inline]
    pub const fn dims(&self) -> (u32, u32) { self.dims }

    /// Returns the number of samples we draw per coordinate.
    #[inline]
    pub const fn n_samples(&self) -> u64 { self.n_samples }
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;
    use crate::specifications::scene::CameraInfo;

    #[test]
    fn test_rays() {
        let mut info = CameraInfo::default();
        info.dims = (3.try_into().unwrap(), 2.try_into().unwrap());
        info.n_samples = 3.try_into().unwrap();
        let cam: Camera = info.into();
        let mut rays = cam.rays(0);
        assert!(matches!(rays.next(), Some((0, 0, 0, _))));
        assert!(matches!(rays.next(), Some((1, 0, 0, _))));
        assert!(matches!(rays.next(), Some((2, 0, 0, _))));
        assert!(matches!(rays.next(), Some((0, 1, 0, _))));
        assert!(matches!(rays.next(), Some((1, 1, 0, _))));
        assert!(matches!(rays.next(), Some((2, 1, 0, _))));
        assert!(matches!(rays.next(), Some((0, 2, 0, _))));
        assert!(matches!(rays.next(), Some((1, 2, 0, _))));
        assert!(matches!(rays.next(), Some((2, 2, 0, _))));
        assert!(matches!(rays.next(), Some((0, 0, 1, _))));
        assert!(matches!(rays.next(), Some((1, 0, 1, _))));
        assert!(matches!(rays.next(), Some((2, 0, 1, _))));
        assert!(matches!(rays.next(), Some((0, 1, 1, _))));
        assert!(matches!(rays.next(), Some((1, 1, 1, _))));
        assert!(matches!(rays.next(), Some((2, 1, 1, _))));
        assert!(matches!(rays.next(), Some((0, 2, 1, _))));
        assert!(matches!(rays.next(), Some((1, 2, 1, _))));
        assert!(matches!(rays.next(), Some((2, 2, 1, _))));
        assert!(matches!(rays.next(), None));
    }
}
