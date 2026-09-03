//  DIFFUSE.rs
//    by Lut99
//
//  Created:
//    05 May 2023, 10:50:32
//  Last edited:
//    06 May 2023, 11:34:15
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements various kinds of diffuse-like materials, all with
//!   slightly different methods of "randomly" bouncing rays.
//

use std::convert::Infallible;
use std::f64::consts::PI;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::Loadable;
use super::super::scene::Environment;
use super::Scattering;
use crate::math::{Colour, ONB, Ray, Vec3};
use crate::random;
use crate::specifications::objects::HitData;
use crate::specifications::textures::{Texture, Textured};


/***** HELPER FUNCTIONS *****/
/// Generates a random, uniformly sampled vector in a unit sphere around the origin.
///
/// # Returns
/// A new [`Vec3`] that represents the random vector.
pub fn random3_uniform() -> Vec3 {
    // // Generate the three coordinates randomly
    // let res: Vec3 = Vec3 { x: fastrand::f64(), y: fastrand::f64(), z: fastrand::f64() };

    // // Always return a unit vector version of this vector
    // res.unit()

    // We'll use a loop - sadly
    loop {
        let p = Vec3::randint(-1.0, 1.0);
        let lensq = p.length2();
        if lensq > 1e-160 && lensq <= 1.0 {
            return p / lensq.sqrt();
        }
    }
}

/// Generates a random, uniformly sampled vector on a hemisphere w.r.t. the normal.
#[inline]
pub fn random3_on_hemisphere(normal: Vec3) -> Vec3 {
    let on_unit_sphere: Vec3 = random3_uniform();
    if on_unit_sphere.dot(normal) > 0.0 { on_unit_sphere } else { -on_unit_sphere }
}

/// Generates a random, uniformly sampled vector in a unit sphere around the origin.
///
/// Except that this math is much more complex yet robust.
///
/// # Returns
/// A new [`Vec3`] that represents the random vector.
pub fn random3_cosine_direction() -> Vec3 {
    let r1 = random::f64();
    let r2 = random::f64();
    let r2_sqrt = r2.sqrt();
    let phi = 2.0 * PI * r1;
    let x = phi.cos() * r2_sqrt;
    let y = phi.sin() * r2_sqrt;
    let z = (1.0 - r2).sqrt();
    Vec3::new(x, y, z)
}



/// Implements the PDF used by Lambertian scattering.
#[inline]
pub fn lambertian_pdf(record: &HitData, scattered: Ray) -> f64 {
    let cos_theta = record.normal.dot(scattered.direct.unit());
    f64::max(0.0, cos_theta / PI)
}

/// Implements Lambertian scattering logic.
#[inline]
pub fn lambertian_scatter(ray: Ray, record: &HitData, colour: Colour) -> (Option<Ray>, Colour, f64) {
    // Get a coordinate base around the hit normal, then get a random direction in that unit sphere
    // to find a random scatter direction.
    let uvw = ONB::from_single_axis(record.normal);
    let scattered = uvw.transform(random3_cosine_direction()).unit();
    let scattered = Ray::with_time(record.hit, scattered, ray.time);

    // Now we can simply return the new ray to bounce and the colour
    // NOTE: By construction, `uvw.w` == `record.normal` but unit)
    (Some(scattered), colour, uvw.w.dot(scattered.direct) / PI)
}





/***** LIBRARY *****/
/// Implements the laziest diffuse material, which simply uniformly bounces the ray off of its surface.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Diffuse {
    /// The colour of the material.
    pub colour: Colour,
}
impl Loadable for Diffuse {
    type Error = Infallible;

    #[inline]
    fn load(&mut self, _dir: &Path) -> Result<(), Self::Error> { Ok(()) }
}
impl Scattering for Diffuse {
    #[inline]
    fn scatter(&self, _ray: Ray, record: &HitData, _env: &Environment) -> (Option<Ray>, Colour, f64) {
        // Return a ray scattered in a random direction
        let direction: Vec3 = random3_on_hemisphere(record.normal);
        (Some(Ray::new(record.hit, direction)), self.colour, 1.0 / (2.0 * PI))
    }
}



/// A diffuse material that emits light in all directions instead of scattering it.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct DiffuseLight {
    /// The colour of the light.
    pub colour: Colour,
}
impl Loadable for DiffuseLight {
    type Error = Infallible;

    #[inline]
    fn load(&mut self, _dir: &Path) -> Result<(), Self::Error> { Ok(()) }
}
impl Scattering for DiffuseLight {
    #[inline]
    fn emitted(&self, _uv: (f64, f64), _p: Vec3) -> Colour { self.colour }
}



/// A diffuse material with truer scattering.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Lambertian {
    /// The colour of the material.
    pub colour: Colour,
}
impl Loadable for Lambertian {
    type Error = Infallible;

    #[inline]
    fn load(&mut self, _dir: &Path) -> Result<(), Self::Error> { Ok(()) }
}
impl Scattering for Lambertian {
    #[inline]
    fn pdf(&self, _ray: Ray, record: &HitData, _env: &Environment, scattered: Ray) -> f64 { lambertian_pdf(record, scattered) }

    #[inline]
    fn scatter(&self, ray: Ray, record: &HitData, _env: &Environment) -> (Option<Ray>, Colour, f64) { lambertian_scatter(ray, record, self.colour) }
}



/// A diffuse material with truer scattering a texture.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct LambertianTexture<T = Texture> {
    /// The texture to scatter.
    pub texture: T,
}
impl<T: Loadable> Loadable for LambertianTexture<T> {
    type Error = T::Error;

    #[inline]
    fn load(&mut self, dir: &Path) -> Result<(), Self::Error> { self.texture.load(dir) }
}
impl<T: Textured> Scattering for LambertianTexture<T> {
    #[inline]
    fn pdf(&self, _ray: Ray, record: &HitData, _env: &Environment, scattered: Ray) -> f64 { lambertian_pdf(record, scattered) }

    #[inline]
    fn scatter(&self, ray: Ray, record: &HitData, _env: &Environment) -> (Option<Ray>, Colour, f64) {
        lambertian_scatter(ray, record, self.texture.value(record.uv, record.hit))
    }
}
