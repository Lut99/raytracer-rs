//  VEC 3.rs
//    by Lut99
//
//  Created:
//    27 Apr 2023, 13:27:44
//  Last edited:
//    06 May 2023, 11:21:40
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the [`Vec3`] class (and related) that we can use for 3D
//!   linear algebra.
//

use std::fmt::{Display, Formatter, Result as FResult};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign};

use serde::de::{self, Deserializer, Visitor};
use serde::ser::{SerializeTuple as _, Serializer};
use serde::{Deserialize, Serialize};


/***** TYPE ALIASES *****/
/// Floating-point type used in the impl.
#[allow(non_camel_case_types)]
pub type fty = f64;





/***** LIBRARY *****/
/// The `Vec3` class implements a 3D vector. By default, it abstracts over double-precision floats, but this can be changed manually.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    /// The X-coordinate / index 0.
    pub x: fty,
    /// The Y-coordinate / index 1.
    pub y: fty,
    /// The Z-coordinate / index 2.
    pub z: fty,
}

// Constructors
impl Default for Vec3 {
    #[inline]
    fn default() -> Self { Self::zeroes() }
}
impl Vec3 {
    /// Constructor for the Vec3.
    ///
    /// # Arguments
    /// - `x`: The X-coordinate for this Vec3.
    /// - `y`: The Y-coordinate for this Vec3.
    /// - `z`: The Z-coordinate for this Vec3.
    ///
    /// # Returns
    /// A new instance of Self with the given coordinates.
    #[inline]
    pub const fn new(x: fty, y: fty, z: fty) -> Self { Self { x, y, z } }

    /// Constructor for the Vec3 that initializes it to all-zeroes.
    ///
    /// # Returns
    /// A new instance of Self with only 0's in it.
    #[inline]
    pub const fn zeroes() -> Self { Self { x: 0.0, y: 0.0, z: 0.0 } }
}

// Facts
impl Vec3 {
    /// Returns whether this Vec3 is _nearly_ zero.
    ///
    /// This is used to "round off" the last bit of colour, and as such, only useful for
    /// floating-point numbers.
    ///
    /// # Returns
    /// True if every component of this vector is below `1e-8`, or false if any of them isn't.
    #[inline]
    pub fn is_nearly_zero(&self) -> bool { self.x.abs() < 1e-8 && self.y.abs() < 1e-8 && self.z.abs() < 1e-8 }

    /// Computes the length of the vector.
    ///
    /// If you plan to square the length later anyway, consider using [`Vec3::length2()`] instead.
    ///
    /// # Returns
    /// The mathmatical length of this vector.
    #[inline]
    pub fn length(&self) -> fty { self.length2().sqrt() }

    /// Computes the length of the vector, but still squared.
    ///
    /// Use this is if you plan to square the length later anyway. Else, consider using
    /// [`Vec3::length()`] instead.
    ///
    /// # Returns
    /// The mathmatical length of this vector to the power of two.
    #[inline]
    pub fn length2(&self) -> fty { self.x * self.x + self.y * self.y + self.z * self.z }
}

// Custom ops
impl Vec3 {
    /// Returns a Vec3 that is the unit vector of this vector.
    ///
    /// # Returns
    /// A new Vec3 of floats that has the same direction but [length](Vec3::length()) `1`.
    #[inline]
    pub fn unit(&self) -> Vec3 {
        let len: f64 = self.length();
        Vec3 { x: self.x / len, y: self.y / len, z: self.z / len }
    }

    /// Computes the dot product of this with another Vec3.
    ///
    /// # Arguments
    /// - `other`: The righthand-side of the computation.
    ///
    /// # Returns
    /// A new Vec3 with the value of `self dot other`.
    #[inline]
    pub fn dot(self, other: Self) -> fty { self.x * other.x + self.y * other.y + self.z * other.z }

    /// Computes the cross product of this with another Vec3.
    ///
    /// # Arguments
    /// - `other`: The righthand-side of the computation.
    ///
    /// # Returns
    /// A new Vec3 with the value of`self cross other`.
    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Vec3 { x: self.y * other.z - self.z * other.y, y: self.z * other.x - self.x * other.z, z: self.x * other.y - self.y * other.x }
    }
}

// Std ops
impl Neg for Vec3 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output { Self { x: -self.x, y: -self.y, z: -self.z } }
}

impl Add for Vec3 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output { Self { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z } }
}
impl AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}
impl Sub for Vec3 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output { Self { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z } }
}
impl SubAssign for Vec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}
impl Mul for Vec3 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output { Self { x: self.x * rhs.x, y: self.y * rhs.y, z: self.z * rhs.z } }
}
impl MulAssign for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}
impl Div for Vec3 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output { Self { x: self.x / rhs.x, y: self.y / rhs.y, z: self.z / rhs.z } }
}
impl DivAssign for Vec3 {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl Add<fty> for Vec3 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: fty) -> Self::Output { Self { x: self.x + rhs, y: self.y + rhs, z: self.z + rhs } }
}
impl AddAssign<fty> for Vec3 {
    #[inline]
    fn add_assign(&mut self, rhs: fty) {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
    }
}
impl Sub<fty> for Vec3 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: fty) -> Self::Output { Self { x: self.x - rhs, y: self.y - rhs, z: self.z - rhs } }
}
impl SubAssign<fty> for Vec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: fty) {
        self.x -= rhs;
        self.y -= rhs;
        self.z -= rhs;
    }
}
impl Mul<fty> for Vec3 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: fty) -> Self::Output { Self { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs } }
}
impl MulAssign<fty> for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: fty) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}
impl Div<fty> for Vec3 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: fty) -> Self::Output { Self { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs } }
}
impl DivAssign<fty> for Vec3 {
    #[inline]
    fn div_assign(&mut self, rhs: fty) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Add<Vec3> for f64 {
    type Output = Vec3;

    #[inline]
    fn add(self, rhs: Vec3) -> Self::Output { Vec3 { x: self + rhs.x, y: self + rhs.y, z: self + rhs.z } }
}
impl Sub<Vec3> for f64 {
    type Output = Vec3;

    #[inline]
    fn sub(self, rhs: Vec3) -> Self::Output { Vec3 { x: self - rhs.x, y: self - rhs.y, z: self - rhs.z } }
}
impl Mul<Vec3> for f64 {
    type Output = Vec3;

    #[inline]
    fn mul(self, rhs: Vec3) -> Self::Output { Vec3 { x: self * rhs.x, y: self * rhs.y, z: self * rhs.z } }
}
impl Div<Vec3> for f64 {
    type Output = Vec3;

    #[inline]
    fn div(self, rhs: Vec3) -> Self::Output { Vec3 { x: self / rhs.x, y: self / rhs.y, z: self / rhs.z } }
}

impl Index<usize> for Vec3 {
    type Output = fty;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            i => {
                panic!("Index '{i}' is out-of-range for Vec3");
            },
        }
    }
}
impl IndexMut<usize> for Vec3 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            i => {
                panic!("Index '{i}' is out-of-range for Vec3");
            },
        }
    }
}

impl Serialize for Vec3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Start serializing it as a tuple
        let mut tup = serializer.serialize_tuple(3)?;
        tup.serialize_element(&self.x)?;
        tup.serialize_element(&self.y)?;
        tup.serialize_element(&self.z)?;
        tup.end()
    }
}
impl<'de> Deserialize<'de> for Vec3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// Visitor for a Vec3
        struct Vec3Visitor;
        impl<'de> Visitor<'de> for Vec3Visitor {
            type Value = Vec3;

            #[inline]
            fn expecting(&self, f: &mut Formatter) -> FResult { write!(f, "a 3D-vector") }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                // Parse three elements
                let x: fty = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let y: fty = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let z: fty = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;

                // Construct the Vec3
                Ok(Vec3 { x, y, z })
            }
        }

        // Call the visitor
        deserializer.deserialize_seq(Vec3Visitor)
    }
}
impl Display for Vec3 {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { write!(f, "({},{},{})", self.x, self.y, self.z) }
}

impl From<[fty; 3]> for Vec3 {
    #[inline]
    fn from(value: [fty; 3]) -> Self {
        let [x, y, z] = value;
        Self { x, y, z }
    }
}
