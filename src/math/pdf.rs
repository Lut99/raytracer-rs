//  PDF.rs
//    by Lut99
//
//  Description:
//!   Implements an abstract idea of Probability Density Functions (PDF), and
//!   some concrete ones while at it.
//

use std::f64::consts::PI;

use super::camera::random_in_unit_disk;
use super::onb::ONB;
use super::vec3::Vec3;
use crate::specifications::materials::diffuse::random3_cosine_direction;


/***** INTERFACES *****/
/// Abstracts over various Probability Density Functions, for convenience.
pub trait PDF {
    /// Sample a value from the PDF based on the direction a vector points in.
    ///
    /// # Arguments
    /// - `direct`: A [`Vec3`] that represents the direction on the unit sphere.
    ///
    /// # Returns
    /// A [`f64`] representing the probability of a ray landing on this direction.
    fn value(&self, direct: Vec3) -> f64;

    /// Generates a random vector in a direction weighted by this PDF.
    ///
    /// # Returns
    /// A new [`Vec3`] randomly sampled from this PDF.
    fn sample(&self) -> Vec3;
}





/***** LIBRARY *****/
/// A uniform PDF over the unit sphere.
pub struct UnitPDF;

// Interfaces
impl PDF for UnitPDF {
    #[inline]
    fn value(&self, _direct: Vec3) -> f64 { 1.0 / (4.0 * PI) }

    #[inline]
    fn sample(&self) -> Vec3 { random_in_unit_disk() }
}



/// Cosine PDF over the unit sphere.
pub struct CosinePDF {
    /// The orthonormal basis for coordinates on the unit sphere.
    pub onb: ONB,
}

// Interfaces
impl PDF for CosinePDF {
    #[inline]
    fn value(&self, direct: Vec3) -> f64 {
        let cosine_theta = direct.unit().dot(self.onb.w);
        f64::max(0.0, cosine_theta / PI)
    }

    #[inline]
    fn sample(&self) -> Vec3 { self.onb.transform(random3_cosine_direction()) }
}
