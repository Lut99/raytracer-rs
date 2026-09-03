//  ORTHONORMAL BASIS.rs
//    by Lut99
//
//  Description:
//!   A class for representing orthonormal basises of coordinate systems.
//

use super::vec3::Vec3;


/***** LIBRARY *****/
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ONB {
    /// The first of the three axes that make up the basis.
    ///
    /// Anagolous to the X-axis in a Cartesian coordinate system.
    pub u: Vec3,
    /// The second of the three axes that make up the basis.
    ///
    /// Anagolous to the Y-axis in a Cartesian coordinate system.
    pub v: Vec3,
    /// The third of the three axes that make up the basis.
    ///
    /// Anagolous to the Z-axis in a Cartesian coordinate system.
    pub w: Vec3,
}

// Constructors
impl ONB {
    /// Computes a new ONB from a single axis.
    ///
    /// The second axis is a vector orthogonal to the given one and either the X- or Y-axis
    /// (depending on which is farther from the given vector) and the third is orthogonal to those.
    ///
    /// # Arguments
    /// - `axis`: The single axis to base the basis around.
    ///
    /// # Returns
    /// A new ONB with three axes that form the coordinate system.
    #[inline]
    pub fn from_single_axis(axis: Vec3) -> Self {
        let w: Vec3 = axis.unit();
        let a: Vec3 = if w.x.abs() > 0.9 { Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) };
        let v: Vec3 = w.cross(a).unit();
        Self { u: w.cross(v), v, w }
    }
}

// Math
impl ONB {
    /// Returns a vector in a Cartesian coordinate system to one in the ONB's coordinate system.
    ///
    /// Inverse interface of [`Vec3::transform()`].
    ///
    /// # Arguments
    /// - `vec`: The [`Vec3`] to transform.
    ///
    /// # Returns
    /// The transformed `Vec3`.
    #[inline]
    pub fn transform(&self, vec: Vec3) -> Vec3 { vec.x * self.u + vec.y * self.v + vec.z * self.w }
}
impl Vec3 {
    /// Returns this vector in the given [`ONB`]'s coordinate space.
    ///
    /// Inverse interface of [`ONB::transform()`].
    ///
    /// # Arguments
    /// - `onb`: Some [`ONB`] to transform this vector to.
    ///
    /// # Returns
    /// The transformed `Vec3`.
    #[inline]
    pub fn transform(self, onb: &ONB) -> Vec3 { onb.transform(self) }
}
