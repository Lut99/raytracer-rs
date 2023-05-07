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

use enum_debug::EnumDebug;

use crate::math::{AABB, Colour, Ray, Vec3};
use crate::math::aabb::surround;
use crate::specifications::objects::{BoundingBoxable, Hittable, HitRecord, Sphere};
use crate::specifications::materials::{Diffuse, Material, NormalMap, StaticColour};
use crate::specifications::scene::{IntoInner, Object};


/***** HELPER STRUCTS *****/
/// The HitItem abstracts over either an inline group marker or an actual object.
#[derive(Clone, Copy, Debug, EnumDebug)]
enum HitItem<T> {
    /// It's a real object.
    Object(T, AABB),
    /// It's a group marker.
    GroupMarker(usize, AABB),
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
    fn object(&self) -> &T { if let Self::Object(o, _) = self { o } else { panic!("Cannot unwrap HitItem::{} as HitItem::Object", self.variant()); } }



    /// Returns the Axis-Aligned Bounding Box (AABB) for this item.
    /// 
    /// # Returns
    /// The internal [`AABB`].
    #[inline]
    fn aabb(&self) -> AABB {
        match self {
            Self::Object(_, aabb)      => *aabb,
            Self::GroupMarker(_, aabb) => *aabb,
        }
    }
}



/// The HitVec implements a hitlist for objects of a single type.
#[derive(Clone, Debug)]
struct HitVec<T> {
    /// The list of items to hit
    items : Vec<HitItem<T>>,
}

impl<T: Hittable> HitVec<T> {
    /// Computes the hit of the given ray with the closest object, if any.
    /// 
    /// # Arguments
    /// - `ray`: The [`Ray`] to shoot through the scene.
    /// - `t_min`: The minimum distance from the `ray`'s origin (along the ray) that we decided still counts as a hit.
    /// - `t_max`: The maximum distance from the `ray`'s origin (along the ray) that we decided still counts as a hit.
    /// 
    /// # Returns
    /// A tuple of the index of the item that was hit and a [`HitRecord`] that contains information about the hit. If the ray never hits anything at all, then [`None`] is returned.
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64) -> Option<(usize, HitRecord)> {
        // Prepare the placeholder for the closest hit
        let mut closest: Option<(usize, HitRecord)> = None;

        // Iterate over the internal objects
        let mut iter = self.items.iter().enumerate();
        while let Some((i, item)) = iter.next() {
            // Match on the type of item
            match item {
                HitItem::Object(obj, aabb) => {
                    // Compute the AABB first as a cheap hit, then the expensive object hit
                    if aabb.hit(ray, t_min, t_max) {
                        if let Some(record) = obj.hit(ray, t_min, t_max) {
                            // Now only replace the closest if the new one is closer (or there wasn't one yet)
                            if let Some(old_record) = &closest {
                                if record.t < old_record.1.t { closest = Some((i, record)); }
                            } else {
                                closest = Some((i, record));
                            }
                        }
                    }
                },

                HitItem::GroupMarker(n, aabb) => {
                    // If the bounding box does _not_ hit, we can skip all these objects
                    if !aabb.hit(ray, t_min, t_max) {
                        // Note that we skip one less element than `n`, since it is a count and not an index (i.e., one-indexed -> zero-indexed)
                        if *n > 0 { iter.nth(n - 1); }
                    }

                    // Now continue with whatever element is on top
                    continue;
                },
            }
        }

        // Done, return the closest hit (if any)
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
                let aabb: AABB = o.aabb();
                items.push(HitItem::Object(o, aabb));

            } else if let Object::Group(g) = o {
                // If it's a group, let us recurse to find the set of items we need
                let group_items: Vec<HitItem<T>> = Self::from(g).items;

                // Construct a bounding box perfectly fitting these items
                let mut aabb: AABB = AABB::new(Vec3::zeroes(), Vec3::zeroes());
                for g in &group_items {
                    aabb = surround(aabb, g.aabb());
                }

                // Insert the group marker for these items, and then the items
                items.push(HitItem::GroupMarker(group_items.len(), aabb));
                items.extend(group_items);
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





/***** AUXILLARY *****/
/// The `HitIndex` enum sources a hit to a location within the [`HitList`].
#[derive(Clone, Copy, Debug, EnumDebug, Eq, Hash, PartialEq)]
pub enum HitIndex {
    /// It's a [`Sphere`] with the [`StaticColour`] material.
    SphereStaticColour(usize),
    /// It's a [`Sphere`] with the [`NormalMap`] material.
    SphereNormalMap(usize),
    /// It's a [`Sphere`] with the [`Diffuse`] material.
    SphereDiffuse(usize),
}





/***** LIBRARY *****/
/// The HitList implements a list of all objects in a scene, effeciently hittable in an ECS-like style.
#[derive(Clone, Debug)]
pub struct HitList {
    /// The list of spheres that have the static material.
    sphere_staticcolour : HitVec<Sphere<StaticColour>>,
    /// The list of spheres that have the normalmap material.
    sphere_normalmap    : HitVec<Sphere<NormalMap>>,
    /// The list of spheres that have the diffuse material.
    sphere_diffuse      : HitVec<Sphere<Diffuse>>,
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
    pub fn hit(&self, ray: Ray, t_min: f64, t_max: f64) -> Option<(HitIndex, HitRecord)> {
        // Compute the first list first as the first option of a hit
        let mut closest: Option<(HitIndex, HitRecord)> = self.sphere_staticcolour.hit(ray, t_min, t_max).map(|(i, r)| (HitIndex::SphereStaticColour(i), r));

        // Now update it only if any of the other lists hit closer
        if let Some(record) = self.sphere_normalmap.hit(ray, t_min, t_max) {
            if let Some(old_record) = &closest {
                if record.1.t < old_record.1.t { closest = Some((HitIndex::SphereNormalMap(record.0), record.1)); }
            } else {
                closest = Some((HitIndex::SphereNormalMap(record.0), record.1));
            }
        }
        if let Some(record) = self.sphere_diffuse.hit(ray, t_min, t_max) {
            if let Some(old_record) = &closest {
                if record.1.t < old_record.1.t { closest = Some((HitIndex::SphereDiffuse(record.0), record.1)); }
            } else {
                closest = Some((HitIndex::SphereDiffuse(record.0), record.1));
            }
        }

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
    pub fn scatter(&self, ray: Ray, index: HitIndex, record: HitRecord) -> (Option<Ray>, Colour) {
        // Get the object that was talked about
        match index {
            HitIndex::SphereStaticColour(i) => self.sphere_staticcolour[i].material.scatter(ray, record),
            HitIndex::SphereNormalMap(i)    => self.sphere_normalmap[i].material.scatter(ray, record),
            HitIndex::SphereDiffuse(i)      => self.sphere_diffuse[i].material.scatter(ray, record),
        }
    }



    /// Returns the total number of objects in this list.
    #[inline]
    pub fn len(&self) -> usize {
        self.n_spheres()
    }

    /// Returns the number of spheres in this list.
    #[inline]
    pub fn n_spheres(&self) -> usize {
        self.sphere_staticcolour.items.len() + self.sphere_normalmap.items.len() + self.sphere_diffuse.items.len()
    }
}

impl From<&[Object]> for HitList {
    #[inline]
    fn from(value: &[Object]) -> Self {
        Self {
            sphere_staticcolour : HitVec::from(value),
            sphere_normalmap    : HitVec::from(value),
            sphere_diffuse      : HitVec::from(value),
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
