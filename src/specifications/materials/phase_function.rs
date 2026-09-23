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
use super::super::objects::pdf::UnitPDF;
use super::super::scene::Environment;
use super::{PDF as _, ScatterRecord, Scattering};
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
    type PDF = UnitPDF;

    #[inline]
    fn pdf(&self, _ray: Ray, _record: &HitData, env: &Environment, scattered: Ray) -> f64 { UnitPDF.value(scattered, env) }

    #[inline]
    fn scatter(&self, _ray: Ray, _rec: &HitData, _env: &Environment) -> Option<ScatterRecord<Self::PDF>> {
        // Create a new ray bouncing randomly in any direction
        // 1.0 / (4.0 * PI)
        Some(ScatterRecord::from_pdf(self.colour, UnitPDF))
    }
}
