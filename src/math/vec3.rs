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

use num_traits::{AsPrimitive, Num, NumAssign, NumCast, Signed, Zero};
use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer, Visitor};
use serde::ser::{Serializer, SerializeTuple as _};


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
pub fn dot3<T: Copy + Num>(lhs: Vec3<T>, rhs: Vec3<T>) -> T {
    lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z
}

/// Computes the cross product of two 3D vectors.
/// 
/// # Arguments
/// - `lhs`: The lefthand-side of the computation.
/// - `rhs`: The righthand-side of the computation.
/// 
/// # Returns
/// The value of the cross product, as `T`.
#[inline]
pub fn cross3<T: Copy + Num>(lhs: Vec3<T>, rhs: Vec3<T>) -> Vec3<T> {
    Vec3 {
        x : lhs.y * rhs.z + lhs.z * rhs.y,
        y : lhs.z * rhs.x + lhs.x * rhs.z,
        z : lhs.x * rhs.y + lhs.y * rhs.x,
    }
}





/***** AUXILLARY TRAITS *****/
/// The `Vector` trait implements functions for vectors of any size.
pub trait Vector: Sized + Copy + Neg + Add + AddAssign + Sub + SubAssign + Mul + MulAssign + Div + DivAssign + Index<usize> + IndexMut<usize> {
    /// Returns whether this Vector is (nearly) zero.
    /// 
    /// Essentially, just checks if `x`, `y` and `z` are all (individually) below some close-to-zero value.
    /// 
    /// # Returns
    /// true if this Vector is essentially zero, or false otherwise.
    fn is_nearly_zero(&self) -> bool;

    /// Returns the unit vector equivalent of this vector.
    /// 
    /// # Returns
    /// A new `Vec3` that is the unit vector of this vector.
    fn unit(&self) -> Self;
    /// Computes the mathmatical length of this vector.
    /// 
    /// # Returns
    /// The length of this vector, as a double-precision floating-point.
    fn length(&self) -> f64 { self.length2().sqrt() }
    /// Computes the mathmatical length squared of this vector.
    /// 
    /// # Returns
    ///  The length of this vector squared, as a double-precision floating-point.
    fn length2(&self) -> f64;

    /// Returns the programmatic length of this vector, i.e., the number of elements in it.
    /// 
    /// # Returns
    /// The number of elements in this vector, as a [`usize`].
    fn len(&self) -> usize;
}





/***** LIBRARY *****/
/// The `Vec3` class implements a 3D vector. By default, it abstracts over double-precision floats, but this can be changed manually.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Vec3<T = f64> {
    /// The X-coordinate / index 0.
    pub x : T,
    /// The Y-coordinate / index 1.
    pub y : T,
    /// The Z-coordinate / index 2.
    pub z : T,
}

impl<T: Zero> Default for Vec3<T> {
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
    pub fn new(x: impl Into<T>, y: impl Into<T>, z: impl Into<T>) -> Self {
        Self {
            x : x.into(),
            y : y.into(),
            z : z.into(),
        }
    }

    /// Constructor for the Vec3 that initializes it to all-zeroes.
    /// 
    /// # Returns
    /// A new instance of Self with only 0's in it.
    #[inline]
    pub fn zeroes() -> Self where T: Zero {
        Self {
            x : T::zero(),
            y : T::zero(),
            z : T::zero(),
        }
    }
}
impl<T: Copy + AsPrimitive<f64> + NumAssign + NumCast + Signed> Vector for Vec3<T> {
    #[inline]
    fn is_nearly_zero(&self) -> bool {
        self.x.as_() < 1e-8 && self.y.as_() < 1e-8 && self.z.as_() < 1e-8
    }

    #[inline]
    fn unit(&self) -> Self {
        let len: f64 = self.length();
        Self {
            x : T::from(self.x.as_() / len).unwrap_or_else(|| panic!("Cannot compute unit length of Vec3<{}>, since element type cannot be created from f64", std::any::type_name::<T>())),
            y : T::from(self.y.as_() / len).unwrap_or_else(|| panic!("Cannot compute unit length of Vec3<{}>, since element type cannot be created from f64", std::any::type_name::<T>())),
            z : T::from(self.z.as_() / len).unwrap_or_else(|| panic!("Cannot compute unit length of Vec3<{}>, since element type cannot be created from f64", std::any::type_name::<T>())),
        }
    }
    #[inline]
    fn length2(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).as_()
    }

    #[inline]
    fn len(&self) -> usize { 3 }
}

impl<T: Signed> Neg for Vec3<T> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            x : -self.x,
            y : -self.y,
            z : -self.z,
        }
    }
}

impl<T: Num> Add for Vec3<T> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x : self.x + rhs.x,
            y : self.y + rhs.y,
            z : self.z + rhs.z,
        }
    }
}
impl<T: NumAssign> AddAssign for Vec3<T> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}
impl<T: Num> Sub for Vec3<T> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x : self.x - rhs.x,
            y : self.y - rhs.y,
            z : self.z - rhs.z,
        }
    }
}
impl<T: NumAssign> SubAssign for Vec3<T> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}
impl<T: Num> Mul for Vec3<T> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x : self.x * rhs.x,
            y : self.y * rhs.y,
            z : self.z * rhs.z,
        }
    }
}
impl<T: NumAssign> MulAssign for Vec3<T> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}
impl<T: Num> Div for Vec3<T> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x : self.x / rhs.x,
            y : self.y / rhs.y,
            z : self.z / rhs.z,
        }
    }
}
impl<T: NumAssign> DivAssign for Vec3<T> {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl<T: Copy + Num> Add<T> for Vec3<T> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: T) -> Self::Output {
        Self {
            x : self.x + rhs,
            y : self.y + rhs,
            z : self.z + rhs,
        }
    }
}
impl<T: Copy + NumAssign> AddAssign<T> for Vec3<T> {
    #[inline]
    fn add_assign(&mut self, rhs: T) {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
    }
}
impl<T: Copy + Num> Sub<T> for Vec3<T> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: T) -> Self::Output {
        Self {
            x : self.x - rhs,
            y : self.y - rhs,
            z : self.z - rhs,
        }
    }
}
impl<T: Copy + NumAssign> SubAssign<T> for Vec3<T> {
    #[inline]
    fn sub_assign(&mut self, rhs: T) {
        self.x -= rhs;
        self.y -= rhs;
        self.z -= rhs;
    }
}
impl<T: Copy + Num> Mul<T> for Vec3<T> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x : self.x * rhs,
            y : self.y * rhs,
            z : self.z * rhs,
        }
    }
}
impl<T: Copy + NumAssign> MulAssign<T> for Vec3<T> {
    #[inline]
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}
impl<T: Copy + Num> Div<T> for Vec3<T> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: T) -> Self::Output {
        Self {
            x : self.x / rhs,
            y : self.y / rhs,
            z : self.z / rhs,
        }
    }
}
impl<T: Copy + NumAssign> DivAssign<T> for Vec3<T> {
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
    fn add(self, rhs: Vec3<f64>) -> Self::Output {
        Vec3 {
            x : self + rhs.x,
            y : self + rhs.y,
            z : self + rhs.z,
        }
    }
}
impl Sub<Vec3<f64>> for f64 {
    type Output = Vec3<f64>;

    #[inline]
    fn sub(self, rhs: Vec3<f64>) -> Self::Output {
        Vec3 {
            x : self - rhs.x,
            y : self - rhs.y,
            z : self - rhs.z,
        }
    }
}
impl Mul<Vec3<f64>> for f64 {
    type Output = Vec3<f64>;

    #[inline]
    fn mul(self, rhs: Vec3<f64>) -> Self::Output {
        Vec3 {
            x : self * rhs.x,
            y : self * rhs.y,
            z : self * rhs.z,
        }
    }
}
impl Div<Vec3<f64>> for f64 {
    type Output = Vec3<f64>;

    #[inline]
    fn div(self, rhs: Vec3<f64>) -> Self::Output {
        Vec3 {
            x : self / rhs.x,
            y : self / rhs.y,
            z : self / rhs.z,
        }
    }
}

impl<T> Index<usize> for Vec3<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            i => { panic!("Index '{i}' is out-of-range for Vec3"); },
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
            i => { panic!("Index '{i}' is out-of-range for Vec3"); },
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
        struct Vec3Visitor<T> { _data: std::marker::PhantomData<T>, }
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
                Ok(Vec3 {
                    x,
                    y,
                    z,
                })
            }
        }

        // Call the visitor
        deserializer.deserialize_seq(Vec3Visitor{ _data: Default::default() })
    }
}
impl<T: Display> Display for Vec3<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "({},{},{})", self.x, self.y, self.z)
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
