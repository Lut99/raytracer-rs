//  COLOUR.rs
//    by Lut99
// 
//  Created:
//    27 Apr 2023, 15:03:09
//  Last edited:
//    28 Apr 2023, 11:22:55
//  Auto updated?
//    Yes
// 
//  Description:
//!   Provides the [`Colour`] struct, which we use to represent a colour value.
//! 
//!   Note that this struct is quite similar to [`crate::math::vec3::Vec3`],
//!   but it's here for Software Engineering purposes.
// 

use std::fmt::{Display, Formatter, Result as FResult};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign};

use image::Rgba;


/***** LIBRARY *****/
/// Defines an RGBA colour quadruplet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colour {
    /// The red colour channel / index 0.
    pub r : f64,
    /// The green colour channel / index 1.
    pub g : f64,
    /// The blue colour channel / index 2.
    pub b : f64,
    /// The alpha channel / index 3.
    pub a : f64,
}

impl Default for Colour {
    #[inline]
    fn default() -> Self { Self::zeroes() }
}
impl Colour {
    /// Constructor for the Colour.
    /// 
    /// # Arguments
    /// - `red`: The red colour channel value for this Colour.
    /// - `green`: The green colour channel value for this Colour.
    /// - `blue`: The blue colour channel value for this Colour.
    /// - `alpha`: The alpha channel value for this Colour.
    /// 
    /// # Returns
    /// A new instance of Self with the given colour values.
    #[inline]
    pub fn new(red: impl Into<f64>, green: impl Into<f64>, blue: impl Into<f64>, alpha: impl Into<f64>) -> Self {
        Self {
            r : red.into(),
            g : green.into(),
            b : blue.into(),
            a : alpha.into(),
        }
    }

    /// Constructor for the Colour that initializes it to all-zeroes.
    /// 
    /// # Returns
    /// A new instance of Self with only 0's in it.
    #[inline]
    pub fn zeroes() -> Self {
        Self {
            r : 0.0,
            g : 0.0,
            b : 0.0,
            a : 0.0,
        }
    }



    /// Returns this Colour, but with the alpha set to 1.0.
    /// 
    /// # Returns
    /// A new `Colour` instance with the same RGB-values, but with alpha set to 1.0.
    pub fn opaque(&self) -> Self {
        Self {
            r : self.r,
            g : self.g,
            b : self.b,
            a : 1.0,
        }
    }
}

impl Neg for Colour {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            r : -self.r,
            g : -self.g,
            b : -self.b,
            a : -self.a,
        }
    }
}

impl Add for Colour {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            r : self.r + rhs.r,
            g : self.g + rhs.g,
            b : self.b + rhs.b,
            a : self.a + rhs.a,
        }
    }
}
impl AddAssign for Colour {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
        self.a += rhs.a;
    }
}
impl Sub for Colour {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            r : self.r - rhs.r,
            g : self.g - rhs.g,
            b : self.b - rhs.b,
            a : self.a - rhs.a,
        }
    }
}
impl SubAssign for Colour {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.r -= rhs.r;
        self.g -= rhs.g;
        self.b -= rhs.b;
        self.a -= rhs.a;
    }
}
impl Mul for Colour {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            r : self.r * rhs.r,
            g : self.g * rhs.g,
            b : self.b * rhs.b,
            a : self.a * rhs.a,
        }
    }
}
impl MulAssign for Colour {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.r *= rhs.r;
        self.g *= rhs.g;
        self.b *= rhs.b;
        self.a *= rhs.a;
    }
}
impl Div for Colour {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            r : self.r / rhs.r,
            g : self.g / rhs.g,
            b : self.b / rhs.b,
            a : self.a / rhs.a,
        }
    }
}
impl DivAssign for Colour {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.r /= rhs.r;
        self.g /= rhs.g;
        self.b /= rhs.b;
        self.a /= rhs.a;
    }
}

impl Add<f64> for Colour {
    type Output = Self;

    #[inline]
    fn add(self, rhs: f64) -> Self::Output {
        Self {
            r : self.r + rhs,
            g : self.g + rhs,
            b : self.b + rhs,
            a : self.a + rhs,
        }
    }
}
impl AddAssign<f64> for Colour {
    #[inline]
    fn add_assign(&mut self, rhs: f64) {
        self.r += rhs;
        self.g += rhs;
        self.b += rhs;
        self.a += rhs;
    }
}
impl Sub<f64> for Colour {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: f64) -> Self::Output {
        Self {
            r : self.r - rhs,
            g : self.g - rhs,
            b : self.b - rhs,
            a : self.a - rhs,
        }
    }
}
impl SubAssign<f64> for Colour {
    #[inline]
    fn sub_assign(&mut self, rhs: f64) {
        self.r -= rhs;
        self.g -= rhs;
        self.b -= rhs;
        self.a -= rhs;
    }
}
impl Mul<f64> for Colour {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            r : self.r * rhs,
            g : self.g * rhs,
            b : self.b * rhs,
            a : self.a * rhs,
        }
    }
}
impl MulAssign<f64> for Colour {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
        self.a *= rhs;
    }
}
impl Div<f64> for Colour {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        Self {
            r : self.r / rhs,
            g : self.g / rhs,
            b : self.b / rhs,
            a : self.a / rhs,
        }
    }
}
impl DivAssign<f64> for Colour {
    #[inline]
    fn div_assign(&mut self, rhs: f64) {
        self.r /= rhs;
        self.g /= rhs;
        self.b /= rhs;
        self.a /= rhs;
    }
}

impl Add<Colour> for f64 {
    type Output = Colour;

    #[inline]
    fn add(self, rhs: Colour) -> Self::Output {
        Colour {
            r : self + rhs.r,
            g : self + rhs.g,
            b : self + rhs.b,
            a : self + rhs.a,
        }
    }
}
impl Sub<Colour> for f64 {
    type Output = Colour;

    #[inline]
    fn sub(self, rhs: Colour) -> Self::Output {
        Colour {
            r : self + rhs.r,
            g : self - rhs.g,
            b : self - rhs.b,
            a : self + rhs.a,
        }
    }
}
impl Mul<Colour> for f64 {
    type Output = Colour;

    #[inline]
    fn mul(self, rhs: Colour) -> Self::Output {
        Colour {
            r : self * rhs.r,
            g : self * rhs.g,
            b : self * rhs.b,
            a : self + rhs.a,
        }
    }
}
impl Div<Colour> for f64 {
    type Output = Colour;

    #[inline]
    fn div(self, rhs: Colour) -> Self::Output {
        Colour {
            r : self / rhs.r,
            g : self / rhs.g,
            b : self / rhs.b,
            a : self + rhs.a,
        }
    }
}

impl Index<usize> for Colour {
    type Output = f64;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.r,
            1 => &self.g,
            2 => &self.b,
            3 => &self.a,
            i => { panic!("Index '{i}' is out-of-range for Colour"); },
        }
    }
}
impl IndexMut<usize> for Colour {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.r,
            1 => &mut self.g,
            2 => &mut self.b,
            3 => &mut self.a,
            i => { panic!("Index '{i}' is out-of-range for Colour"); },
        }
    }
}

impl Display for Colour {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "({},{},{},{})", self.r, self.g, self.b, self.a)
    }
}

impl AsRef<Colour> for Colour {
    #[inline]
    fn as_ref(&self) -> &Self { self }
}
impl AsMut<Colour> for Colour {
    #[inline]
    fn as_mut(&mut self) -> &mut Self { self }
}
impl From<&Colour> for Colour {
    #[inline]
    fn from(value: &Colour) -> Self { *value }
}
impl From<&mut Colour> for Colour {
    #[inline]
    fn from(value: &mut Colour) -> Self { *value }
}

impl From<Colour> for Rgba<u8> {
    #[inline]
    fn from(value: Colour) -> Self {
        Self([
            (255.0 * value.r) as u8,
            (255.0 * value.g) as u8,
            (255.0 * value.b) as u8,
            (255.0 * value.a) as u8,
        ])
    }
}
