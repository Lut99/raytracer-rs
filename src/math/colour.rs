//  COLOUR.rs
//    by Lut99
// 
//  Created:
//    27 Apr 2023, 15:03:09
//  Last edited:
//    07 May 2023, 10:49:59
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
use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer, Visitor};
use serde::ser::{Serializer, SerializeTuple as _};


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

    /// Returns this Colour, but with all its values clamped in the [0.0, 1.0] range.
    /// 
    /// # Returns
    /// A new `Colour` instance with the same RGBA-values, but clamped where necessary.
    pub fn clamp(&self) -> Self {
        Self {
            r : self.r.clamp(0.0, 1.0),
            g : self.g.clamp(0.0, 1.0),
            b : self.b.clamp(0.0, 1.0),
            a : self.a.clamp(0.0, 1.0),
        }
    }

    /// Returns this Colour corrected for gamma.
    /// 
    /// # Returns
    /// A new `Colour` instance with the same RGB-values, but corrected for gamma. The alpha channel is passed as-is.
    pub fn gamma(&self) -> Self {
        Self {
            r : self.r.sqrt(),
            g : self.g.sqrt(),
            b : self.b.sqrt(),
            a : self.a,
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


impl Serialize for Colour {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Start serializing it as a tuple
        let mut tup = serializer.serialize_tuple(4)?;
        tup.serialize_element(&self.r)?;
        tup.serialize_element(&self.g)?;
        tup.serialize_element(&self.b)?;
        tup.serialize_element(&self.a)?;
        tup.end()
    }
}
impl<'de> Deserialize<'de> for Colour {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// Visitor for a Colour
        struct ColourVisitor;
        impl<'de> Visitor<'de> for ColourVisitor {
            type Value = Colour;

            #[inline]
            fn expecting(&self, f: &mut Formatter) -> FResult { write!(f, "an RGBA-colour") }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                // Parse three to four elements
                let r: f64 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let g: f64 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let b: f64 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let a: f64 = seq.next_element()?.unwrap_or(1.0);

                // Construct the Colour
                Ok(Colour {
                    r,
                    g,
                    b,
                    a,
                })
            }
        }

        // Call the visitor
        deserializer.deserialize_seq(ColourVisitor)
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
