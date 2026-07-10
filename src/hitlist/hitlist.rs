//  HITLIST.rs
//    by Lut99
//
//  Created:
//    07 May 2023, 11:13:55
//  Last edited:
//    07 May 2023, 12:46:50
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the [`HitList`] and associates, which implements an
//!   ECS-like list of hittable objects.
//

use std::ops::Index;

use crate::math::{AABB, Colour, Ray};
use crate::specifications::animations::Vertical;
use crate::specifications::materials::{Dielectric, Diffuse, Lambertian, Material, Metal, NormalMap, PartialDielectric, StaticColour};
use crate::specifications::objects::{AnimatedSphere, BoundingBoxable, HitRecord, Hittable, Sphere};
use crate::specifications::scene::{Environment, IntoInner, Object};


/***** HELPER FUNCTINS *****/
/// Computes the AABB of a whole slice of objects.
///
/// The AABB's are stored in a slice that mirrors the objects.
fn aabb_slice<T: BoundingBoxable>(slice: &[HitItem<T>], t_us: u64, aabbs: &mut [Option<AABB>]) {
    let mut iter = slice.into_iter().enumerate();
    while let Some((i, item)) = iter.next() {
        match item {
            HitItem::Object(o) => {
                aabbs[i] = Some(o.aabb(t_us));
            },
            HitItem::BVH(len) => {
                // Populate the nested AABB's first
                let nested_aabbs = &mut aabbs[i + 1..i + 1 + *len];
                aabb_slice(&slice[i + 1..i + 1 + *len], t_us, nested_aabbs);

                // Compute an AABB for this group
                let mut res: Option<AABB> = None;
                for aabb in nested_aabbs {
                    let Some(aabb) = aabb else { continue };
                    if let Some(res) = &mut res {
                        *res = AABB::surround(*res, *aabb);
                    } else {
                        res = Some(*aabb);
                    }
                }
                aabbs[i] = res;

                // Skip these elements
                iter.nth(*len);
            },
        }
    }
}





/***** HELPER STRUCTS *****/
/// The HitItem abstracts over either an inline group marker or an actual object.
#[derive(Clone, Copy, Debug)]
enum HitItem<T> {
    /// It's a real object.
    Object(T),
    /// It's a marker indicating a group computed by the BVH algorithm.
    ///
    /// The index recounts how many of the next items are part of it. This may be nested!
    BVH(usize),
}
impl<T> HitItem<T> {
    /// Returns the internal object if this HitItem is one.
    ///
    /// # Returns
    /// A reference to the internal `T`.
    ///
    /// # Panics
    /// This function may panic if [`Self::is_object()`] returns false (i.e., we are not a [`HitItem::Object`] after all).
    #[inline]
    #[track_caller]
    fn object(&self) -> &T {
        match self {
            Self::Object(o) => o,
            _ => panic!("Cannot unwrap non-HitItem::Object as a HitItem::Object"),
        }
    }
}



/// The HitVec implements a hitlist for objects of a single type.
#[derive(Clone, Debug)]
struct HitVec<T> {
    /// The list of items to hit
    items: Vec<HitItem<T>>,
}

impl<T: Hittable> HitVec<T> {
    /// Organise the objects in this HitVec into a BVH (Bounded Volume Hiearchy) for efficient
    /// hittable search.
    ///
    /// # Arguments
    /// - `t_us`: The time since the start of the scene (as microseconds) at which we compute
    ///   AABBs.
    fn recompute_bvh(&mut self) {}



    /// Computes the hit of the given ray with the closest object, if any.
    ///
    /// # Arguments
    /// - `ray`: The [`Ray`] to shoot through the scene.
    /// - `t_min`: The minimum distance from the `ray`'s origin (along the ray) that we decided still counts as a hit.
    /// - `t_max`: The maximum distance from the `ray`'s origin (along the ray) that we decided still counts as a hit.
    ///
    /// # Returns
    /// A tuple of the index of the item that was hit and a [`HitRecord`] that contains information about the hit. If the ray never hits anything at all, then [`None`] is returned.
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<(usize, HitRecord)> {
        // Pre-compute all AABBs
        let mut aabbs: Vec<Option<AABB>> = vec![None; self.items.len()];
        aabb_slice(&self.items, ray.time, &mut aabbs);

        // Iterate over all the items in the slice to find the closest...
        let mut closest: Option<(usize, HitRecord)> = None;
        let mut iter = self.items.iter().enumerate();
        while let Some((i, item)) = iter.next() {
            match item {
                HitItem::Object(o) => {
                    // Compute the AABB first as a cheap hit, then the expensive object hit
                    if aabbs[i].map(|aabb| aabb.hit(ray, t_min, t_max)).unwrap_or(true) {
                        if let Some(record) = o.hit(ray, t_min, t_max, env) {
                            // Now only replace the closest if the new one is closer (or there wasn't one yet)
                            if let Some(old_record) = &closest {
                                if record.t < old_record.1.t {
                                    closest = Some((i, record));
                                }
                            } else {
                                closest = Some((i, record));
                            }
                        }
                    }
                },
                HitItem::BVH(len) => {
                    // If we _don't_ hit the AABB, skip this group of items
                    if !aabbs[i].map(|aabb| aabb.hit(ray, t_min, t_max)).unwrap_or(true) {
                        iter.nth(*len);
                    }
                    // Else, continue and compute the hit of the objects in the group
                },
            }
        }
        closest
    }
}
impl<T> Index<usize> for HitVec<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output { self.items[index].object() }
}

impl<T> From<&[Object]> for HitVec<T>
where
    T: BoundingBoxable,
    Object: IntoInner<T>,
{
    fn from(value: &[Object]) -> Self {
        // Let us iterate over the list of objects
        let mut items: Vec<HitItem<T>> = Vec::with_capacity(value.len());
        for o in value {
            // Check if we are ourselves
            if let Some(o) = <Object as IntoInner<T>>::into_inner(o.clone()) {
                // We are, so we can now just add us (neat!)
                items.push(HitItem::Object(o));
            }

            // We ignore other cases (that's for other lists)
        }

        // Done
        items.shrink_to_fit();
        Self { items }
    }
}
impl<T> From<&mut [Object]> for HitVec<T>
where
    T: BoundingBoxable,
    Object: IntoInner<T>,
{
    #[inline]
    fn from(value: &mut [Object]) -> Self { Self::from(&*value) }
}
impl<T> From<Vec<Object>> for HitVec<T>
where
    T: BoundingBoxable,
    Object: IntoInner<T>,
{
    #[inline]
    fn from(value: Vec<Object>) -> Self { Self::from(value.as_slice()) }
}
impl<T> From<&Vec<Object>> for HitVec<T>
where
    T: BoundingBoxable,
    Object: IntoInner<T>,
{
    #[inline]
    fn from(value: &Vec<Object>) -> Self { Self::from(value.as_slice()) }
}
impl<T> From<&mut Vec<Object>> for HitVec<T>
where
    T: BoundingBoxable,
    Object: IntoInner<T>,
{
    #[inline]
    fn from(value: &mut Vec<Object>) -> Self { Self::from(value.as_slice()) }
}





/***** LIBRARY *****/
/// Implements the [`HitList`] based on a list of object/material pairs.
macro_rules! hit_list_impl {
    ([$($obj:ident<$mat:ident>),* $(,)?], [$($aobj:ident<$amat:ident, $ani:ident>),* $(,)?] $(,)?) => {
        ::paste::paste! {
            /// The `HitIndex` enum sources a hit to a location within the [`HitList`].
            #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
            pub enum HitIndex {
                $(
                #[doc = concat!("It's a [`", stringify!($obj), "`] with the [`", stringify!($mat), "`] material.")]
                [<$obj $mat>](usize),
                )*
                $(
                #[doc = concat!("It's a [`", stringify!($aobj), "`] with the [`", stringify!($amat), "`] material and [`", stringify!($ani), "`] animation.")]
                [<$aobj $amat $ani>](usize),
                )*
            }



            /// The HitList implements a list of all objects in a scene, effeciently hittable in an ECS-like style.
            #[derive(Clone, Debug)]
            pub struct HitList {
                $([<$obj:lower _ $mat:lower>]: HitVec<$obj<$mat>>,)*
                $([<$aobj:lower _ $amat:lower _ $ani:lower>]: HitVec<$aobj<$amat, $ani>>,)*
            }

            impl HitList {
                /// Computes the hit of the given ray with the closest object, if any.
                ///
                /// # Arguments
                /// - `ray`: The [`Ray`] to shoot through the scene.
                /// - `t_min`: The minimum distance from the `ray`'s origin (along the ray) that we decided still counts as a hit.
                /// - `t_max`: The maximum distance from the `ray`'s origin (along the ray) that we decided still counts as a hit.
                ///
                /// # Returns
                /// A tuple of the index within this HitList of the object that was hit and a new [`HitRecord`] that contains information about the hit. If the ray never hits anything at all, then [`None`] is returned.
                pub fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<(HitIndex, HitRecord)> {
                    let mut closest: Option<(HitIndex, HitRecord)> = None;

                    $(if let Some(record) = self.[<$obj:lower _ $mat:lower>].hit(ray, t_min, t_max, env) {
                        if let Some(old_record) = &closest {
                            if record.1.t < old_record.1.t {
                                closest = Some((HitIndex::[<$obj $mat>](record.0), record.1));
                            }
                        } else {
                            closest = Some((HitIndex::[<$obj $mat>](record.0), record.1));
                        }
                    })*
                    $(if let Some(record) = self.[<$aobj:lower _ $amat:lower _ $ani:lower>].hit(ray, t_min, t_max, env) {
                        if let Some(old_record) = &closest {
                            if record.1.t < old_record.1.t {
                                closest = Some((HitIndex::[<$aobj $amat $ani>](record.0), record.1));
                            }
                        } else {
                            closest = Some((HitIndex::[<$aobj $amat $ani>](record.0), record.1));
                        }
                    })*

                    // Return the closest hit (if any)
                    closest
                }



                /// Scatters a particular hit, returning the new ray to shoot and the colour to apply on the way back to the camera.
                ///
                /// # Arguments
                /// - `ray`: The shot [`Ray`] that produced the hit.
                /// - `index`: The HitList-specific [`HitIndex`] that we can use to know which object was hit.
                /// - `record`: The [`HitRecord`] that records where the `ray` hit.
                ///
                /// # Returns
                /// A tuple of an optional, new [`Ray`] and a mandatory colour. If the ray is [`None`], then it may be interepreted as that the material does not bounce further.
                pub fn scatter(&self, ray: Ray, index: HitIndex, record: HitRecord, env: &Environment) -> (Option<Ray>, Colour) {
                    // Get the object that was talked about
                    match index {
                        $(HitIndex::[<$obj $mat>](i) => self.[<$obj:lower _ $mat:lower>][i].material.scatter(ray, record, env),)*
                        $(HitIndex::[<$aobj $amat $ani>](i) => self.[<$aobj:lower _ $amat:lower _ $ani:lower>][i].sphere.material.scatter(ray, record, env),)*
                    }
                }



                /// Returns the total number of objects in this list.
                #[inline]
                pub fn len(&self) -> usize { 0 $(+ self.[<$obj:lower _ $mat:lower>].items.len())* $(+ self.[<$aobj:lower _ $amat:lower _ $ani:lower>].items.len())* }
            }

            impl From<&[Object]> for HitList {
                #[inline]
                fn from(value: &[Object]) -> Self {
                    Self {
                        $([<$obj:lower _ $mat:lower>]: HitVec::from(value),)*
                        $([<$aobj:lower _ $amat:lower _ $ani:lower>]: HitVec::from(value),)*
                    }
                }
            }
            impl From<&mut [Object]> for HitList {
                #[inline]
                fn from(value: &mut [Object]) -> Self { Self::from(&*value) }
            }
            impl From<Vec<Object>> for HitList {
                #[inline]
                fn from(value: Vec<Object>) -> Self { Self::from(value.as_slice()) }
            }
            impl From<&Vec<Object>> for HitList {
                #[inline]
                fn from(value: &Vec<Object>) -> Self { Self::from(value.as_slice()) }
            }
            impl From<&mut Vec<Object>> for HitList {
                #[inline]
                fn from(value: &mut Vec<Object>) -> Self { Self::from(value.as_slice()) }
            }
        }
    };
}

// Actual implementation
hit_list_impl!([
    Sphere<StaticColour>,
    Sphere<NormalMap>,
    Sphere<Diffuse>,
    Sphere<Lambertian>,
    Sphere<Metal>,
    Sphere<PartialDielectric>,
    Sphere<Dielectric>,
], [
    AnimatedSphere<StaticColour, Vertical>,
    AnimatedSphere<NormalMap, Vertical>,
    AnimatedSphere<Diffuse, Vertical>,
    AnimatedSphere<Lambertian, Vertical>,
    AnimatedSphere<Metal, Vertical>,
    AnimatedSphere<PartialDielectric, Vertical>,
    AnimatedSphere<Dielectric, Vertical>,
]);
