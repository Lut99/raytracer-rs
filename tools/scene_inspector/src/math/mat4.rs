//  MATRIX 4D.rs
//    by Lut99
//
//  Description:
//!   Implements a 4D matrix, akin to the raytracer's
//!   [`Vec3`](raytracer::math::Vec3).
//

use std::ops::{Index, IndexMut, Mul, MulAssign};


/***** TYPE ALIASES *****/
/// Floating-point type used in the impl.
#[allow(non_camel_case_types)]
pub type fty = f64;





/***** LIBRARY *****/
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4(pub [[f64; 4]; 4]);

// Constants
impl Mat4 {
    /// The default matrix we use to go from OpenGL's coordinate system to WGPU's coordinate system
    pub const OPENGL_TO_WGPU: Self = Self([[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 0.5, 0.5], [0.0, 0.0, 0.0, 1.0]]);
}

// Constructors
impl Default for Mat4 {
    #[inline]
    fn default() -> Self { Self::zeroes() }
}
impl Mat4 {
    /// Constructor for the Vec3 that initializes it to all-zeroes.
    ///
    /// # Returns
    /// A new instance of Self with only 0's in it.
    #[inline]
    pub const fn zeroes() -> Self { Self([[0.0; 4]; 4]) }
}

// Ops
impl Mul for Mat4 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        let mut res = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    res[i][j] += self.0[i][k] * rhs.0[k][j];
                }
            }
        }
        Self(res)
    }
}
impl MulAssign for Mat4 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) { *self = self.mul(rhs); }
}
impl Index<usize> for Mat4 {
    type Output = fty;

    #[inline]
    #[track_caller]
    fn index(&self, index: usize) -> &Self::Output {
        if index > 15 {
            panic!("An index of {index} is out-of-bounds for a 4x4 matrix")
        }
        let y = index / 4;
        let x = index % 4;
        &self.0[y][x]
    }
}
impl IndexMut<usize> for Mat4 {
    /// NOTE: Indexes as (y, x)
    #[inline]
    #[track_caller]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index > 15 {
            panic!("An index of {index} is out-of-bounds for a 4x4 matrix")
        }
        let y = index / 4;
        let x = index % 4;
        &mut self.0[y][x]
    }
}
impl Index<(usize, usize)> for Mat4 {
    type Output = fty;

    /// NOTE: Indexes as (y, x)
    #[inline]
    #[track_caller]
    fn index(&self, (y, x): (usize, usize)) -> &Self::Output {
        if y > 3 {
            panic!("A Y-coordinate of {y} is out-of-bounds for a 4x4 matrix")
        }
        if x > 3 {
            panic!("A X-coordinate of {x} is out-of-bounds for a 4x4 matrix")
        }
        &self.0[y][x]
    }
}
impl IndexMut<(usize, usize)> for Mat4 {
    /// NOTE: Indexes as (y, x)
    #[inline]
    #[track_caller]
    fn index_mut(&mut self, (y, x): (usize, usize)) -> &mut Self::Output {
        if y > 3 {
            panic!("A Y-coordinate of {y} is out-of-bounds for a 4x4 matrix")
        }
        if x > 3 {
            panic!("A X-coordinate of {x} is out-of-bounds for a 4x4 matrix")
        }
        &mut self.0[y][x]
    }
}
