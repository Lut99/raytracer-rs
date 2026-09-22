//  PDF.rs
//    by Lut99
//
//  Description:
//!   Implements an abstract idea of Probability Density Functions (PDF), and
//!   some concrete ones while at it.
//

use std::cell::{Ref, RefMut};
use std::f64::consts::PI;
use std::rc::Rc;
use std::sync::{Arc, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

use super::super::materials::diffuse::random3_cosine_direction;
use super::super::scene::Environment;
use crate::math::camera::random_in_unit_disk;
use crate::math::onb::ONB;
use crate::math::ray::Ray;
use crate::math::vec3::Vec3;
use crate::random;


/***** HELPER MACROS *****/
macro_rules! pdf_ptr_impl {
    ('a, $ty:ty) => {
        impl<'a, T: PDF> PDF for $ty {
            #[inline]
            fn value(&self, direct: Ray, env: &Environment) -> f64 { <T as PDF>::value(self, direct, env) }

            #[inline]
            fn sample(&self, t_us: u64, origin: Vec3) -> Vec3 { <T as PDF>::sample(self, t_us, origin) }
        }
    };

    ($ty:ty) => {
        impl<T: PDF> PDF for $ty {
            #[inline]
            fn value(&self, direct: Ray, env: &Environment) -> f64 { <T as PDF>::value(self, direct, env) }

            #[inline]
            fn sample(&self, t_us: u64, origin: Vec3) -> Vec3 { <T as PDF>::sample(self, t_us, origin) }
        }
    };
}





/***** INTERFACES *****/
/// Abstracts over various Probability Density Functions, for convenience.
pub trait PDF {
    /// Sample a value from the PDF based on the direction a vector points in.
    ///
    /// # Arguments
    /// - `direct`: A [`Ray`] that represents the ray of which we check the value.
    /// - `env`: An [`Environment`] that is used to do general scene information.
    ///
    /// # Returns
    /// A [`f64`] representing the probability of a ray landing on this direction.
    fn value(&self, direct: Ray, env: &Environment) -> f64;

    /// Generates a random vector in a direction weighted by this PDF.
    ///
    /// Actually, generates a direction for a ray starting in `origin`.
    ///
    /// # Arguments
    /// - `t_us`: The time, in us, since the start of the scene.
    /// - `origin`: The origin to shoot the ray from.
    ///
    /// # Returns
    /// A new [`Vec3`] randomly sampled from this PDF.
    fn sample(&self, t_us: u64, origin: Vec3) -> Vec3;
}

// Pointer-like impls
pdf_ptr_impl!('a, &'a T);
pdf_ptr_impl!('a, &'a mut T);
pdf_ptr_impl!(std::boxed::Box<T>);
pdf_ptr_impl!(Rc<T>);
pdf_ptr_impl!(Arc<T>);
pdf_ptr_impl!('a, Ref<'a, T>);
pdf_ptr_impl!('a, RefMut<'a, T>);
pdf_ptr_impl!('a, RwLockReadGuard<'a, T>);
pdf_ptr_impl!('a, RwLockWriteGuard<'a, T>);
pdf_ptr_impl!('a, MutexGuard<'a, T>);
pdf_ptr_impl!('a, parking_lot::RwLockReadGuard<'a, T>);
pdf_ptr_impl!('a, parking_lot::RwLockWriteGuard<'a, T>);
pdf_ptr_impl!('a, parking_lot::MutexGuard<'a, T>);





/***** LIBRARY *****/
/// A uniform PDF over the unit sphere.
pub struct UnitPDF;

// Interfaces
impl PDF for UnitPDF {
    #[inline]
    fn value(&self, _direct: Ray, _env: &Environment) -> f64 { 1.0 / (4.0 * PI) }

    #[inline]
    fn sample(&self, _t_us: u64, _origin: Vec3) -> Vec3 { random_in_unit_disk() }
}



/// Cosine PDF over the unit sphere.
pub struct CosinePDF {
    /// The orthonormal basis for coordinates on the unit sphere.
    pub onb: ONB,
}

// Interfaces
impl PDF for CosinePDF {
    #[inline]
    fn value(&self, direct: Ray, _env: &Environment) -> f64 {
        let cosine_theta = direct.direct.unit().dot(self.onb.w);
        f64::max(0.0, cosine_theta / PI)
    }

    #[inline]
    fn sample(&self, _t_us: u64, _origin: Vec3) -> Vec3 { self.onb.transform(random3_cosine_direction()) }
}



/// PDF over objects.
pub struct LightPDF<I> {
    /// An iterator yielding the objects to sample from.
    pub objs: I,
}

// Interfaces
impl<I: Clone + ExactSizeIterator + Iterator<Item = impl PDF>> PDF for LightPDF<I> {
    #[inline]
    fn value(&self, direct: Ray, env: &Environment) -> f64 {
        // Weight all of the objects equally
        let iter = self.objs.clone();
        let weight: f64 = 1.0 / iter.len() as f64;
        iter.map(|p| weight * p.value(direct, env)).sum::<f64>()
    }

    #[inline]
    fn sample(&self, t_us: u64, origin: Vec3) -> Vec3 {
        // Weight all of the objects equally
        let mut iter = self.objs.clone();
        let i: usize = random::usizeint(0, iter.len());
        // SAFETY: `i` is always within range of `iter`, so unwrapping this is sound.
        iter.nth(i).unwrap().sample(t_us, origin)
    }
}



/// PDF over two PDFs.
pub struct MixturePDF<P1, P2> {
    /// The first pdf.
    pub p1: P1,
    /// The second PDF.
    pub p2: P2,
}

// Interface
impl<P1: PDF, P2: PDF> PDF for MixturePDF<P1, P2> {
    #[inline]
    fn value(&self, direct: Ray, env: &Environment) -> f64 { 0.5 * self.p1.value(direct, env) + 0.5 * self.p2.value(direct, env) }

    #[inline]
    fn sample(&self, t_us: u64, origin: Vec3) -> Vec3 {
        if fastrand::f64() < 0.5 { self.p1.sample(t_us, origin) } else { self.p2.sample(t_us, origin) }
    }
}
