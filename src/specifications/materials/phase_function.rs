//  PHASE FUNCTION.rs
//    by Lut99
//
//  Description:
//!   Defines phase functions for gas clouds.
//

use std::convert::Infallible;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::Loadable;
use super::super::objects::HitData;
use super::super::scene::Environment;
use super::Scattering;
use super::diffuse::random3_uniform;
use crate::math::{Colour, Ray};


/***** LIBRARY *****/
/// A phase function that simply randomly scatters a ray, regardless of surface.
///
/// This is useful for scattering in a volume like a gas cloud instead of on a surface.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Isotropic {
    /// The colour of the gas.
    pub colour: Colour,
}

// Interfaces
impl Loadable for Isotropic {
    type Error = Infallible;

    #[inline]
    fn load(&mut self, _dir: &Path) -> Result<(), Self::Error> { Ok(()) }
}
impl Scattering for Isotropic {
    #[inline]
    fn scatter(&self, ray: Ray, rec: &HitData, _env: &Environment) -> (Option<Ray>, Colour) {
        // Create a new ray bouncing randomly in any direction
        (Some(Ray::with_time(rec.hit, random3_uniform(), ray.time)), self.colour)
    }
}
