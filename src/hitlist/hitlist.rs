//  SYSTEM.rs
//    by Lut99
// 
//  Created:
//    29 Apr 2023, 10:51:41
//  Last edited:
//    05 May 2023, 11:54:17
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements the [`HitList`] itself, which is kind of an ECS but heavily
//!   optimized towards the raytracing use-case.
// 

use enum_debug::EnumDebug;

use crate::math::AABB;
use crate::specifications::objects::{BoundingBoxable, Sphere};
use crate::specifications::objects::utils::surround_list;
use crate::specifications::materials::{Diffuse, NormalMap};
use crate::specifications::scene::{Material, Object};


/***** HELPER STRUCTS *****/
/// Helper iterator that iterates over a list of [`HitItem`]s, only matching groups but skipping those within.
struct ToplevelObjects<'l, T> {
    /// The list to iterate over
    list  : &'l Vec<HitItem<T>>,
    /// Our current index of iteration
    index : usize,
}
impl<'l, T> Iterator for ToplevelObjects<'l, T> {
    type Item = &'l HitItem<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Quit if we grow out-of-bounds
        if self.index >= self.list.len() { return None; }

        // Check how much to progress the index
        let index = self.index;
        self.index += match &self.list[index] {
            HitItem::Object(_) => 1,
            HitItem::Group(b)  => 1 + b.obj,
        };

        // Return it
        Some(&self.list[index])
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { (self.list.len(), Some(self.list.len())) }
}
impl<'l, T> ExactSizeIterator for ToplevelObjects<'l, T> {
    fn len(&self) -> usize { if self.index < self.list.len() { self.list.len() - self.index } else { 0 } }
}





/***** AUXILLARY STRUCTS *****/
/// Wraps around an [`Object`] (or rather, its possible forms) to provide an Axis-Aligned Bounding Box (AABB) that is cheap hitting.
#[derive(Clone, Copy, Debug)]
pub struct BoundingBox<T> {
    /// The object we are wrapping.
    pub obj  : T,
    /// The bounding box we use to cheap.
    pub aabb : AABB,
}
impl<T> BoundingBox<T> {
    /// Constructor for the AABB that computes the box around the given object.
    /// 
    /// # Arguments
    /// - `object`: The object to wrap, which must implement the [`BoundingBoxable`] trait.
    /// 
    /// # Returns
    /// A new instance of a BoundingBox that provides the object plus its box.
    #[inline]
    pub fn new(object: T) -> Self where T: BoundingBoxable {
        let aabb: AABB = object.aabb();
        Self {
            obj : object,
            aabb,
        }
    }
}
impl<T> AsRef<T> for BoundingBox<T> {
    #[inline]
    fn as_ref(&self) -> &T { &self.obj }
}
impl<T> AsMut<T> for BoundingBox<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T { &mut self.obj }
}

impl<T> BoundingBoxable for BoundingBox<T> {
    #[inline]
    fn aabb(&self) -> AABB { self.aabb }
}



/// Defines a thin wrapper around a given type to be able to either refer to it, or to indicate the following objects of that type are part of a group.
#[derive(Clone, Copy, Debug, EnumDebug)]
pub enum HitItem<T> {
    /// The object itself.
    Object(BoundingBox<T>),
    /// The following `.0` items are part of this group.
    Group(BoundingBox<usize>),
}
impl<T> HitItem<T> {
    /// Returns whether this HitItem is a [`HitItem::Object`].
    #[inline]
    pub fn is_object(&self) -> bool { matches!(self, Self::Object(_)) }
    /// Returns this HitItem as if it is a [`HitItem::Object`].
    /// 
    /// # Returns
    /// A reference to the internal `T`.
    /// 
    /// # Panics
    /// This function panics if we were not a [`HitItem::Object`].
    #[inline]
    pub fn object(&self) -> &BoundingBox<T> { if let Self::Object(o) = self { o } else { panic!("Cannot unwrap a HitItem::{} as a HitItem::Object", self.variant()); } }
    /// Returns this HitItem as if it is a [`HitItem::Object`].
    /// 
    /// # Returns
    /// A mutable reference to the internal `T`.
    /// 
    /// # Panics
    /// This function panics if we were not a [`HitItem::Object`].
    #[inline]
    pub fn object_mut(&mut self) -> &mut BoundingBox<T> { if let Self::Object(o) = self { o } else { panic!("Cannot unwrap a HitItem::{} as a HitItem::Object", self.variant()); } }
    /// Returns this HitItem as if it is a [`HitItem::Object`].
    /// 
    /// # Returns
    /// The internal `T`.
    /// 
    /// # Panics
    /// This function panics if we were not a [`HitItem::Object`].
    #[inline]
    pub fn into_object(self) -> BoundingBox<T> { if let Self::Object(o) = self { o } else { panic!("Cannot unwrap a HitItem::{} as a HitItem::Object", self.variant()); } }

    /// Returns whether this HitItem is a [`HitItem::Group`].
    #[inline]
    pub fn is_group(&self) -> bool { matches!(self, Self::Group(_)) }
    /// Returns this HitItem as if it is a [`HitItem::Group`].
    /// 
    /// # Returns
    /// A reference to the number of next elements that are part of this group.
    /// 
    /// # Panics
    /// This function panics if we were not a [`HitItem::Group`].
    #[inline]
    pub fn group(&self) -> &BoundingBox<usize> { if let Self::Group(g) = self { g } else { panic!("Cannot unwrap a HitItem::{} as a HitItem::Group", self.variant()); } }
    /// Returns this HitItem as if it is a [`HitItem::Group`].
    /// 
    /// # Returns
    /// A mutable reference to the number of next elements that are part of this group.
    /// 
    /// # Panics
    /// This function panics if we were not a [`HitItem::Group`].
    #[inline]
    pub fn group_mut(&mut self) -> &mut BoundingBox<usize> { if let Self::Group(g) = self { g } else { panic!("Cannot unwrap a HitItem::{} as a HitItem::Group", self.variant()); } }
    /// Returns this HitItem as if it is a [`HitItem::Group`].
    /// 
    /// # Returns
    /// The number of next elements that are part of this group.
    /// 
    /// # Panics
    /// This function panics if we were not a [`HitItem::Group`].
    #[inline]
    pub fn into_group(self) -> BoundingBox<usize> { if let Self::Group(g) = self { g } else { panic!("Cannot unwrap a HitItem::{} as a HitItem::Group", self.variant()); } }
}
impl<T> BoundingBoxable for HitItem<T> {
    #[inline]
    fn aabb(&self) -> AABB {
        match self {
            Self::Object(o) => o.aabb(),
            Self::Group(g)  => g.aabb(),
        }
    }
}





/***** LIBRARY *****/
/// Defines a list of hittable objects, in an Entity Component System-kinda way.
/// 
/// Note this struct is non-configurable, as it imposes specific preprocessing and ordering on its contents. Specifically:
/// - Every object is sorted by type, for better cache usage and to avoid virtual pointers / conditional branches
/// - Every object's AABB is computed
/// - The object groups are also separated by group, and then flattened for most cache-friendly traversal.
#[derive(Clone, Debug)]
pub struct HitList {
    /// The [`Sphere`]s which use the [`NormalMap`] material.
    sphere_normalmap : Vec<HitItem<Sphere<NormalMap>>>,
    /// The [`Sphere`]s which use the basic [`Diffuse`] material.
    sphere_diffuse   : Vec<HitItem<Sphere<Diffuse>>>,
}

impl HitList {
    /// Provides immutable access to the internal list of normalmap-spheres.
    #[inline]
    pub fn sphere_normalmap(&self) -> &[HitItem<Sphere<NormalMap>>] { &self.sphere_normalmap }
    /// Provides immutable access to the internal list of diffuse-spheres.
    #[inline]
    pub fn sphere_diffuse(&self) -> &[HitItem<Sphere<Diffuse>>] { &self.sphere_diffuse }
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
impl From<&[Object]> for HitList {
    fn from(value: &[Object]) -> Self {
        // Prepare an empty self, and start populating it based on what we find
        let mut result: HitList = HitList {
            sphere_normalmap : vec![],
            sphere_diffuse   : vec![],
        };

        // Iterate over the objects
        for obj in value {
            // Match on the object's type
            match obj {
                // Normal objects
                Object::Sphere(s) => {
                    // Match on the material to find the target list
                    match s.material {
                        Material::NormalMap(n) => {
                            result.sphere_normalmap.push(HitItem::Object(BoundingBox::new(Sphere {
                                center : s.center,
                                radius : s.radius,
        
                                // But first, unpack the material
                                material : n,
                            })));
                        },

                        Material::Diffuse(d) => {
                            result.sphere_diffuse.push(HitItem::Object(BoundingBox::new(Sphere {
                                center : s.center,
                                radius : s.radius,
        
                                // But first, unpack the material
                                material : d,
                            })));
                        },
                    };
                },

                // Groups
                Object::Group(g) => {
                    // Recursively construct ourselves
                    let list: HitList = HitList::from(g);

                    // Compute a bounding box surrounding all of its normalmap spheres (and we skip anything within a group)
                    let aabb: AABB = surround_list(ToplevelObjects{ list: &list.sphere_normalmap, index: 0 });
                    // Then add it to the thing with a group prepended to it
                    result.sphere_normalmap.reserve(1 + list.sphere_normalmap.len());
                    result.sphere_normalmap.push(HitItem::Group(BoundingBox{ obj: list.sphere_normalmap.len(), aabb }));
                    result.sphere_normalmap.extend(&list.sphere_normalmap);

                    // Compute a bounding box surrounding all of its diffuse spheres (and we skip anything within a group)
                    let aabb: AABB = surround_list(ToplevelObjects{ list: &list.sphere_diffuse, index: 0 });
                    // Then add it to the thing with a group prepended to it
                    result.sphere_diffuse.reserve(1 + list.sphere_diffuse.len());
                    result.sphere_diffuse.push(HitItem::Group(BoundingBox{ obj: list.sphere_diffuse.len(), aabb }));
                    result.sphere_diffuse.extend(&list.sphere_diffuse);
                },
            }
        }

        // OK!
        result
    }
}
impl From<&mut [Object]> for HitList {
    #[inline]
    fn from(value: &mut [Object]) -> Self { Self::from(&value[..]) }
}
