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


/***** HELPER MACROS *****/
/// Implements [`Number`].
macro_rules! number_impl {
    (f32) => {
        impl Number for f32 {
            const ZERO: Self = 0.0;

            #[inline]
            fn as_f64(self) -> f64 { self as f64 }
        }
    };
    (f64) => {
        impl Number for f64 {
            const ZERO: Self = 0.0;

            #[inline]
            fn as_f64(self) -> f64 { self }
        }
    };
    ($ty:ty) => {
        impl Number for $ty {
            const ZERO: Self = 0;

            #[inline]
            fn as_f64(self) -> f64 { self as f64 }
        }
    };
}





/***** AUXILLARY FUNCTIONS *****/
/// Computes the dot product of two 3D vectors.
///
/// # Arguments
/// - `lhs`: The lefthand-side of the computation.
/// - `rhs`: The righthand-side of the computation.
///
/// # Returns
/// The value of the dot product, as `T`.
#[inline]
pub fn dot3<T: Number>(lhs: Vec3<T>, rhs: Vec3<T>) -> T { lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z }

/// Computes the cross product of two 3D vectors.
///
/// # Arguments
/// - `lhs`: The lefthand-side of the computation.
/// - `rhs`: The righthand-side of the computation.
///
/// # Returns
/// The value of the cross product, as `T`.
#[inline]
pub fn cross3<T: Number>(lhs: Vec3<T>, rhs: Vec3<T>) -> Vec3<T> {
    Vec3 { x: lhs.y * rhs.z - lhs.z * rhs.y, y: lhs.z * rhs.x - lhs.x * rhs.z, z: lhs.x * rhs.y - lhs.y * rhs.x }
}





/***** AUXILLARY TRAITS *****/
/// The `Number` trait implements functions for anything in a vector.
pub trait Number:
    Sized + Copy + Add<Output = Self> + AddAssign + Sub<Output = Self> + SubAssign + Mul<Output = Self> + MulAssign + Div<Output = Self> + DivAssign
{
    const ZERO: Self;

    /// Returns this number as an [`f64`].
    fn as_f64(self) -> f64;
}

// Canonical impls
number_impl!(u8);
number_impl!(u16);
number_impl!(u32);
number_impl!(u64);
number_impl!(u128);
number_impl!(usize);
number_impl!(i8);
number_impl!(i16);
number_impl!(i32);
number_impl!(i64);
number_impl!(i128);
number_impl!(isize);
number_impl!(f32);
number_impl!(f64);



/// Defines a [`Number`] that can be negative.
pub trait Signed: Number + Neg<Output = Self> {}

// Canonical impls
impl Signed for i8 {}
impl Signed for i16 {}
impl Signed for i32 {}
impl Signed for i64 {}
impl Signed for i128 {}
impl Signed for isize {}
impl Signed for f32 {}
impl Signed for f64 {}



/// Defines a [`Number`] that is floating-point.
pub trait Float: Number {}

// Canonical impls
impl Float for f32 {}
impl Float for f64 {}



/***** LIBRARY *****/
/// The `Vec3` class implements a 3D vector. By default, it abstracts over double-precision floats, but this can be changed manually.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Vec3<T = f64> {
    /// The X-coordinate / index 0.
    pub x: T,
    /// The Y-coordinate / index 1.
    pub y: T,
    /// The Z-coordinate / index 2.
    pub z: T,
}

// Constructors
impl<T: Number> Default for Vec3<T> {
    #[inline]
    fn default() -> Self { Self::zeroes() }
}
impl<T> Vec3<T> {
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
    pub const fn new(x: T, y: T, z: T) -> Self { Self { x, y, z } }
}
impl<T: Number> Vec3<T> {
    /// Constructor for the Vec3 that initializes it to all-zeroes.
    ///
    /// # Returns
    /// A new instance of Self with only 0's in it.
    #[inline]
    pub fn zeroes() -> Self { Self { x: T::ZERO, y: T::ZERO, z: T::ZERO } }
}

// Facts
impl<T: Float> Vec3<T> {
    /// Returns whether this Vec3 is _nearly_ zero.
    ///
    /// This is used to "round off" the last bit of colour, and as such, only useful for
    /// floating-point numbers.
    ///
    /// # Returns
    /// True if every component of this vector is below `1e-8`, or false if any of them isn't.
    #[inline]
    pub fn is_nearly_zero(&self) -> bool { self.x.as_f64() < 1e-8 && self.y.as_f64() < 1e-8 && self.z.as_f64() < 1e-8 }
}
impl<T: Number> Vec3<T> {
    /// Computes the length of the vector.
    ///
    /// If you plan to square the length later anyway, consider using [`Vec3::length2()`] instead.
    ///
    /// # Returns
    /// The mathmatical length of this vector.
    #[inline]
    pub fn length(&self) -> f64 { self.length2().sqrt() }

    /// Computes the length of the vector, but still squared.
    ///
    /// Use this is if you plan to square the length later anyway. Else, consider using
    /// [`Vec3::length()`] instead.
    ///
    /// # Returns
    /// The mathmatical length of this vector to the power of two.
    #[inline]
    pub fn length2(&self) -> f64 { (self.x * self.x + self.y * self.y + self.z * self.z).as_f64() }
}

// Custom ops
impl<T: Float> Vec3<T> {
    /// Returns a Vec3 that is the unit vector of this vector.
    ///
    /// # Returns
    /// A new Vec3 of floats that has the same direction but [length](Vec3::length()) `1`.
    #[inline]
    pub fn unit(&self) -> Vec3<f64> {
        let len: f64 = self.length();
        Vec3 { x: self.x.as_f64() / len, y: self.y.as_f64() / len, z: self.z.as_f64() / len }
    }
}

// Std ops
impl<T: Signed> Neg for Vec3<T> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output { Self { x: -self.x, y: -self.y, z: -self.z } }
}

impl<T: Number> Add for Vec3<T> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output { Self { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z } }
}
impl<T: Number> AddAssign for Vec3<T> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}
impl<T: Number> Sub for Vec3<T> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output { Self { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z } }
}
impl<T: Number> SubAssign for Vec3<T> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}
impl<T: Number> Mul for Vec3<T> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output { Self { x: self.x * rhs.x, y: self.y * rhs.y, z: self.z * rhs.z } }
}
impl<T: Number> MulAssign for Vec3<T> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}
impl<T: Number> Div for Vec3<T> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output { Self { x: self.x / rhs.x, y: self.y / rhs.y, z: self.z / rhs.z } }
}
impl<T: Number> DivAssign for Vec3<T> {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl<T: Number> Add<T> for Vec3<T> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: T) -> Self::Output { Self { x: self.x + rhs, y: self.y + rhs, z: self.z + rhs } }
}
impl<T: Number> AddAssign<T> for Vec3<T> {
    #[inline]
    fn add_assign(&mut self, rhs: T) {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
    }
}
impl<T: Number> Sub<T> for Vec3<T> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: T) -> Self::Output { Self { x: self.x - rhs, y: self.y - rhs, z: self.z - rhs } }
}
impl<T: Number> SubAssign<T> for Vec3<T> {
    #[inline]
    fn sub_assign(&mut self, rhs: T) {
        self.x -= rhs;
        self.y -= rhs;
        self.z -= rhs;
    }
}
impl<T: Number> Mul<T> for Vec3<T> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: T) -> Self::Output { Self { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs } }
}
impl<T: Number> MulAssign<T> for Vec3<T> {
    #[inline]
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}
impl<T: Number> Div<T> for Vec3<T> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: T) -> Self::Output { Self { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs } }
}
impl<T: Number> DivAssign<T> for Vec3<T> {
    #[inline]
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Add<Vec3<f64>> for f64 {
    type Output = Vec3<f64>;

    #[inline]
    fn add(self, rhs: Vec3<f64>) -> Self::Output { Vec3 { x: self + rhs.x, y: self + rhs.y, z: self + rhs.z } }
}
impl Sub<Vec3<f64>> for f64 {
    type Output = Vec3<f64>;

    #[inline]
    fn sub(self, rhs: Vec3<f64>) -> Self::Output { Vec3 { x: self - rhs.x, y: self - rhs.y, z: self - rhs.z } }
}
impl Mul<Vec3<f64>> for f64 {
    type Output = Vec3<f64>;

    #[inline]
    fn mul(self, rhs: Vec3<f64>) -> Self::Output { Vec3 { x: self * rhs.x, y: self * rhs.y, z: self * rhs.z } }
}
impl Div<Vec3<f64>> for f64 {
    type Output = Vec3<f64>;

    #[inline]
    fn div(self, rhs: Vec3<f64>) -> Self::Output { Vec3 { x: self / rhs.x, y: self / rhs.y, z: self / rhs.z } }
}

impl<T> Index<usize> for Vec3<T> {
    type Output = T;

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
impl<T> IndexMut<usize> for Vec3<T> {
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

impl<T: Serialize> Serialize for Vec3<T> {
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
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Vec3<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// Visitor for a Vec3
        struct Vec3Visitor<T> {
            _data: std::marker::PhantomData<T>,
        }
        impl<'de, T: Deserialize<'de>> Visitor<'de> for Vec3Visitor<T> {
            type Value = Vec3<T>;

            #[inline]
            fn expecting(&self, f: &mut Formatter) -> FResult { write!(f, "a 3D-vector") }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                // Parse three elements
                let x: T = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let y: T = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let z: T = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;

                // Construct the Vec3
                Ok(Vec3 { x, y, z })
            }
        }

        // Call the visitor
        deserializer.deserialize_seq(Vec3Visitor { _data: Default::default() })
    }
}
impl<T: Display> Display for Vec3<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { write!(f, "({},{},{})", self.x, self.y, self.z) }
}

impl<T> From<[T; 3]> for Vec3<T> {
    #[inline]
    fn from(value: [T; 3]) -> Self {
        let [x, y, z] = value;
        Self { x, y, z }
    }
}

impl<T> AsRef<Vec3<T>> for Vec3<T> {
    #[inline]
    fn as_ref(&self) -> &Self { self }
}
impl<T> AsMut<Vec3<T>> for Vec3<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut Self { self }
}
impl<T: Copy> From<&Vec3<T>> for Vec3<T> {
    #[inline]
    fn from(value: &Vec3<T>) -> Self { *value }
}
impl<T: Copy> From<&mut Vec3<T>> for Vec3<T> {
    #[inline]
    fn from(value: &mut Vec3<T>) -> Self { *value }
}
